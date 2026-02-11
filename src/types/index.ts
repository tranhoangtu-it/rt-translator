export type MeetingStatus = "idle" | "recording" | "paused" | "stopped";

export type Language = string;

export type NoteType = "key_point" | "decision" | "risk" | "action_item";

export interface Caption {
  id: string;
  text: string;
  translatedText?: string;
  lang: Language;
  timestamp: number;
  isFinal: boolean;
}

export interface Meeting {
  id: string;
  title: string;
  startedAt: string;
  endedAt?: string;
  sourceLang: Language;
  targetLangs: Language[];
  status: MeetingStatus;
}

export interface DeviceInfo {
  id: string;
  name: string;
  is_input: boolean;
  is_loopback: boolean;
  sample_rate: number;
  channels: number;
}

export interface AudioState {
  devices: DeviceInfo[];
  selectedDeviceId: string | null;
  isCapturing: boolean;
  error: string | null;
}

export interface SttEventPayload {
  text: string;
  language: string;
  start_ms: number;
  end_ms: number;
  is_final: boolean;
  segment_id: string;
}

export interface TranscriptEntry {
  id: string;
  text: string;
  language: string;
  startMs: number;
  endMs: number;
  timestamp: Date;
}

export interface ModelStatus {
  available: boolean;
  model_name: string;
  file_size_mb: number;
}

export interface TranslationUpdatePayload {
  segment_id: string;
  text: string;
  target_lang: string;
  is_final: boolean;
}

export interface TranslationErrorPayload {
  segment_id: string;
  error: string;
}

export interface OllamaModelInfo {
  name: string;
  size: number;
  digest?: string;
  details?: {
    parameter_size?: string;
    quantization_level?: string;
    family?: string;
  };
}

export type ExportFormat = "txt" | "md" | "json";

export interface OverlaySettings {
  isOpen: boolean;
  fontSize: number;
  maxCaptions: number;
}

export * from "./notes";
