# Changelog

Tất cả thay đổi đáng chú ý của dự án sẽ được ghi nhận tại đây.

Format theo [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning theo [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2026-02-11 (UI/UX Polish)

### Added
- shadcn components: Select, Slider, AlertDialog, Tooltip, Badge, Separator, Sonner (toast notifications)
- NoteCard component: Extracted note rendering logic từ notes-panel (197 lines, modular design)
- App header: Languages icon, recording indicator (pulsing red dot), dark mode toggle (Moon/Sun icons)
- Audio device selector: shadcn Select với Mic icon, pulsing green dot khi đang capture
- Settings panel: shadcn Slider cho font size, Tooltips on language pills
- Notes panel: AlertDialog confirmation cho delete, toast feedback on success/error
- Transcript timeline: Copy button với toast notification, LIVE badge, empty state với icon
- Export buttons: File type icons (FileText, FileCode), Loader2 spinner, toast notifications
- Loading states: Start/stop meeting buttons hiển thị loading spinner
- Dark mode toggle: Header button chuyển đổi light/dark theme

### Changed
- Audio device selector: Native select → shadcn Select component với Mic icon
- Settings panel: Native input → shadcn Slider component
- Notes panel: Modularized với NoteCard component, inline editing UI improved
- Export buttons: Enhanced với icons, loading states, toast feedback

### Fixed
- tauri.conf.json: Fixed `plugins.sql.preload` từ map format → array format (was causing panic on startup)
- Settings panel: Overlay button variant standardized

### Technical Decisions
- Modular component extraction: NoteCard.tsx giữ file size manageable (notes-panel + note-card < 400 LOC total)
- shadcn/ui components: Consistent design system across all UI elements
- Toast notifications (Sonner): User feedback cho all CRUD operations
- Loading states: Visual feedback cho async operations (export, start/stop meeting)

### Related Files
- Frontend: src/components/note-card.tsx (NEW - 197 lines)
- Frontend: src/components/ui/ (alert-dialog, badge, select, separator, slider, sonner, tooltip)
- Frontend: src/components/app-layout.tsx (header with recording indicator + dark mode toggle)
- Frontend: src/components/audio-device-selector.tsx (shadcn Select + icons)
- Frontend: src/components/settings-panel.tsx (shadcn Slider + Tooltips)
- Frontend: src/components/notes-panel.tsx (AlertDialog + toast integration)
- Frontend: src/components/transcript-timeline.tsx (copy button + LIVE badge)
- Frontend: src/components/export-button.tsx (icons + loading + toast)
- Frontend: src/components/memo-export-button.tsx (icons + loading + toast)
- Config: src-tauri/tauri.conf.json (sql plugin preload fix)

## [0.7.0] - 2026-02-11 (Sprint 7: Note Engine + Meeting Memo)

### Added
- NoteEngine module: Incremental summarization orchestrator (notes/note_engine.rs)
- OllamaSummarizer: LLM-based note generation via Ollama API (notes/ollama_summarizer.rs)
- Prompt templates: System prompt + 4 category-specific prompts (notes/prompt_templates.rs)
- Note categories: key_points, decisions, action_items, risks (notes/note_types.rs)
- MemoBuilder: Meeting memo formatter with markdown output (notes/memo_builder.rs)
- NoteStore: SQLite CRUD operations for notes table (storage/note_store.rs)
- NoteState: Shared mutable state for note management (Arc<Mutex<NoteState>>)
- 5 Tauri commands: get_notes, update_note, delete_note, generate_memo, export_memo
- NotesPanel component: 4-tab UI (key_points/decisions/action_items/risks)
- MemoExportButton component: Export meeting memo to .md file
- notes-updated event: Real-time note synchronization
- useNoteEvents hook: Event listener for note updates

### Changed
- Database schema Migration V4: CREATE TABLE notes (id, meeting_id, category, content, created_at, updated_at)

### Technical Decisions
- NoteEngine uses OllamaSummarizer for flexible LLM provider
- Prompt engineering: Category-specific prompts with system context
- Inline editing: Notes editable via update_note command
- Markdown memo format: H1 title, H2 sections, bullet points
- Note categories stored as TEXT enum (key_points, decisions, action_items, risks)

### Related Files
- Backend: notes/ (note_engine.rs, ollama_summarizer.rs, prompt_templates.rs, note_types.rs, memo_builder.rs)
- Backend: storage/note_store.rs, storage/migrations.rs (V4), commands/notes.rs
- Frontend: components/notes-panel.tsx, components/memo-export-button.tsx
- Frontend: hooks/use-note-events.ts, types/notes.ts
- Frontend: stores/app-store.ts (notes slice)

## [0.6.0] - 2026-02-10 (Sprint 6: Multi-target Language Support)

### Added
- Migration V3: Thêm table `translations` (transcript_id, target_lang, translated_text) với UNIQUE constraint, ON DELETE CASCADE
- TranslationRecord model: Serde struct cho multi-lang translations với methods insert, query by transcript/meeting
- Parallel translation support: tokio::JoinSet + Semaphore(3) capping concurrent Ollama requests
- 30-second translation timeout per language với graceful error handling
- Multi-language target selector UI: Pill buttons với max 4 languages selectable
- Tabbed transcript timeline: Switch between target languages, canonical lang display
- Multi-language caption overlay: Show target language label, fallback to primary lang
- Multi-language export: TXT/MD/JSON formats với all translations included
- Frontend translation store: Nested Record<segmentId, Record<language, text>> structure
- useAutoTranslation hook: Support for targetLangs array, demux translation-update events by lang

### Changed
- `translate_text` Tauri command: `target_lang: String` → `target_langs: Vec<String>`
- Frontend store shape: translations from `{ [id]: text }` → `{ [id]: { [lang]: text } }`
- `start_meeting` command: Now accepts `target_langs: Vec<String>` parameter
- TranslationUpdatePayload: Include `target_lang` field để demux on frontend

### Fixed
- Translation latency: Parallel processing keeps latency sublinear (not O(n*lang_count))
- Storage queries: Index on (transcript_id, target_lang) for fast lookups

### Technical Decisions
- tokio::JoinSet for fan-out parallel translation tasks (cleaner than spawn + collect)
- Semaphore(3) matches typical `OLLAMA_NUM_PARALLEL` env (user can configure if needed)
- Normalized `translations` table instead of JSON columns (better indexing + query flexibility)
- Per-language timeout (30s) prevents one slow language blocking others

### Related Files
- `src-tauri/src/storage/models.rs` — TranslationRecord struct
- `src-tauri/src/storage/migrations.rs` — Migration V3
- `src-tauri/src/storage/transcript_store.rs` — insert_translation, get_translations_for_transcript/meeting methods
- `src-tauri/src/commands/translation.rs` — Refactored translate_text with Vec<String>
- `src-tauri/src/translation/pipeline.rs` — Updated for multi-lang pipeline instances
- `src/stores/app-store.ts` — Translation slice with nested Record structure
- `src/hooks/use-translation-events.ts` — Event demuxing by language
- `src/hooks/use-auto-translation.ts` — Multi-lang translation invocation
- `src/components/settings-panel.tsx` — Multi-select language selector (pills, max 4)
- `src/components/transcript-timeline.tsx` — Tabbed language view
- `src/overlay/overlay.tsx` — Multi-lang caption display with language labels
- `src-tauri/src/commands/export.rs` — Multi-lang export support

## [0.5.0] - 2026-02-10 (Sprint 5: Caption Overlay + Transcript Export)

### Added
- TranscriptDb: Thread-safe SQLite storage với Arc<Mutex<Connection>>, WAL mode cho concurrency
- Migration V2: Thêm segment_id column vào transcripts table để tracking canonical segment IDs
- export_transcript command: Export transcript với 3 formats (TXT, MD, JSON)
  - TxtFormatter: Plain text với timestamps
  - MdFormatter: Markdown table với Original/Translation columns
  - JsonFormatter: Structured JSON với metadata và segments
- Caption overlay window: Transparent, always-on-top, no decorations Tauri window
  - Vite multi-page build: main.html + overlay.html entry points
  - Cross-window event communication qua "caption-update" event
- SettingsPanel component: Target language selector, font size slider, overlay toggle
- TranscriptTimeline component: 3-column grid (Time | Original | Translation) với auto-scroll
- ExportButton component: Native file save dialog qua tauri-plugin-dialog
- Overlay window commands: show_overlay_window, hide_overlay_window với label "overlay"

### Fixed
- Race condition: Meeting record bị create trước khi STT pipeline start → moved meeting creation vào start_meeting success callback
- Canonical segment_id emit: Rust emit segment_id trực tiếp thay vì frontend construct lại (tránh inconsistency)
- Atomic meeting_id lifecycle: stop_meeting atomically clear state và emit final meeting_id để prevent stale references

### Changed
- Vite config: Multi-page build với rollupOptions input { main, overlay }
- Database schema: transcripts.segment_id TEXT UNIQUE NOT NULL (migration V2)
- start_meeting event payload: Include meeting_id ngay khi start thành công

### Dependencies Added
- tauri-plugin-dialog 2.x: Native file save dialog integration

### Technical Decisions
- WAL mode SQLite cho concurrent reads without blocking writes
- Arc<Mutex<Connection>> shared across commands, initialized at app setup
- Canonical segment_id format: {meeting_id}:{index} generated in Rust
- Overlay window lifecycle: Pre-created hidden window (không dynamic spawn)
- Export formatters: Strategy pattern với ExportFormat trait

### Related Files
- `src-tauri/src/storage/transcript_store.rs` — TranscriptDb implementation
- `src-tauri/src/storage/migrations.rs` — Migration V2 for segment_id column
- `src-tauri/src/commands/export.rs` — export_transcript command + formatters
- `src-tauri/src/commands/overlay.rs` — show/hide overlay window commands
- `src/overlay/` — Overlay window React app (overlay.tsx, overlay.html)
- `src/components/settings-panel.tsx` — Settings UI component
- `src/components/transcript-timeline.tsx` — Transcript display component
- `src/components/export-button.tsx` — Export UI với native dialog
- `vite.config.ts` — Multi-page build configuration

## [0.4.0] - 2026-02-10 (Sprint 4: Translation Pipeline - Ollama)

### Added
- LLM Provider abstraction: RPITIT trait-based design in `providers/traits.rs` (chat, chat_streaming, health_check, name methods)
- OllamaProvider implementation: Full HTTP API integration with native Ollama endpoints
- NDJSON streaming support: Efficient streaming event parsing from Ollama /api/chat endpoint
- Model management API: list_ollama_models, pull_ollama_model, delete_ollama_model Tauri commands
- TranslationPipeline orchestrator: Text → Ollama → streaming result emission
- Tauri commands: ollama_health_check, translate_text, list_ollama_models, pull_ollama_model, delete_ollama_model
- Event emission: translation-update event with partial results (streaming)
- Frontend Translation slice: Zustand store for translation state management
- Frontend hooks: useTranslationEvents for listening to translation streams
- Frontend integration: TranscriptPanel component updated to show translations alongside transcripts
- Error handling: OllamaError enum with thiserror for provider-specific errors

### Dependencies Added
- No new external crates (reqwest already present from Sprint 3)

### Technical Decisions
- Native Ollama HTTP API (POST /api/chat) instead of client libraries for flexibility
- RPITIT trait pattern (Return Position Impl Trait In Trait) for streaming support without async_trait crate
- reqwest bytes_stream() for efficient NDJSON line-by-line streaming
- Default model: qwen2.5:3b (3B param, good balance for translations)
- thiserror for OllamaError, anyhow in command handlers (consistent with STT module)
- TranslationState registered in app state, accessible from all translation commands

### Related Files
- `src-tauri/src/providers/traits.rs` — LLM provider trait (chat, chat_streaming methods)
- `src-tauri/src/providers/ollama.rs` — OllamaProvider implementation
- `src-tauri/src/providers/ollama_types.rs` — Serde request/response types
- `src-tauri/src/providers/ollama_error.rs` — OllamaError enum
- `src-tauri/src/translation/pipeline.rs` — TranslationPipeline with streaming
- `src-tauri/src/translation/translation_types.rs` — TranslationResult types
- `src-tauri/src/commands/translation.rs` — 5 Tauri commands
- `src/types/index.ts` — Frontend translation types
- `src/stores/app-store.ts` — Translation slice in Zustand
- `src/hooks/use-translation-events.ts` — useTranslationEvents hook
- `src/hooks/use-auto-translation.ts` — useAutoTranslation integration hook
- `src/components/transcript-panel.tsx` — Updated with translation display

## [0.3.0] - 2026-02-10 (Sprint 3: STT Integration + VAD)

### Added
- STT engine: whisper-rs 0.15.1 integration with sync + async transcription
- Model manager: HuggingFace CDN model download with progress events and temp file handling
- SttPipeline orchestrator: VAD → resampling → STT → Tauri event emission on dedicated std::thread
- Voice Activity Detection (VAD): Energy-based implementation with RMS + ZCR, SpeechBuffer with 30s cap
- Audio resampler integration: Support for 44.1kHz/48kHz → 16kHz mono conversion
- AudioCaptureManager updates: Arc<Mutex> stt_tx for dynamic sender, mic_format() method
- Tauri commands: check_model_status, download_model, start_meeting, stop_meeting
- Frontend types: SttEventPayload, TranscriptEntry, ModelStatus interfaces
- Tauri event listener: use-stt-events hook with mounted flag race condition fix
- State management: STT slice in Zustand store (transcript, currentCaption, isTranscribing)
- UI components: transcript-panel with scrollable display and auto-scroll feature
- App integration: Meeting start/stop buttons, transcript panel rendering

### Dependencies Added
- whisper-rs 0.15.1 (speech-to-text engine)
- reqwest 0.12 with stream feature (HTTP client for model downloads)
- uuid 1.x with v4 feature (unique identifiers)
- futures-util 0.3 (async utilities)

### Technical Decisions
- Energy-based VAD chosen over external libraries (minimal dependencies)
- Dedicated std::thread for pipeline instead of tokio (cleaner shutdown)
- Arc<Mutex> for dynamic stt_tx to prevent snapshot stale issues
- JoinHandle storage in SttPipeline for clean thread management
- Whisper greedy sampling with best_of: 1 for consistent results

## [0.2.0] - 2026-02-10 (Sprint 2: Audio Capture)

### Added
- Audio capture module with cpal 0.17 (mic input)
- WASAPI loopback capture for system audio (Windows)
- AudioResampler with rubato 1.0 (stereo→mono, sample rate conversion)
- Tauri commands: list_audio_devices, start_audio_capture, stop_audio_capture
- Crossbeam channel pipeline: mic→processor→IPC bridge→frontend
- React AudioDeviceSelector component with device selection UI
- useAudioCapture hook with Tauri Channel binary streaming
- Audio state management in Zustand store
- Audio module files: types.rs, device.rs, resampler.rs, capture.rs, mod.rs

### Fixed
- Memory leak from std::mem::forget(stream) - now stores stream handle for proper cleanup
- React hook unmount cleanup - stops capture on component unmount
- IPC bridge auto-stops manager when frontend channel disconnects

## [0.1.0] - 2026-02-10

### Added
- Tauri v2 project scaffold (React 19 + TypeScript + Vite 7)
- Rust backend module skeleton (audio, stt, translation, notes, providers, storage, commands, auth, security)
- Frontend with Tailwind CSS v4, zustand state management, shadcn/ui component library
- SQLite database with tauri-plugin-sql (meetings, transcripts, notes tables with migrations)
- GitHub Actions CI pipeline (clippy, test, build with caching)
- Tauri capabilities security configuration (minimal default permissions)
- Project documentation suite (9 files):
  - `docs/system-architecture.md` — Kiến trúc chi tiết với Mermaid diagrams
  - `docs/project-overview-pdr.md` — Product Development Requirements
  - `docs/code-standards.md` — Chuẩn mã nguồn Rust + TypeScript/React
  - `docs/development-roadmap.md` — Lộ trình 4 phases với timeline chi tiết
  - `docs/design-guidelines.md` — Hướng dẫn thiết kế UI/UX
  - `docs/codebase-summary.md` — Tổng quan codebase (template)
  - `docs/deployment-guide.md` — Hướng dẫn build & distribution
  - `README.md` — Project overview, setup instructions
  - `docs/project-changelog.md` — Changelog (this file)

## [0.0.0] - 2026-02-09

### Added
- Khởi tạo project documentation suite (9 files)
- `docs/system-architecture.md` — Kiến trúc chi tiết với Mermaid diagrams
- `docs/project-overview-pdr.md` — Product Development Requirements
- `docs/code-standards.md` — Chuẩn mã nguồn Rust + TypeScript/React
- `docs/development-roadmap.md` — Lộ trình 4 phases với timeline chi tiết
- `docs/design-guidelines.md` — Hướng dẫn thiết kế UI/UX
- `docs/codebase-summary.md` — Tổng quan codebase (template)
- `docs/deployment-guide.md` — Hướng dẫn build & distribution
- `README.md` — Project overview, setup instructions

## [0.0.0] - 2026-02-10

### Added
- Khởi tạo dự án RT Translator
- Product specification (`docs/specs.md`)
- Development infrastructure (.claude rules, skills, hooks)
- Implementation plan framework (`plans/`)

---

### Quy Ước Ghi Changelog

**Categories:**
- `Added` — Tính năng mới
- `Changed` — Thay đổi tính năng hiện có
- `Deprecated` — Tính năng sẽ bị loại bỏ
- `Removed` — Tính năng đã loại bỏ
- `Fixed` — Sửa lỗi
- `Security` — Cập nhật bảo mật

**Format mỗi entry:**
```
## [x.y.z] - YYYY-MM-DD

### Added
- Mô tả ngắn gọn thay đổi
```
