# Tổng Quan Codebase

> Real-Time Multilingual Meeting Translator & Notetaker

## 1. Cấu Trúc Tổng Quan

```
rt-translator/
├── src-tauri/          # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── lib.rs      # Library entry point
│   │   ├── main.rs     # Tauri app entry point
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── audio.rs         # Audio capture Tauri commands ✅ NEW
│   │   │   ├── stt.rs           # STT Tauri commands
│   │   │   ├── translation.rs   # Translation Tauri commands ✅ SPRINT 4
│   │   │   ├── export.rs        # Transcript export commands ✅ SPRINT 5
│   │   │   ├── overlay.rs       # Overlay window commands ✅ SPRINT 5
│   │   │   ├── notes.rs         # Note management commands ✅ SPRINT 7
│   │   │   ├── meeting.rs       # Meeting control commands
│   │   │   └── settings.rs      # Settings management
│   │   ├── audio/
│   │   │   ├── mod.rs                # Module re-exports ✅ NEW
│   │   │   ├── types.rs              # Audio types (DeviceInfo, AudioConfig) ✅ NEW
│   │   │   ├── device.rs             # Device enumeration (cpal + WASAPI) ✅ NEW
│   │   │   ├── resampler.rs          # AudioResampler (rubato 1.0) ✅ NEW
│   │   │   ├── capture.rs            # AudioCaptureManager (dual-stream capture) ✅ NEW
│   │   │   └── [legacy] capture.rs   # cpal-based audio capture (skeleton)
│   │   ├── stt/
│   │   │   ├── mod.rs
│   │   │   └── whisper.rs    # whisper-rs wrapper
│   │   ├── translation/
│   │   │   ├── mod.rs
│   │   │   ├── pipeline.rs           # Translation orchestration ✅ SPRINT 4
│   │   │   └── translation_types.rs  # TranslationResult types ✅ SPRINT 4
│   │   ├── notes/               # Note engine ✅ SPRINT 7
│   │   │   ├── mod.rs
│   │   │   ├── note_engine.rs       # NoteEngine orchestrator ✅ SPRINT 7
│   │   │   ├── ollama_summarizer.rs # LLM summarizer ✅ SPRINT 7
│   │   │   ├── prompt_templates.rs  # Prompt engineering ✅ SPRINT 7
│   │   │   ├── note_types.rs        # Serde types (NoteCategory, Note) ✅ SPRINT 7
│   │   │   └── memo_builder.rs      # Memo formatter ✅ SPRINT 7
│   │   ├── providers/
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs             # Provider abstraction trait ✅ SPRINT 4
│   │   │   ├── ollama.rs             # Ollama HTTP client ✅ SPRINT 4
│   │   │   ├── ollama_types.rs       # Serde request/response types ✅ SPRINT 4
│   │   │   └── ollama_error.rs       # OllamaError enum ✅ SPRINT 4
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── models.rs          # Database models
│   │   │   ├── migrations.rs      # SQLite migrations (V1-V4) ✅ SPRINT 7
│   │   │   ├── transcript_store.rs # TranscriptDb ✅ SPRINT 5
│   │   │   └── note_store.rs      # NoteStore (CRUD operations) ✅ SPRINT 7
│   │   ├── auth/             # [Chưa implement]
│   │   └── security/         # [Chưa implement]
│   └── Cargo.toml
├── src/                # React frontend (Vite)
│   ├── main.tsx        # Main app entry point
│   ├── overlay/
│   │   ├── overlay.tsx                 # Overlay window entry point ✅ SPRINT 5
│   │   └── overlay.html                # Overlay HTML template ✅ SPRINT 5
│   ├── components/
│   │   ├── app-layout.tsx              # Main layout (Skeleton)
│   │   ├── audio-device-selector.tsx   # Device selector UI component ✅ NEW
│   │   ├── settings-panel.tsx          # Settings UI (language, font size, overlay) ✅ SPRINT 5
│   │   ├── transcript-timeline.tsx     # 3-column transcript display ✅ SPRINT 5
│   │   ├── export-button.tsx           # Export UI với native dialog ✅ SPRINT 5
│   │   ├── notes-panel.tsx             # Notes UI (4-tab, inline edit) ✅ SPRINT 7
│   │   ├── note-card.tsx               # Note card component (extracted) ✅ v0.7.1
│   │   ├── memo-export-button.tsx      # Memo export ✅ SPRINT 7
│   │   └── ui/
│   │       ├── button.tsx              # shadcn/ui button
│   │       ├── select.tsx              # shadcn/ui select ✅ v0.7.1
│   │       ├── slider.tsx              # shadcn/ui slider ✅ v0.7.1
│   │       ├── alert-dialog.tsx        # shadcn/ui alert dialog ✅ v0.7.1
│   │       ├── tooltip.tsx             # shadcn/ui tooltip ✅ v0.7.1
│   │       ├── badge.tsx               # shadcn/ui badge ✅ v0.7.1
│   │       ├── separator.tsx           # shadcn/ui separator ✅ v0.7.1
│   │       └── sonner.tsx              # Toast notifications ✅ v0.7.1
│   ├── hooks/
│   │   ├── use-tauri-events.ts         # Tauri event hook (Skeleton)
│   │   ├── use-audio-capture.ts        # Audio capture React hook ✅ NEW
│   │   └── use-note-events.ts          # Note event listener ✅ SPRINT 7
│   ├── stores/
│   │   └── app-store.ts                # zustand store (Skeleton)
│   └── types/                          # TypeScript definitions
│       └── notes.ts                    # Note types ✅ SPRINT 7
├── docs/               # Documentation
├── plans/              # Implementation plans
├── tailwind.config.ts  # Tailwind CSS v4 config
└── vite.config.ts      # Vite config
```

