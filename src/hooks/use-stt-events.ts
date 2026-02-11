import { useTauriEvent } from "./use-tauri-events";
import { useAppStore } from "@/stores/app-store";
import type { SttEventPayload, TranscriptEntry } from "@/types";

/**
 * Listens for stt-partial events from the Rust backend
 * and updates the Zustand store with transcript entries.
 */
export function useSttEvents() {
  const addTranscriptEntry = useAppStore((s) => s.addTranscriptEntry);
  const setCurrentCaption = useAppStore((s) => s.setCurrentCaption);

  useTauriEvent<SttEventPayload>("stt-partial", (payload) => {
    if (payload.is_final) {
      const entry: TranscriptEntry = {
        id: payload.segment_id,
        text: payload.text.trim(),
        language: payload.language,
        startMs: payload.start_ms,
        endMs: payload.end_ms,
        timestamp: new Date(),
      };
      addTranscriptEntry(entry);
      setCurrentCaption(null);
    } else {
      setCurrentCaption(payload.text);
    }
  });
}
