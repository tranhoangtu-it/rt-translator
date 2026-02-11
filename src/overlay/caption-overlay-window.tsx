import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface CaptionItem {
  id: string;
  text: string;
  /** target_lang -> translated text (final) */
  translations: Record<string, string>;
  timestamp: number;
}

interface CaptionUpdatePayload {
  segment_id: string;
  text: string;
  target_lang: string;
  is_final: boolean;
}

const MAX_CAPTIONS = 4;

export function CaptionOverlayWindow() {
  const [captions, setCaptions] = useState<CaptionItem[]>([]);
  const [pending, setPending] = useState<Record<string, Record<string, string>>>({});
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let mounted = true;

    // Listen for stt-partial events (original text)
    const unlistenStt = listen<{
      text: string;
      segment_id: string;
      start_ms: number;
      is_final: boolean;
    }>("stt-partial", (event) => {
      if (!mounted || !event.payload.is_final) return;
      const p = event.payload;
      const id = p.segment_id;
      setCaptions((prev) => {
        const exists = prev.some((c) => c.id === id);
        if (exists) return prev;
        const next = [
          ...prev,
          { id, text: p.text, translations: {}, timestamp: p.start_ms },
        ];
        return next.slice(-MAX_CAPTIONS);
      });
    });

    // Listen for translation-update events (multi-lang)
    const unlistenTranslation = listen<CaptionUpdatePayload>(
      "translation-update",
      (event) => {
        if (!mounted) return;
        const { segment_id, text, target_lang, is_final } = event.payload;

        if (is_final) {
          setCaptions((prev) =>
            prev.map((c) =>
              c.id === segment_id
                ? { ...c, translations: { ...c.translations, [target_lang]: text } }
                : c,
            ),
          );
          setPending((prev) => {
            const segPending = { ...(prev[segment_id] || {}) };
            delete segPending[target_lang];
            if (Object.keys(segPending).length === 0) {
              const { [segment_id]: _, ...rest } = prev;
              return rest;
            }
            return { ...prev, [segment_id]: segPending };
          });
        } else {
          setPending((prev) => ({
            ...prev,
            [segment_id]: { ...(prev[segment_id] || {}), [target_lang]: text },
          }));
        }
      },
    );

    return () => {
      mounted = false;
      unlistenStt.then((f) => f());
      unlistenTranslation.then((f) => f());
    };
  }, []);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [captions.length, pending]);

  const handleClose = async () => {
    const win = getCurrentWindow();
    await win.close();
  };

  return (
    <div className="flex h-full flex-col bg-black/75 text-white">
      {/* Drag region + close button */}
      <div
        data-tauri-drag-region
        className="flex shrink-0 items-center justify-between px-3 py-1"
      >
        <span
          data-tauri-drag-region
          className="text-[10px] uppercase tracking-wider text-white/50"
        >
          Captions
        </span>
        <button
          onClick={handleClose}
          className="rounded px-1.5 text-xs text-white/50 hover:bg-white/10 hover:text-white"
        >
          ✕
        </button>
      </div>

      {/* Caption list */}
      <div className="flex-1 space-y-1.5 overflow-y-auto px-3 pb-2">
        {captions.length === 0 && (
          <p className="text-center text-xs italic text-white/40">
            Waiting for captions...
          </p>
        )}
        {captions.map((c) => {
          const allLangs = new Set([
            ...Object.keys(c.translations),
            ...Object.keys(pending[c.id] || {}),
          ]);
          return (
            <div key={c.id} className="space-y-0.5">
              <p className="text-sm leading-snug text-white/70">{c.text}</p>
              {[...allLangs].map((lang) => {
                const finalText = c.translations[lang];
                const pendingText = pending[c.id]?.[lang];
                const display = finalText || pendingText;
                if (!display) return null;
                return (
                  <p key={lang} className="text-sm font-medium leading-snug text-white">
                    <span className="mr-1 text-[10px] uppercase text-white/50">
                      {lang}
                    </span>
                    {finalText ? (
                      display
                    ) : (
                      <span className="italic text-white/80">{display}...</span>
                    )}
                  </p>
                );
              })}
            </div>
          );
        })}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