## 2. Module Index

| Module | Thư mục | Mô tả | Status |
|--------|---------|-------|--------|
| AudioCapture | `src-tauri/src/audio/` | Thu audio mic + system (cpal + WASAPI) | ✅ Complete |
| AudioResampler | `src-tauri/src/audio/` | Resampling 48→16kHz, stereo→mono (rubato) | ✅ Complete |
| Device Enumeration | `src-tauri/src/audio/` | Device listing (cpal + WASAPI) | ✅ Complete |
| VAD | `src-tauri/src/audio/vad.rs` | Energy-based VAD (RMS + ZCR, SpeechBuffer) | ✅ Complete |
| STT Engine | `src-tauri/src/stt/whisper.rs` | whisper-rs 0.15.1 wrapper (sync + async) | ✅ Complete |
| Model Manager | `src-tauri/src/stt/model_manager.rs` | HuggingFace CDN downloads, progress events | ✅ Complete |
| STT Pipeline | `src-tauri/src/stt/pipeline.rs` | Orchestrator (VAD→resampling→STT→events) | ✅ Complete |
| LLM Provider Trait | `src-tauri/src/providers/traits.rs` | RPITIT trait (chat, chat_streaming, health_check) | ✅ Sprint 4 |
| OllamaProvider | `src-tauri/src/providers/ollama.rs` | Native Ollama HTTP API integration | ✅ Sprint 4 |
| Translation Pipeline | `src-tauri/src/translation/` | Ollama text translation with streaming | ✅ Sprint 4 |
| Translation Types | `src-tauri/src/translation/translation_types.rs` | TranslationResult, event payloads | ✅ Sprint 4 |
| TranscriptDb | `src-tauri/src/storage/transcript_store.rs` | Thread-safe SQLite storage (Arc<Mutex<Connection>>, WAL mode) | ✅ Sprint 5 |
| TranslationRecord | `src-tauri/src/storage/models.rs` | Multi-lang translation model (transcript_id, target_lang, text) | ✅ Sprint 6 |
| Export Formatters | `src-tauri/src/commands/export.rs` | TXT/MD/JSON formatters cho transcript export (multi-lang support) | ✅ Sprint 5-6 |
| Overlay Commands | `src-tauri/src/commands/overlay.rs` | Show/hide caption overlay window (multi-lang support) | ✅ Sprint 5-6 |
| Note Engine | `src-tauri/src/notes/` | Incremental summarization (4 categories) | ✅ Sprint 7 |
| NoteStore | `src-tauri/src/storage/note_store.rs` | SQLite CRUD for notes table | ✅ Sprint 7 |
| Storage | `src-tauri/src/storage/` | SQLite models + migrations (V1-V4) | ✅ Sprint 7 |
| Auth | `src-tauri/src/auth/` | Email/password + Google OAuth | Chưa implement |
| Key Vault | `src-tauri/src/security/` | OS keychain (keyring crate) | Chưa implement |
| Audio Commands | `src-tauri/src/commands/audio.rs` | Tauri IPC commands + state | ✅ Complete |
| STT Commands | `src-tauri/src/commands/stt.rs` | Tauri commands (check/download model, start/stop meeting) | ✅ Complete |
| Translation Commands | `src-tauri/src/commands/translation.rs` | Tauri commands (translate_text with Vec<String>, multi-lang, JoinSet+Semaphore) | ✅ Sprint 4-6 |
| Note Commands | `src-tauri/src/commands/notes.rs` | 5 Tauri commands (get/update/delete notes, generate/export memo) | ✅ Sprint 7 |
| Command Handler | `src-tauri/src/commands/` | IPC commands (meeting, settings, export, overlay, notes) | ✅ Sprint 5-7 |
| AudioDeviceSelector | `src/components/audio-device-selector.tsx` | Device selector UI component | ✅ Complete |
| TranscriptPanel | `src/components/transcript-panel.tsx` | Transcript + translation display | ✅ Sprint 4 |
| SettingsPanel | `src/components/settings-panel.tsx` | Settings UI (multi-select target languages with pills, max 4, font size, overlay toggle) | ✅ Sprint 5-6 |
| TranscriptTimeline | `src/components/transcript-timeline.tsx` | Tabbed timeline display (switch between target languages) | ✅ Sprint 5-6 |
| ExportButton | `src/components/export-button.tsx` | Export UI với multi-lang support (TXT/MD/JSON with all translations) | ✅ Sprint 5-6 |
| Overlay Window | `src/overlay/overlay.tsx` | Caption overlay React app (multi-lang with language labels) | ✅ Sprint 5-6 |
| NotesPanel | `src/components/notes-panel.tsx` | Notes UI (4-tab, inline edit, modular) | ✅ Sprint 7 |
| NoteCard | `src/components/note-card.tsx` | Note card component (extracted 197 LOC) | ✅ v0.7.1 |
| MemoExportButton | `src/components/memo-export-button.tsx` | Memo export to .md file (icons + toast) | ✅ Sprint 7 |
| shadcn/ui Components | `src/components/ui/` | Select, Slider, AlertDialog, Tooltip, Badge, Separator, Sonner | ✅ v0.7.1 |
| useNoteEvents | `src/hooks/use-note-events.ts` | Note event listener | ✅ Sprint 7 |
| useAudioCapture | `src/hooks/use-audio-capture.ts` | Audio capture React hook | ✅ Complete |
| useSttEvents | `src/hooks/use-stt-events.ts` | STT event listener with race fix | ✅ Complete |
| useTranslationEvents | `src/hooks/use-translation-events.ts` | Translation event listener (demux by language) | ✅ Sprint 4-6 |
| useAutoTranslation | `src/hooks/use-auto-translation.ts` | Auto-translate with targetLangs array | ✅ Sprint 6 |
| Caption UI | `src/components/app-layout.tsx` | App main layout (header with recording indicator + dark mode toggle) | ✅ v0.7.1 |
| Tauri Event Hook | `src/hooks/use-tauri-events.ts` | Generic event handling bridge | ✅ Complete |
| App Store | `src/stores/app-store.ts` | zustand state management (nested Record<segId, Record<lang, text>> multi-lang structure) | ✅ Sprint 4-6 |
| Settings UI | `src/components/settings/` | App configuration UI | Chưa implement |

