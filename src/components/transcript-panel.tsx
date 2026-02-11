import { useEffect, useRef } from "react";
import { useAppStore } from "@/stores/app-store";

/**
 * Scrollable panel that displays transcript segments
 * with translations and a live caption at the bottom.
 */
export function TranscriptPanel() {
  const transcript = useAppStore((s) => s.stt.transcript);
  const currentCaption = useAppStore((s) => s.stt.currentCaption);
  const translations = useAppStore((s) => s.translation.translations);
  const pendingTranslation = useAppStore(
    (s) => s.translation.pendingTranslation,
  );
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcript.length, currentCaption]);

  return (
    <div className="flex h-80 flex-col rounded-md border bg-muted/30 p-3">
      <h3 className="mb-2 text-sm font-semibold text-muted-foreground">
        Transcript
      </h3>
      <div className="flex-1 space-y-2 overflow-y-auto text-sm">
        {transcript.length === 0 && !currentCaption && (
          <p className="text-muted-foreground italic">
            Waiting for speech...
          </p>
        )}
        {transcript.map((entry) => {
          const segTranslations = translations[entry.id] || {};
          const segPending = pendingTranslation[entry.id] || {};
          const firstLang = Object.keys(segTranslations)[0] || Object.keys(segPending)[0];
          const finalText = firstLang ? segTranslations[firstLang] : undefined;
          const pendingText = firstLang ? segPending[firstLang] : undefined;
          return (
            <div key={entry.id} className="space-y-0.5">
              <p>
                <span className="mr-2 text-xs text-muted-foreground">
                  [{formatMs(entry.startMs)}]
                </span>
                {entry.text}
              </p>
              {(finalText || pendingText) && (
                <p className="ml-6 text-muted-foreground">
                  {finalText || (
                    <span className="italic">{pendingText}...</span>
                  )}
                </p>
              )}
            </div>
          );
        })}
        {currentCaption && (
          <p className="text-muted-foreground italic">{currentCaption}</p>
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}

function formatMs(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
