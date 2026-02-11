import { invoke } from "@tauri-apps/api/core";
import { useTauriEvent } from "./use-tauri-events";
import { useAppStore } from "@/stores/app-store";
import type { TranslationUpdatePayload, SttEventPayload } from "@/types";

/**
 * Listens for translation-update events from backend
 * and updates the Zustand store with target_lang demuxing.
 */
export function useTranslationEvents() {
  const setTranslation = useAppStore((s) => s.setTranslation);

  useTauriEvent<TranslationUpdatePayload>("translation-update", (payload) => {
    setTranslation(payload.segment_id, payload.target_lang, payload.text, payload.is_final);
  });
}

/**
 * Auto-translates final STT segments by invoking translate_text command.
 * Sends all targetLangs in a single invoke (backend fans out in parallel).
 * Uses getState() to avoid stale closure over targetLangs/isTranslating.
 */
export function useAutoTranslation() {
  useTauriEvent<SttEventPayload>("stt-partial", (payload) => {
    const { isTranslating, targetLangs } =
      useAppStore.getState().translation;
    if (!isTranslating || !payload.is_final || !payload.text.trim()) return;
    if (targetLangs.length === 0) return;

    const segmentId = payload.segment_id;
    invoke("translate_text", {
      text: payload.text.trim(),
      targetLangs,
      segmentId,
    }).catch((err) => {
      console.error("Translation invoke failed:", err);
    });
  });
}