## 3. Key Entry Points

| Entry Point | File | Mô tả |
|-------------|------|-------|
| Tauri main | `src-tauri/src/main.rs` | Rust backend entry, registers commands |
| React app | `src/main.tsx` | Frontend entry, mounts React app |
| CLI | `src-tauri/src/main.rs` (clap) | Command-line interface |

## 4. Dependencies Chính

### Rust (Cargo.toml)
| Crate | Version | Vai trò |
|-------|---------|---------|
| tauri | 2.x | Desktop framework |
| tauri-plugin-sql | 2.x | SQLite via tauri plugin |
| tokio | 1.x | Async runtime |
| serde + serde_json | 1.x | Serialization/JSON |
| thiserror | 2.x | Error handling |
| anyhow | 1.x | Error context |
| tracing + tracing-subscriber | 0.1/0.3 | Structured logging |
| tauri-plugin-opener | 2.x | File/URL opener |
| whisper-rs | 0.15.1 | STT engine (whisper.cpp binding) |
| cpal | 0.17 | Audio capture (cross-platform) |
| wasapi | 0.22 | Windows loopback capture |
| rubato | 1.0 | Audio resampling |
| reqwest | 0.12 | HTTP client (Ollama, HuggingFace CDN) |
| uuid | 1.x | Unique identifiers (v4 feature) |
| futures-util | 0.3 | Async utilities |
| *Pending:* keyring | - | OS keychain (Phase 2) |

