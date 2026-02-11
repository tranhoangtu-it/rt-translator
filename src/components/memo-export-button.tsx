import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/stores/app-store";
import { Button } from "@/components/ui/button";
import { FileText, Loader2 } from "lucide-react";
import { toast } from "sonner";

export function MemoExportButton() {
  const meetingId = useAppStore((s) => s.meetingId);
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    if (!meetingId) return;
    setExporting(true);

    try {
      const path = await save({
        defaultPath: `meeting-memo-${meetingId}.md`,
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });

      if (!path) {
        setExporting(false);
        return;
      }

      await invoke("export_memo", {
        meetingId,
        filePath: path,
      });

      toast.success(`Đã xuất file: ${path.split(/[\\/]/).pop()}`);
    } catch (err) {
      toast.error(`Lỗi xuất file: ${err}`);
    } finally {
      setExporting(false);
    }
  };

  if (!meetingId) return null;

  return (
    <Button
      variant="outline"
      size="sm"
      disabled={exporting}
      onClick={handleExport}
      className="cursor-pointer"
    >
      {exporting ? (
        <Loader2 className="mr-2 h-3 w-3 animate-spin" />
      ) : (
        <FileText className="mr-2 h-3 w-3" />
      )}
      {exporting ? "Generating Memo..." : "Export Meeting Memo"}
    </Button>
  );
}
