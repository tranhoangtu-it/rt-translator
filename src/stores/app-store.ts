import { create } from "zustand";
import type {
  AudioState,
  DeviceInfo,
  OverlaySettings,
  TranscriptEntry,
  NoteRecord,
} from "@/types";

interface SttSlice {
  transcript: TranscriptEntry[];
  currentCaption: string | null;
  isTranscribing: boolean;
}

interface TranslationSlice {
  /** Nested map: segment_id -> target_lang -> translated text */
  translations: Record<string, Record<string, string>>;
  /** Currently streaming: segment_id -> target_lang -> partial text */
  pendingTranslation: Record<string, Record<string, string>>;
  targetLangs: string[];
  isTranslating: boolean;
}

interface AppState {
  initialized: boolean;
  currentView: "home" | "meeting" | "settings";
  setInitialized: (value: boolean) => void;
  setView: (view: AppState["currentView"]) => void;

  // Audio state
  audio: AudioState;
  setAudioDevices: (devices: DeviceInfo[]) => void;
  setSelectedDevice: (deviceId: string | null) => void;
  setCapturing: (isCapturing: boolean) => void;
  setAudioError: (error: string | null) => void;

  // STT state
  stt: SttSlice;
  addTranscriptEntry: (entry: TranscriptEntry) => void;
  clearTranscript: () => void;
  setTranscribing: (value: boolean) => void;
  setCurrentCaption: (text: string | null) => void;

  // Translation state
  translation: TranslationSlice;
  setTranslation: (segmentId: string, targetLang: string, text: string, isFinal: boolean) => void;
  clearTranslations: () => void;
  setTargetLangs: (langs: string[]) => void;
  toggleTargetLang: (lang: string) => void;
  setIsTranslating: (value: boolean) => void;

  // Meeting + overlay state
  meetingId: number | null;
  setMeetingId: (id: number | null) => void;
  overlay: OverlaySettings;
  setOverlayOpen: (open: boolean) => void;
  setOverlayFontSize: (size: number) => void;

  // Notes state
  notes: NoteRecord[];
  isGeneratingNotes: boolean;
  addNotes: (newNotes: NoteRecord[]) => void;
  setNotes: (notes: NoteRecord[]) => void;
  updateNoteContent: (noteId: number, content: string) => void;
  removeNote: (noteId: number) => void;
  clearNotes: () => void;
  setGeneratingNotes: (val: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  initialized: false,
  currentView: "home",
  setInitialized: (value) => set({ initialized: value }),
  setView: (view) => set({ currentView: view }),

  audio: {
    devices: [],
    selectedDeviceId: null,
    isCapturing: false,
    error: null,
  },
  setAudioDevices: (devices) =>
    set((state) => ({ audio: { ...state.audio, devices } })),
  setSelectedDevice: (deviceId) =>
    set((state) => ({
      audio: { ...state.audio, selectedDeviceId: deviceId },
    })),
  setCapturing: (isCapturing) =>
    set((state) => ({ audio: { ...state.audio, isCapturing } })),
  setAudioError: (error) =>
    set((state) => ({ audio: { ...state.audio, error } })),

  stt: {
    transcript: [],
    currentCaption: null,
    isTranscribing: false,
  },
  addTranscriptEntry: (entry) =>
    set((state) => ({
      stt: { ...state.stt, transcript: [...state.stt.transcript, entry] },
    })),
  clearTranscript: () =>
    set((state) => ({
      stt: { ...state.stt, transcript: [], currentCaption: null },
    })),
  setTranscribing: (value) =>
    set((state) => ({ stt: { ...state.stt, isTranscribing: value } })),
  setCurrentCaption: (text) =>
    set((state) => ({ stt: { ...state.stt, currentCaption: text } })),

  // Translation state (multi-target)
  translation: {
    translations: {},
    pendingTranslation: {},
    targetLangs: ["vi"],
    isTranslating: false,
  },
  setTranslation: (segmentId, targetLang, text, isFinal) =>
    set((state) => {
      if (isFinal) {
        const segPending = { ...(state.translation.pendingTranslation[segmentId] || {}) };
        delete segPending[targetLang];
        const pendingTranslation = { ...state.translation.pendingTranslation };
        if (Object.keys(segPending).length === 0) {
          delete pendingTranslation[segmentId];
        } else {
          pendingTranslation[segmentId] = segPending;
        }
        return {
          translation: {
            ...state.translation,
            translations: {
              ...state.translation.translations,
              [segmentId]: {
                ...(state.translation.translations[segmentId] || {}),
                [targetLang]: text,
              },
            },
            pendingTranslation,
          },
        };
      }
      return {
        translation: {
          ...state.translation,
          pendingTranslation: {
            ...state.translation.pendingTranslation,
            [segmentId]: {
              ...(state.translation.pendingTranslation[segmentId] || {}),
              [targetLang]: text,
            },
          },
        },
      };
    }),
  clearTranslations: () =>
    set((state) => ({
      translation: {
        ...state.translation,
        translations: {},
        pendingTranslation: {},
      },
    })),
  setTargetLangs: (langs) =>
    set((state) => ({
      translation: { ...state.translation, targetLangs: langs },
    })),
  toggleTargetLang: (lang) =>
    set((state) => {
      const current = state.translation.targetLangs;
      const next = current.includes(lang)
        ? current.filter((l) => l !== lang)
        : [...current, lang].slice(0, 4); // cap at 4
      return {
        translation: { ...state.translation, targetLangs: next.length > 0 ? next : current },
      };
    }),
  setIsTranslating: (value) =>
    set((state) => ({
      translation: { ...state.translation, isTranslating: value },
    })),

  // Meeting + overlay state
  meetingId: null,
  setMeetingId: (id) => set({ meetingId: id }),
  overlay: { isOpen: false, fontSize: 18, maxCaptions: 4 },
  setOverlayOpen: (open) =>
    set((state) => ({ overlay: { ...state.overlay, isOpen: open } })),
  setOverlayFontSize: (size) =>
    set((state) => ({ overlay: { ...state.overlay, fontSize: size } })),

  // Notes state
  notes: [],
  isGeneratingNotes: false,
  addNotes: (newNotes) =>
    set((state) => ({ notes: [...state.notes, ...newNotes] })),
  setNotes: (notes) => set({ notes }),
  updateNoteContent: (noteId, content) =>
    set((state) => ({
      notes: state.notes.map((note) =>
        note.id === noteId ? { ...note, content } : note
      ),
    })),
  removeNote: (noteId) =>
    set((state) => ({
      notes: state.notes.filter((note) => note.id !== noteId),
    })),
  clearNotes: () => set({ notes: [] }),
  setGeneratingNotes: (val) => set({ isGeneratingNotes: val }),
}));