### Frontend (package.json)
| Package | Version | Vai trò |
|---------|---------|---------|
| react | 19.1.0 | UI framework |
| react-dom | 19.1.0 | React rendering |
| typescript | ~5.8.3 | Type safety |
| vite | 7.0.4 | Build tool |
| tailwindcss | 4.1.18 | Styling (v4) |
| @tailwindcss/vite | 4.1.18 | Tailwind Vite plugin |
| zustand | 5.0.11 | State management |
| @tauri-apps/api | 2.x | Tauri IPC bridge |
| @tauri-apps/plugin-sql | 2.3.2 | SQL client bridge |
| @tauri-apps/plugin-dialog | 2.x | Native file dialogs (Sprint 5) |
| shadcn/ui + radix-ui | 3.8.4 / 1.4.3 | UI component library |
| sonner | latest | Toast notification system (v0.7.1) |
| lucide-react | 0.563.0 | Icon library (Languages, Moon, Sun, FileText, FileCode, Mic, Loader2 icons) |

## 5. Sprint 2 Audio Module Files (Newly Implemented)

### Backend Audio Engine (Rust)

| File | Mô tả |
|------|-------|
| `src-tauri/src/audio/types.rs` | Audio types (DeviceInfo, AudioConfig, AudioFrame) |
| `src-tauri/src/audio/device.rs` | Device enumeration (cpal input/output, WASAPI loopback) |
| `src-tauri/src/audio/resampler.rs` | AudioResampler class (rubato FftFixedInOut, 48→16kHz, stereo→mono) |
| `src-tauri/src/audio/capture.rs` | AudioCaptureManager (mic stream + loopback stream, crossbeam channels, Arc<Mutex> stt_tx) |
| `src-tauri/src/audio/vad.rs` | Energy-based VAD (RMS + ZCR metrics, SpeechBuffer with 30s frame cap) |
| `src-tauri/src/audio/mod.rs` | Module re-exports (public types, manager) |
| `src-tauri/src/commands/audio.rs` | Tauri commands (list_audio_devices, start_audio_capture, stop_audio_capture) + AudioState |

### Frontend Audio UI (React/TypeScript)

| File | Mô tả |
|------|-------|
| `src/hooks/use-audio-capture.ts` | Custom hook wrapping Tauri invoke/Channel, manages capture lifecycle |
| `src/components/audio-device-selector.tsx` | React component for device selection dropdown, start/stop buttons |

## 6. Sprint 5 Caption Overlay + Export Module Files (Newly Implemented)

### Backend Storage & Export (Rust)

| File | Mô tả |
|------|-------|
| `src-tauri/src/storage/transcript_store.rs` | TranscriptDb với Arc<Mutex<Connection>>, WAL mode, save_segment/get_meeting_transcript methods |
| `src-tauri/src/storage/migrations.rs` | Migration V2: ALTER TABLE transcripts ADD COLUMN segment_id TEXT UNIQUE NOT NULL |
| `src-tauri/src/commands/export.rs` | export_transcript command, ExportFormat trait, TxtFormatter/MdFormatter/JsonFormatter (multi-lang in Sprint 6) |
| `src-tauri/src/commands/overlay.rs` | show_overlay_window/hide_overlay_window commands, label "overlay" (multi-lang in Sprint 6) |

