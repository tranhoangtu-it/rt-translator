import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "@/stores/app-store";
import { Slider } from "@/components/ui/slider";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";

const LANGUAGES = [
  { value: "vi", label: "Vietnamese", fullName: "Tiếng Việt" },
  { value: "en", label: "English", fullName: "English" },
  { value: "ja", label: "Japanese", fullName: "日本語" },
  { value: "ko", label: "Korean", fullName: "한국어" },
  { value: "zh", label: "Chinese", fullName: "中文" },
  { value: "fr", label: "French", fullName: "Français" },
  { value: "de", label: "German", fullName: "Deutsch" },
  { value: "es", label: "Spanish", fullName: "Español" },
];

export function SettingsPanel() {
  const targetLangs = useAppStore((s) => s.translation.targetLangs);
  const toggleTargetLang = useAppStore((s) => s.toggleTargetLang);
  const overlay = useAppStore((s) => s.overlay);
  const setOverlayOpen = useAppStore((s) => s.setOverlayOpen);
  const setOverlayFontSize = useAppStore((s) => s.setOverlayFontSize);

  const handleOverlayToggle = async () => {
    try {
      if (overlay.isOpen) {
        await invoke("close_overlay_window");
        setOverlayOpen(false);
      } else {
        await invoke("open_overlay_window");
        setOverlayOpen(true);
      }
    } catch (err) {
      console.error("Overlay toggle failed:", err);
    }
  };

  return (
    <div className="space-y-3 rounded-md border bg-muted/20 p-3">
      <h3 className="text-sm font-semibold text-muted-foreground">Settings</h3>

      {/* Target languages (multi-select, max 4) */}
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">
          Target Languages ({targetLangs.length}/4)
        </label>
        <div className="flex flex-wrap gap-2">
          {LANGUAGES.map((l) => {
            const active = targetLangs.includes(l.value);
            const isDisabled = !active && targetLangs.length >= 4;
            return (
              <Tooltip key={l.value}>
                <TooltipTrigger asChild>
                  <button
                    onClick={() => toggleTargetLang(l.value)}
                    disabled={isDisabled}
                    className={`cursor-pointer rounded-full border px-2.5 py-0.5 text-xs transition-colors ${
                      active
                        ? "border-primary bg-primary text-primary-foreground"
                        : isDisabled
                          ? "cursor-not-allowed border-border bg-muted/50 text-muted-foreground/50"
                          : "border-border bg-background text-muted-foreground hover:bg-muted"
                    }`}
                  >
                    {l.label}
                  </button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{l.fullName}</p>
                </TooltipContent>
              </Tooltip>
            );
          })}
        </div>
      </div>

      {/* Font size */}
      <div className="flex items-center gap-2">
        <label className="w-28 text-xs text-muted-foreground">
          Font Size ({overlay.fontSize}px)
        </label>
        <Slider
          value={[overlay.fontSize]}
          onValueChange={(value) => setOverlayFontSize(value[0])}
          min={14}
          max={32}
          step={2}
          className="flex-1"
        />
      </div>

      {/* Overlay toggle */}
      <div className="flex items-center gap-2">
        <label className="w-28 text-xs text-muted-foreground">
          Caption Overlay
        </label>
        <Button
          onClick={handleOverlayToggle}
          variant={overlay.isOpen ? "outline" : "default"}
          size="sm"
          className="cursor-pointer"
        >
          {overlay.isOpen ? "Close Overlay" : "Open Overlay"}
        </Button>
      </div>
    </div>
  );
}
