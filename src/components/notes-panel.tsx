import { useState } from "react";
import { useAppStore } from "@/stores/app-store";
import { NoteCard } from "@/components/note-card";

const CATEGORIES = [
  { value: "key_point" as const, label: "Key Points" },
  { value: "decision" as const, label: "Decisions" },
  { value: "action_item" as const, label: "Action Items" },
  { value: "risk" as const, label: "Risks" },
];

export function NotesPanel() {
  const notes = useAppStore((s) => s.notes);
  const isGeneratingNotes = useAppStore((s) => s.isGeneratingNotes);
  const updateNoteContent = useAppStore((s) => s.updateNoteContent);
  const removeNote = useAppStore((s) => s.removeNote);
  const [activeTab, setActiveTab] = useState<
    "key_point" | "decision" | "action_item" | "risk"
  >("key_point");

  const filteredNotes = notes.filter((n) => n.note_type === activeTab);

  return (
    <div className="flex h-96 flex-col rounded-md border bg-muted/30 p-3">
      <div className="mb-2 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-muted-foreground">
          Meeting Notes (AI)
        </h3>
        {isGeneratingNotes && (
          <span className="text-xs italic text-muted-foreground">
            Generating notes...
          </span>
        )}
      </div>

      {/* Tab bar */}
      <div className="mb-2 flex gap-1">
        {CATEGORIES.map((cat) => {
          const count = notes.filter((n) => n.note_type === cat.value).length;
          return (
            <button
              key={cat.value}
              onClick={() => setActiveTab(cat.value)}
              className={`cursor-pointer rounded px-2 py-0.5 text-xs font-medium transition-colors ${
                activeTab === cat.value
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted text-muted-foreground hover:bg-muted/80"
              }`}
            >
              {cat.label} ({count})
            </button>
          );
        })}
      </div>

      {/* Note list */}
      <div className="flex-1 overflow-y-auto text-sm">
        {filteredNotes.length === 0 ? (
          <p className="text-center italic text-muted-foreground">
            {isGeneratingNotes
              ? "Waiting for AI to extract notes..."
              : "No notes yet. Notes will appear as the meeting progresses."}
          </p>
        ) : (
          <div>
            {filteredNotes.map((note) => (
              <NoteCard
                key={note.id}
                note={note}
                onUpdate={updateNoteContent}
                onDelete={removeNote}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