### Frontend Overlay & UI (React/TypeScript)

| File | Mô tả |
|------|-------|
| `src/overlay/overlay.tsx` | Overlay window React entry point, listen "caption-update" event (multi-lang support in Sprint 6) |
| `src/overlay/overlay.html` | Overlay HTML template, separate Vite entry point |
| `src/components/settings-panel.tsx` | Settings UI: multi-select target languages with pill buttons, max 4, font size slider, overlay toggle (Sprint 6) |
| `src/components/transcript-timeline.tsx` | Tabbed timeline display, switch between target languages (Sprint 6) |
| `src/components/export-button.tsx` | Export button, multi-lang export support TXT/MD/JSON (Sprint 6) |
| `vite.config.ts` | Multi-page build: rollupOptions input { main: index.html, overlay: overlay.html } |

## 7. Sprint 6 Multi-target Language Support Module Files (Newly Implemented)

### Backend Translation Engine (Rust)

| File | Mô tả |
|------|-------|
| `src-tauri/src/storage/models.rs` | TranslationRecord struct (transcript_id, target_lang, translated_text) - Serde Serialize/Deserialize |
| `src-tauri/src/storage/migrations.rs` | Migration V3: CREATE TABLE translations (id, transcript_id, target_lang, translated_text) với UNIQUE constraint, ON DELETE CASCADE |
| `src-tauri/src/storage/transcript_store.rs` | Methods: insert_translation, get_translations_for_transcript, get_translations_for_meeting |
| `src-tauri/src/commands/translation.rs` | Refactored translate_text: `target_langs: Vec<String>`, tokio::JoinSet + Semaphore(3) parallel fan-out, 30s per-lang timeout |
| `src-tauri/src/translation/pipeline.rs` | Updated for multi-lang: each language gets own pipeline instance, save to translations table |

### Frontend Multi-language Support (React/TypeScript)

| File | Mô tả |
|------|-------|
| `src/types/index.ts` | Translation types: nested Record<segmentId, Record<language, text>> structure |
| `src/stores/app-store.ts` | Translation slice: multi-lang state structure với demuxing by language |
| `src/hooks/use-translation-events.ts` | Event handler: demux translation-update events by target_lang field |
| `src/hooks/use-auto-translation.ts` | Auto-translate: invoke translate_text with targetLangs array |
| `src/components/settings-panel.tsx` | Multi-select language selector: pill buttons, max 4 languages, validation |
| `src/components/transcript-timeline.tsx` | Tabbed transcript view: switch between target languages, show canonical lang as default |
| `src/overlay/overlay.tsx` | Multi-lang caption display: show target language label, fallback to primary lang |
| `src-tauri/src/commands/export.rs` | Multi-lang export: TXT/MD/JSON include all translations from translations table |

## 7. Sprint 4 Translation Pipeline Module Files (Newly Implemented)

### Backend Translation Engine (Rust)

| File | Mô tả |
|------|-------|
| `src-tauri/src/providers/traits.rs` | LLM provider trait with RPITIT (chat, chat_streaming, health_check, name) |
| `src-tauri/src/providers/ollama.rs` | OllamaProvider struct (HTTP client initialization, model management) |
| `src-tauri/src/providers/ollama_types.rs` | Serde types (OllamaRequest, OllamaResponse, OllamaModel, etc.) |
| `src-tauri/src/providers/ollama_error.rs` | OllamaError enum with thiserror derive |
| `src-tauri/src/translation/pipeline.rs` | TranslationPipeline orchestrator (invoke provider, emit events) |
| `src-tauri/src/translation/translation_types.rs` | TranslationResult, TranslationUpdate event payload |
| `src-tauri/src/commands/translation.rs` | Tauri commands (5 total: health, translate, list, pull, delete) + TranslationState |

### Frontend Translation Integration (React/TypeScript)

