import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/stores/app-store";
import { Button } from "@/components/ui/button";
import { FileText, FileCode, Loader2 } from "lucide-react";
import { toast } from "sonner";
import type { ExportFormat } from "@/types";

const FORMATS: {
  value: ExportFormat;
  label: string;
  ext: string;
  icon: React.ComponentType<{ className?: string }>;
}[] = [
  { value: "txt", label: "Plain Text", ext: "txt", icon: FileText },
  { value: "md", label: "Markdown", ext: "md", icon: FileCode },
  { value: "json", label: "JSON", ext: "json", icon: FileCode },
];

export function ExportButton() {
  const meetingId = useAppStore((s) => s.meetingId);
  const [exporting, setExporting] = useState<ExportFormat | null>(null);

  const handleExport = async (format: ExportFormat) => {
    if (!meetingId) return;
    setExporting(format);

    try {
      const path = await save({
        defaultPath: `meeting-transcript.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
      });
      if (!path) {
        setExporting(null);
        return;
      }

      await invoke("export_transcript", {
        meetingId,
        format,
        path,
      });

      toast.success(`Đã xuất file: ${path.split(/[\\/]/).pop()}`);
    } catch (err) {
      toast.error(`Lỗi xuất file: ${err}`);
    } finally {
      setExporting(null);
    }
  };

  if (!meetingId) return null;

  return (
    <div className="flex gap-2">
      {FORMATS.map((f) => {
        const Icon = f.icon;
        const isExporting = exporting === f.value;
        return (
          <Button
            key={f.value}
            variant="outline"
            size="sm"
            disabled={exporting !== null}
            onClick={() => handleExport(f.value)}
            className="cursor-pointer"
          >
            {isExporting ? (
              <Loader2 className="mr-2 h-3 w-3 animate-spin" />
            ) : (
              <Icon className="mr-2 h-3 w-3" />
            )}
            {isExporting ? "Exporting..." : `Export ${f.label}`}
          </Button>
        );
      })}
    </div>
  );
}
