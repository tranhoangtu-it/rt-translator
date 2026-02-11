import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppLayout } from "@/components/app-layout";
import { AudioDeviceSelector } from "@/components/audio-device-selector";
import { TranscriptTimeline } from "@/components/transcript-timeline";
import { SettingsPanel } from "@/components/settings-panel";
import { ExportButton } from "@/components/export-button";
import { NotesPanel } from "@/components/notes-panel";
import { MemoExportButton } from "@/components/memo-export-button";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";
import { useSttEvents } from "@/hooks/use-stt-events";
import {
  useTranslationEvents,
  useAutoTranslation,
} from "@/hooks/use-translation-events";
import { useNoteEvents } from "@/hooks/use-note-events";
import { useAppStore } from "@/stores/app-store";

function App() {
  // Listen for STT, translation, and note events at app level
  useSttEvents();
  useTranslationEvents();
  useAutoTranslation();
  useNoteEvents();

  const isCapturing = useAppStore((s) => s.audio.isCapturing);
  const isTranscribing = useAppStore((s) => s.stt.isTranscribing);
  const setTranscribing = useAppStore((s) => s.setTranscribing);
  const setIsTranslating = useAppStore((s) => s.setIsTranslating);
  const clearTranscript = useAppStore((s) => s.clearTranscript);
  const clearTranslations = useAppStore((s) => s.clearTranslations);
  const clearNotes = useAppStore((s) => s.clearNotes);
  const setMeetingId = useAppStore((s) => s.setMeetingId);
  const meetingId = useAppStore((s) => s.meetingId);
  const [meetingError, setMeetingError] = useState<string | null>(null);
  const [isStarting, setIsStarting] = useState(false);
  const [isStopping, setIsStopping] = useState(false);

  const handleStartMeeting = async () => {
    try {
      setIsStarting(true);
      setMeetingError(null);
      clearTranscript();
      clearTranslations();
      clearNotes();
      const targetLangs = useAppStore.getState().translation.targetLangs;
      const result = await invoke<string>("start_meeting", {
        srcLang: "en",
        targetLangs,
      });
      const dbId = parseInt(result, 10);
      setMeetingId(isNaN(dbId) ? null : dbId);
      setTranscribing(true);
      setIsTranslating(true);
    } catch (err) {
      setMeetingError(String(err));
    } finally {
      setIsStarting(false);
    }
  };

  const handleStopMeeting = async () => {
    try {
      setIsStopping(true);
      setMeetingError(null);
      await invoke("stop_meeting");
      setTranscribing(false);
      setIsTranslating(false);
      // Keep meetingId for export — cleared on next start
    } catch (err) {
      setMeetingError(String(err));
    } finally {
      setIsStopping(false);
    }
  };

  return (
    <AppLayout>
      <div className="flex flex-col items-center justify-center gap-4 pt-10">
        <h2 className="text-3xl font-bold">Real-Time Meeting Translator</h2>
        <p className="text-muted-foreground">
          Translate meetings in real-time with AI
        </p>

        <div className="mt-4 w-full max-w-2xl space-y-4">
          <SettingsPanel />
          <AudioDeviceSelector />

          {/* Meeting controls */}
          {isTranscribing ? (
            <Button
              variant="destructive"
              size="lg"
              className="w-full cursor-pointer"
              onClick={handleStopMeeting}
              disabled={isStopping}
            >
              {isStopping && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {isStopping ? "Stopping..." : "Stop Meeting"}
            </Button>
          ) : (
            <Button
              variant="default"
              size="lg"
              className="w-full cursor-pointer"
              disabled={!isCapturing || isStarting}
              onClick={handleStartMeeting}
            >
              {isStarting && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {isStarting ? "Starting..." : "Start Meeting"}
            </Button>
          )}

          {meetingError && (
            <p className="text-sm text-destructive">{meetingError}</p>
          )}

          {/* Transcript timeline */}
          {(isTranscribing || meetingId) && <TranscriptTimeline />}

          {/* Notes panel */}
          {(isTranscribing || meetingId) && <NotesPanel />}

          {/* Export buttons (visible after meeting stopped) */}
          {!isTranscribing && meetingId && (
            <div className="flex flex-wrap gap-2">
              <ExportButton />
              <MemoExportButton />
            </div>
          )}
        </div>
      </div>
    </AppLayout>
  );
}

export default App;