| File | Mô tả |
|------|-------|
| `src/types/index.ts` | Translation types (TranslationResult, TranslationUpdate, OllamaModel) |
| `src/hooks/use-translation-events.ts` | useTranslationEvents hook for streaming translation |
| `src/hooks/use-auto-translation.ts` | useAutoTranslation hook for auto-translate on transcript changes |
| `src/stores/app-store.ts` | Translation slice in Zustand (results, loading, error) |
| `src/components/transcript-panel.tsx` | Updated to display original + translated text side-by-side |

## 8. Multi-page Vite Build (Sprint 5)

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        overlay: resolve(__dirname, 'src/overlay/overlay.html')
      }
    }
  }
});
```

**Entry points:**
- `index.html` → Main app window (App.tsx)
- `src/overlay/overlay.html` → Overlay window (overlay.tsx)

**Tauri window labels:**
- `main` - Main app window
- `overlay` - Caption overlay window (transparent, always-on-top, no decorations)

## 9. Sprint 3 STT + VAD Module Files (Newly Implemented)

### Backend STT Engine (Rust)

| File | Mô tả |
|------|-------|
| `src-tauri/src/stt/whisper.rs` | SttEngine wrapper (sync transcribe + async transcribe_async) |
| `src-tauri/src/stt/model_manager.rs` | ModelManager (HuggingFace CDN downloads, progress tracking, temp files) |
| `src-tauri/src/stt/pipeline.rs` | SttPipeline orchestrator (VAD→resampling→STT→event emission on dedicated thread) |
| `src-tauri/src/audio/vad.rs` | Energy-based VAD module (RMS + ZCR detection, SpeechBuffer) |
| `src-tauri/src/commands/stt.rs` | Tauri commands (check_model_status, download_model, start_meeting, stop_meeting) |

### Frontend STT Integration (React/TypeScript)

| File | Mô tả |
|------|-------|
| `src/types/index.ts` | TypeScript types (SttEventPayload, TranscriptEntry, ModelStatus) |
| `src/hooks/use-tauri-events.ts` | Generic Tauri event listener hook (mounted flag race fix) |
| `src/hooks/use-stt-events.ts` | STT-specific event listener hook |
| `src/stores/app-store.ts` | Zustand store with STT slice (transcript, currentCaption, isTranscribing) |
| `src/components/transcript-panel.tsx` | Scrollable transcript display with auto-scroll feature |
| `src/App.tsx` | Meeting start/stop buttons, transcript panel integration |

## 9. Hướng Dẫn Tìm Code

- **IPC commands** (Rust ↔ React): `src-tauri/src/commands/`
- **Audio pipeline**: `src-tauri/src/audio/` → `stt/` → `translation/`
- **UI components**: `src/components/`
- **State management**: `src/stores/`
- **Database schema**: `src-tauri/src/storage/models.rs`
- **AI provider config**: `src-tauri/src/providers/`

---

## 10. Dependencies Summary (Cargo.toml)

### Core Framework
- `tauri` 2.x - Desktop framework
- `tauri-plugin-sql` 2.x - SQLite via Tauri plugin
- `tokio` 1.x - Async runtime

### Speech & Audio
- `whisper-rs` 0.15.1 - Speech-to-text engine (whisper.cpp binding)
- `cpal` 0.15 - Cross-platform audio library
- `rubato` 1.0 - Audio resampling library
- `uuid` 1.x - Unique identifier generation (v4 feature)

### HTTP & Async
- `reqwest` 0.12 - HTTP client (with stream feature for downloads)
- `futures-util` 0.3 - Async utilities

### Serialization & Logging
- `serde` + `serde_json` 1.x - Serialization
- `tracing` + `tracing-subscriber` 0.1/0.3 - Structured logging
- `thiserror` 2.x + `anyhow` 1.x - Error handling

### Cross-platform Support
- `crossbeam` 0.8 - Multi-threading utilities (channel)
- `tauri-plugin-opener` 2.x - File/URL opener

## 11. Key Architecture Patterns

### Translation Pipeline Architecture (Sprint 4)
```rust
Frontend: translate_text command
  → TranslationPipeline::translate()
    ├── OllamaProvider::chat_streaming()
    │   ├── POST /api/chat (native Ollama endpoint)
    │   └── reqwest bytes_stream() for NDJSON
    ├── Parse streaming responses
    └── Emit "translation-update" events
      → Frontend useTranslationEvents hook
        → Update Zustand translation slice
          → TranscriptPanel renders translations
