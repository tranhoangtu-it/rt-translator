import { useTauriEvent } from "./use-tauri-events";
import { useAppStore } from "@/stores/app-store";
import type { NotesUpdatedPayload, NoteRecord } from "@/types";

interface NotesErrorPayload {
  meeting_id: number;
  error: string;
}

/**
 * Listens for notes-updated and notes-error events from backend
 * and updates the Zustand store accordingly.
 */
export function useNoteEvents() {
  const addNotes = useAppStore((s) => s.addNotes);
  const setGeneratingNotes = useAppStore((s) => s.setGeneratingNotes);

  useTauriEvent<NotesUpdatedPayload>("notes-updated", (payload) => {
    const newNotes: NoteRecord[] = [];
    const ids = payload.inserted_ids ?? [];
    let idIdx = 0;

    payload.new_notes.key_points.forEach((kp) => {
      newNotes.push({
        id: ids[idIdx++] ?? -Date.now() - idIdx,
        meeting_id: payload.meeting_id,
        note_type: "key_point",
        content: JSON.stringify(kp),
        created_at: new Date().toISOString(),
      });
    });

    payload.new_notes.decisions.forEach((dec) => {
      newNotes.push({
        id: ids[idIdx++] ?? -Date.now() - idIdx,
        meeting_id: payload.meeting_id,
        note_type: "decision",
        content: JSON.stringify(dec),
        created_at: new Date().toISOString(),
      });
    });

    payload.new_notes.action_items.forEach((action) => {
      newNotes.push({
        id: ids[idIdx++] ?? -Date.now() - idIdx,
        meeting_id: payload.meeting_id,
        note_type: "action_item",
        content: JSON.stringify(action),
        created_at: new Date().toISOString(),
      });
    });

    payload.new_notes.risks.forEach((risk) => {
      newNotes.push({
        id: ids[idIdx++] ?? -Date.now() - idIdx,
        meeting_id: payload.meeting_id,
        note_type: "risk",
        content: JSON.stringify(risk),
        created_at: new Date().toISOString(),
      });
    });

    if (newNotes.length > 0) {
      addNotes(newNotes);
      setGeneratingNotes(false);
    }
  });

  // Fix #6: Listen for note generation errors
  useTauriEvent<NotesErrorPayload>("notes-error", (payload) => {
    console.warn("Note generation error:", payload.error);
    setGeneratingNotes(false);
  });
}
