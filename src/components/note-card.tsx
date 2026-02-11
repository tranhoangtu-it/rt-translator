import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Trash2, Pencil, Check, X } from "lucide-react";
import { toast } from "sonner";
import type {
  NoteRecord,
  KeyPoint,
  Decision,
  ActionItem,
  Risk,
} from "@/types";

interface NoteCardProps {
  note: NoteRecord;
  onUpdate: (noteId: number, content: string) => void;
  onDelete: (noteId: number) => void;
}

export function NoteCard({ note, onUpdate, onDelete }: NoteCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(note.content);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  let parsed: KeyPoint | Decision | ActionItem | Risk;
  try {
    parsed = JSON.parse(note.content);
  } catch {
    return (
      <div className="py-2 text-xs text-destructive">Invalid note data</div>
    );
  }

  const handleSave = async () => {
    try {
      await invoke("update_note", { noteId: note.id, content: editContent });
      onUpdate(note.id, editContent);
      setIsEditing(false);
      toast.success("Đã cập nhật ghi chú");
    } catch (err) {
      toast.error(`Lỗi cập nhật: ${err}`);
    }
  };

  const handleDelete = async () => {
    try {
      await invoke("delete_note", { noteId: note.id });
      onDelete(note.id);
      setShowDeleteDialog(false);
      toast.success("Đã xóa ghi chú");
    } catch (err) {
      toast.error(`Lỗi xóa: ${err}`);
    }
  };

  if (isEditing) {
    return (
      <div className="space-y-2 border-b border-border/30 py-2">
        <textarea
          value={editContent}
          onChange={(e) => setEditContent(e.target.value)}
          rows={3}
          className="w-full rounded border border-border bg-background px-2 py-1 text-xs"
        />
        <div className="flex gap-2">
          <Button
            size="sm"
            onClick={handleSave}
            className="cursor-pointer"
          >
            <Check className="h-3 w-3" />
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              setIsEditing(false);
              setEditContent(note.content);
            }}
            className="cursor-pointer"
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="group flex justify-between gap-2 border-b border-border/30 py-2 transition-colors hover:bg-muted/20">
        <div className="flex-1 text-sm">
          {note.note_type === "key_point" && (
            <div>
              <span className="font-semibold">{(parsed as KeyPoint).topic}:</span>{" "}
              {(parsed as KeyPoint).summary}
              <span className="ml-2 text-xs text-muted-foreground">
                {(parsed as KeyPoint).timestamp}
              </span>
            </div>
          )}
          {note.note_type === "decision" && (
            <div>
              <span className="font-semibold">Decision:</span>{" "}
              {(parsed as Decision).decision}
              {(parsed as Decision).rationale && (
                <span className="text-muted-foreground">
                  {" "}
                  — {(parsed as Decision).rationale}
                </span>
              )}
              <span className="ml-2 text-xs text-muted-foreground">
                {(parsed as Decision).timestamp}
              </span>
            </div>
          )}
          {note.note_type === "action_item" && (
            <div>
              <span className="font-semibold">Task:</span>{" "}
              {(parsed as ActionItem).task}
              {(parsed as ActionItem).owner && (
                <span className="text-muted-foreground">
                  {" "}
                  → {(parsed as ActionItem).owner}
                </span>
              )}
              {(parsed as ActionItem).deadline && (
                <span className="text-muted-foreground">
                  {" "}
                  (by {(parsed as ActionItem).deadline})
                </span>
              )}
              {(parsed as ActionItem).priority && (
                <span
                  className={`ml-2 rounded px-1.5 py-0.5 text-xs ${
                    (parsed as ActionItem).priority === "high"
                      ? "bg-destructive/20 text-destructive"
                      : (parsed as ActionItem).priority === "medium"
                        ? "bg-yellow-500/20 text-yellow-700"
                        : "bg-green-500/20 text-green-700"
                  }`}
                >
                  {(parsed as ActionItem).priority}
                </span>
              )}
            </div>
          )}
          {note.note_type === "risk" && (
            <div>
              <span className="font-semibold text-destructive">Risk:</span>{" "}
              {(parsed as Risk).risk}
              {(parsed as Risk).impact && (
                <div className="mt-1 text-xs text-muted-foreground">
                  Impact: {(parsed as Risk).impact}
                </div>
              )}
              {(parsed as Risk).mitigation && (
                <div className="text-xs text-muted-foreground">
                  Mitigation: {(parsed as Risk).mitigation}
                </div>
              )}
              <span className="ml-2 text-xs text-muted-foreground">
                {(parsed as Risk).timestamp}
              </span>
            </div>
          )}
        </div>
        <div className="flex gap-1">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              setIsEditing(true);
              setEditContent(note.content);
            }}
            className="cursor-pointer"
          >
            <Pencil className="h-3 w-3" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setShowDeleteDialog(true)}
            className="cursor-pointer"
          >
            <Trash2 className="h-3 w-3 text-destructive" />
          </Button>
        </div>
      </div>

      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Xóa ghi chú này?</AlertDialogTitle>
            <AlertDialogDescription>
              Hành động này không thể hoàn tác. Ghi chú sẽ bị xóa vĩnh viễn.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Hủy</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>Xác nhận</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