```

### STT Pipeline Architecture (Sprint 3)
```rust
AudioCaptureManager (tokio task)
  ↓ Arc<Mutex<mpsc::Sender>>
SttPipeline (dedicated std::thread)
  ├── VAD (energy-based: RMS + ZCR)
  ├── AudioResampler (44.1/48kHz → 16kHz mono)
  ├── SttEngine (whisper-rs)
  └── Tauri event emitter
      ├── emit "stt-partial" (streaming)
      └── emit "stt-final" (complete)
```

### Model Download Flow
```
Frontend: check_model_status → ModelManager
  ├── Check cache
  └── If missing: download_model
      ├── HuggingFace CDN request
      ├── Progress tracking via event
      ├── Temp file with atomic rename
      └── Return status
```

### Frontend STT Integration
```
App.tsx (start_meeting command)
  → useSttEvents hook
    ├── Listen to "stt-partial" events
    ├── Listen to "stt-final" events
    └── Update Zustand store (transcript slice)
      ├── transcript array
      ├── currentCaption string
      └── isTranscribing boolean
  → TranscriptPanel component renders from store
```

## 10. Key Architectural Fixes (Sprint 5)

### Race Condition Fixes

1. **Meeting Record Lifecycle**: Meeting record creation moved từ frontend useEffect vào Rust start_meeting success callback → eliminates race where record exists nhưng pipeline chưa start

2. **Canonical segment_id**: Rust emit segment_id trực tiếp thay vì frontend construct → prevents inconsistency khi frontend concat meeting_id + index independently

3. **Atomic stop_meeting**: stop_meeting command atomically clear state và emit final meeting_id để frontend biết exactly which meeting stopped

### Database Concurrency

- WAL mode (Write-Ahead Logging): Concurrent reads không block writes
- Arc<Mutex<Connection>>: Thread-safe shared DB access across Tauri commands
- Migration V2: segment_id UNIQUE constraint ensures no duplicates

### Cross-Window Communication

```rust
// Main window emits
app_handle.emit("caption-update", payload)?;

// Overlay window listens
listen("caption-update", handler);
```

**Pattern**: Fire-and-forget events cho real-time UI updates across windows

## 12. UI/UX Polish (v0.7.1)

### shadcn Components Added
- **Select**: Audio device selector với icon support
- **Slider**: Settings panel font size control
- **AlertDialog**: Delete confirmation trong notes panel
- **Tooltip**: Language pills hover hints
- **Badge**: LIVE indicator trong transcript timeline
- **Separator**: Visual section dividers
- **Sonner**: Toast notifications cho user feedback

### Component Modularization
- **NoteCard.tsx** (197 lines): Extracted từ notes-panel.tsx
  - Inline editing UI với edit/save/cancel buttons
  - Delete confirmation với AlertDialog
  - Toast feedback on success/error
  - JSON parsing với error handling

### App Layout Enhancements
- **Header bar**: Languages icon + recording indicator (pulsing red dot)
- **Dark mode toggle**: Moon/Sun icon button (toggles `dark` class)
- **Recording indicator**: Shows when `isTranscribing === true`

### Visual Feedback Improvements
- **Loading states**: Start/stop meeting buttons với Loader2 spinner
- **Export buttons**: FileText/FileCode icons, loading spinner, toast on success
- **Copy button**: Transcript timeline copy với toast confirmation
- **Empty states**: Icons + placeholder text khi no data
- **LIVE badge**: Real-time indicator trong transcript timeline

### Config Fix
- **tauri.conf.json**: Fixed `plugins.sql.preload` từ map format → array format
  - Previous (panic on startup): `{ "sqlite:translator.db": [...] }`
  - Fixed: `["sqlite:translator.db"]`

---

> **Lưu ý**: Document này sẽ được cập nhật liên tục khi implementation tiến triển. Last updated: v0.7.1 UI/UX Polish (2026-02-11)
