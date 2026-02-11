import { useEffect, useRef, useState } from "react";
import { useAppStore } from "@/stores/app-store";
import { Copy, MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";

const LANG_LABELS: Record<string, string> = {
  vi: "VI", en: "EN", ja: "JA", ko: "KO",
  zh: "ZH", fr: "FR", de: "DE", es: "ES",
};

/**
 * Transcript timeline with tabbed multi-language translation display.
 * Shows original text + translations for the selected target language tab.
 */
export function TranscriptTimeline() {
  const transcript = useAppStore((s) => s.stt.transcript);
  const currentCaption = useAppStore((s) => s.stt.currentCaption);
  const translations = useAppStore((s) => s.translation.translations);
  const pendingTranslation = useAppStore(
    (s) => s.translation.pendingTranslation,
  );
  const targetLangs = useAppStore((s) => s.translation.targetLangs);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [activeLang, setActiveLang] = useState(targetLangs[0] || "vi");

  // Keep activeLang in sync when targetLangs changes
  useEffect(() => {
    if (!targetLangs.includes(activeLang) && targetLangs.length > 0) {
      setActiveLang(targetLangs[0]);
    }
  }, [targetLangs, activeLang]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcript.length, currentCaption]);

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success("Đã sao chép");
    } catch (err) {
      toast.error("Lỗi sao chép");
    }
  };

  return (
    <div className="flex h-96 flex-col rounded-md border bg-muted/30 p-3">
      <div className="mb-2 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-muted-foreground">
          Transcript Timeline
        </h3>

        {/* Language tabs */}
        {targetLangs.length > 1 && (
          <div className="flex gap-1">
            {targetLangs.map((lang) => (
              <button
                key={lang}
                onClick={() => setActiveLang(lang)}
                className={`rounded px-2 py-0.5 text-xs font-medium transition-colors ${
                  activeLang === lang
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted text-muted-foreground hover:bg-muted/80"
                }`}
              >
                {LANG_LABELS[lang] || lang.toUpperCase()}
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto text-sm">
        {transcript.length === 0 && !currentCaption && (
          <div className="flex flex-col items-center justify-center gap-2 py-8 text-center">
            <MessageSquare className="h-8 w-8 text-muted-foreground/50" />
            <p className="text-muted-foreground italic">
              Chưa có nội dung. Bắt đầu cuộc họp để ghi nhận.
            </p>
          </div>
        )}

        {/* Header row */}
        {transcript.length > 0 && (
          <div className="mb-2 grid grid-cols-[60px_1fr_1fr_32px] gap-2 border-b pb-1 text-xs font-medium text-muted-foreground">
            <span>Time</span>
            <span>Original</span>
            <span>
              Translation ({LANG_LABELS[activeLang] || activeLang.toUpperCase()})
            </span>
            <span></span>
          </div>
        )}

        {/* Transcript rows */}
        {transcript.map((entry) => {
          const segTranslations = translations[entry.id];
          const segPending = pendingTranslation[entry.id];
          const finalText = segTranslations?.[activeLang];
          const pendingText = segPending?.[activeLang];
          return (
            <div
              key={entry.id}
              className="group grid grid-cols-[60px_1fr_1fr_32px] gap-2 border-b border-border/30 py-1.5"
            >
              <span className="text-xs text-muted-foreground">
                {formatMs(entry.startMs)}
              </span>
              <p className="leading-snug">{entry.text}</p>
              <p className="leading-snug text-muted-foreground">
                {finalText || (
                  <span className="italic">
                    {pendingText ? `${pendingText}...` : "—"}
                  </span>
                )}
              </p>
              <Button
                size="sm"
                variant="ghost"
                className="cursor-pointer opacity-0 transition-opacity group-hover:opacity-100"
                onClick={() =>
                  copyToClipboard(
                    `${entry.text}${finalText ? `\n${finalText}` : ""}`
                  )
                }
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
          );
        })}

        {/* Live caption */}
        {currentCaption && (
          <div className="grid grid-cols-[60px_1fr_1fr_32px] gap-2 py-1.5">
            <span className="text-xs text-muted-foreground">
              <Badge variant="secondary" className="animate-pulse text-[10px]">
                LIVE
              </Badge>
            </span>
            <p className="italic text-muted-foreground">{currentCaption}</p>
            <p />
            <div />
          </div>
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
