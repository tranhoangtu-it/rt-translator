# Lộ Trình Phát Triển
> Real-Time Multilingual Meeting Translator & Notetaker

## 1. Tổng Quan Timeline

| Phase | Thời gian | Tính năng chính | Trạng thái |
|-------|-----------|-----------------|------------|
| **Phase 1: MVP Desktop** | 8-10 tuần | Audio capture, STT, dịch 1 ngôn ngữ, caption overlay | ✅ Complete (All 5 Sprints) |
| **Phase 2: Multi-lang + Notes + Auth** | 8-10 tuần | Dịch đa ngôn ngữ, note-taking, auth, key vault | Not Started |
| **Phase 3: Doc Translation + CLI** | 6-8 tuần | Dịch file (DOCX/PPTX/XLSX/PDF), CLI đầy đủ | Not Started |
| **Phase 4: Mobile + Advanced** | 8-10 tuần | Mobile app, speaker diarization, glossary, quality scoring | Not Started |
| **Tổng cộng** | **30-38 tuần** | | |

---

## 2. Phase 1: MVP Desktop (8-10 tuần)

### Sprint 1 (Tuần 1-2): Project Setup + Tauri Scaffold ✅ Complete

**Mục tiêu:** Thiết lập môi trường, Tauri boilerplate, CI/CD cơ bản

**Tasks:**
- Setup repository + Git workflow + branch protection
- Init Tauri v2 project (Rust + React/TypeScript)
- Configure build pipeline (GitHub Actions hoặc GitLab CI)
- Setup Rust crate structure (modules: audio, stt, translation, providers, storage)
- Setup React frontend (Tailwind CSS, zustand, routing)
- Write project docs: CLAUDE.md, code-standards.md, system-architecture.md
- Init SQLite database schema (meetings, transcripts, notes)

**Deliverables:**
- Tauri app chạy được (empty window)
- Rust module skeleton
- CI/CD chạy tests + build artifacts

**Success Criteria:**
- `npm run tauri dev` chạy không lỗi
- Unit tests pass (skeleton modules)
- Build .exe/.dmg thành công

---

### Sprint 2 (Tuần 3-4): Audio Capture (cpal + WASAPI) ✅ Complete

**Mục tiêu:** Thu được system audio + microphone trên Windows/macOS

**Tasks:** ✅ Hoàn thành
- Implement `audio/capture.rs` với `cpal` 0.15 ✅
- Xử lý WASAPI loopback capture (Windows) + CoreAudio loopback (macOS) ✅
- Dual stream mode: thu riêng mic + system audio ✅
- Audio processing: downmix stereo → mono 16kHz f32 ✅
- GUI: device selector (list input/output devices) ✅
- Unit tests: mock audio stream, verify PCM format ✅

**Deliverables:** ✅ Hoàn thành
- Module `audio::capture` hoạt động ✅
- Tauri commands: `list_audio_devices`, `start_audio_capture`, `stop_audio_capture` ✅
- Tauri Channel binary streaming: PCM chunks đến frontend ✅

**Success Criteria:** ✅ Đạt được
- Capture được system audio + mic không lỗi ✅
- PCM stream 16kHz mono f32 đúng định dạng ✅
- Tests pass cho dual stream mode ✅

---

### Sprint 3 (Tuần 5-6): STT Integration (whisper-rs) + VAD ✅ Complete

**Mục tiêu:** Speech-to-text streaming với voice activity detection

**Tasks:** ✅ Hoàn thành
- Integrate `whisper-rs` 0.15.1 ✅
- Download + model manager via HuggingFace CDN (with progress events) ✅
- Implement `stt/whisper.rs`: sync + async transcription wrapper ✅
- VAD module (`audio/vad.rs`): energy-based VAD (RMS + ZCR) ✅
- SttPipeline (`stt/pipeline.rs`): orchestrator với dedicated std::thread ✅
- Audio resampler integration: 44.1/48kHz → 16kHz mono ✅
- Tauri commands: check_model_status, download_model, start_meeting, stop_meeting ✅
- Frontend hooks + components: use-stt-events, transcript-panel ✅
- Tauri events: `stt-partial`, `stt-final` streaming ✅

**Deliverables:** ✅ Hoàn thành
- Module `stt::whisper` + `stt::model_manager` + `stt::pipeline` ✅
- VAD filter silence + SpeechBuffer (30s cap) ✅
- Event stream: `stt-partial`, `stt-final` ✅
- Frontend STT integration: Tauri event listener + Zustand store ✅

**Success Criteria:** ✅ Đạt được
- STT nhận dạng tiếng Anh/tiếng Việt chính xác >80% ✅
- Latency STT < 2s (từ audio → text) ✅
- Tests: mock audio → verify text output ✅
- Dynamic stt_tx with Arc<Mutex> fixed snapshot stale issue ✅
- JoinHandle stored for clean thread shutdown ✅

---

### Sprint 4 (Tuần 7-8): Translation Pipeline (Ollama) ✅ Complete

**Mục tiêu:** Dịch text real-time qua Ollama local LLM

**Tasks:** ✅ Hoàn thành
- Setup Ollama HTTP client (`providers/ollama.rs`) ✅
- Implement Provider trait (`providers/traits.rs`) ✅
- Translation engine (`translation/pipeline.rs`): text → Ollama API → translated text ✅
- Model management: list/pull/delete Ollama models via API ✅
- Default bundled model: qwen2.5:3b ✅
- Streaming translation: NDJSON response handling ✅
- Tauri commands: `translate_text`, `ollama_health_check`, `list_ollama_models`, `pull_ollama_model`, `delete_ollama_model` ✅
- Event: `translation-update` + streaming event emission ✅

**Deliverables:** ✅ Hoàn thành
- Module `providers::ollama` (OllamaProvider with RPITIT trait pattern) ✅
- Module `translation::pipeline` (TranslationPipeline with streaming) ✅
- Types: `providers::ollama_types` (Serde request/response) ✅
- Error handling: `providers::ollama_error` (thiserror) ✅
- Frontend types: Translation slice in Zustand, useTranslationEvents hook ✅
- Frontend component: TranscriptPanel updated with translations ✅

**Success Criteria:** ✅ Đạt được
- Dịch en→vi, vi→en chính xác (manual review) ✅
- Latency translation < 1s cho đoạn ngắn ✅
- NDJSON streaming parsing hoạt động ✅
- Tauri commands invoke thành công ✅
- Default model qwen2.5:3b ready ✅

---

### Sprint 5 (Tuần 9-10): Caption Overlay UI + Transcript Export ✅ Complete

**Mục tiêu:** Live caption hiển thị always-on-top, export transcript

**Tasks:** ✅ Hoàn thành
- React component: `CaptionOverlay` (always-on-top window) ✅
- Tauri window config: transparent, decorations: false, always_on_top: true ✅
- Caption UI: hiển thị translated text + timestamp, tự động scroll ✅
- Transcript view: timeline view, nguyên văn + dịch song song ✅
- Export transcript: .txt, .md, .json ✅
- Storage: lưu transcript vào SQLite (table: `transcripts`) ✅
- Tauri commands: `export_transcript(meeting_id: String, format: String)` ✅
- GUI: settings panel (chọn target language, font size, overlay position) ✅

**Deliverables:** ✅ Hoàn thành
- Caption overlay window hoạt động ✅
- Transcript view panel ✅
- Export transcript 3 formats (.txt/.md/.json) ✅
- Thread-safe TranscriptDb (Arc<Mutex<Connection>>) ✅
- Cross-window caption events ✅

**Success Criteria:** ✅ Đạt được
- Caption hiển thị real-time không lag ✅
- Overlay always-on-top, resize/move được ✅
- Export đúng format, đầy đủ data ✅
- Meeting lifecycle atomic (no race condition) ✅

---

### Phase 1 Deliverables Tổng Hợp

- Desktop app (Windows + macOS) chạy ổn định
- Audio capture dual stream (mic + system)
- STT streaming (Whisper)
- Translation real-time (Ollama) cho 1 ngôn ngữ đích
- Caption overlay UI + transcript export
- SQLite storage cơ bản

### Phase 1 Success Criteria

- User chạy app, chọn device, caption hiển thị real-time
- Latency end-to-end < 3s (audio → caption)
- Accuracy STT >80%, translation pass manual review
- App không crash trong 30 phút meeting test

---

## 3. Phase 2: Multi-lang + Notes + Auth (8-10 tuần) - In Progress (~40% Complete)

### Sprint 6 (Tuần 11-12): Multi-target Language Support ✅ Complete

**Mục tiêu:** Dịch song song nhiều ngôn ngữ

**Tasks:** ✅ Hoàn thành
- Refactor translation pipeline: support Vec<TargetLang> ✅
- Parallel translation requests đến Ollama (dùng tokio::JoinSet + Semaphore) ✅
- UI: multi-language caption view (tabs hoặc split panels) ✅
- Settings: chọn nhiều target languages (multi-select pill buttons, max 4) ✅
- Storage: lưu translations multi-lang trong SQLite (Migration V3, translations table) ✅

**Success Criteria:** ✅ Đạt được
- Dịch 1 source → 3 target languages đồng thời ✅
- Latency không tăng tuyến tính (parallel processing via tokio::JoinSet) ✅

---

### Sprint 7 (Tuần 13-14): Note Engine + Meeting Memo ✅ Complete

**Mục tiêu:** AI tạo note liên tục, xuất memo cuối buổi

**Tasks:** ✅ Hoàn thành
- Implement NoteEngine với incremental summarization ✅
- Ollama prompt engineering: 4 categories (key_points, decisions, action_items, risks) ✅
- GUI: Notes panel với 4 tabs, inline edit ✅
- End-of-meeting: generate Meeting Memo (summary + sections + timestamps) ✅
- Export memo: .md format ✅
- Storage: table `notes` (meeting_id, category, content, created_at) ✅

**Deliverables:** ✅ Hoàn thành
- NoteEngine orchestrator với OllamaSummarizer ✅
- Prompt templates module (system + category prompts) ✅
- NoteStore (DB CRUD operations) ✅
- 5 Tauri commands (get_notes, update_note, delete_note, generate_memo, export_memo) ✅
- Notes panel UI với 4-tab layout ✅
- Memo export button với markdown formatter ✅

**Success Criteria:** ✅ Đạt được
- Notes có thể được tạo và update via Ollama LLM ✅
- Memo export đầy đủ 4 sections (key_points, decisions, action_items, risks) ✅
- User có thể edit/delete notes trong meeting ✅
- Markdown export với proper formatting ✅

---

### Sprint 8 (Tuần 15-16): Authentication (Email/Password + Google SSO)

**Mục tiêu:** User login, sync settings (optional cloud sync)

**Tasks:**
- Backend auth service (Rust): email/password hash (argon2)
- Google OAuth2 integration (`oauth2` crate)
- SQLite: table `users`, `sessions`
- Tauri commands: `login`, `logout`, `register`, `google_login`
- GUI: Login screen, registration form
- Session management: JWT token stored in OS keychain

**Success Criteria:**
- User đăng ký/đăng nhập thành công
- Google OAuth flow hoàn tất
- Session persist sau khi đóng app

---

### Sprint 9 (Tuần 17-18): Key Vault + Security

**Mục tiêu:** Lưu API keys an toàn, encrypt data at rest

**Tasks:**
- Implement `security/keyring.rs`: wrapper cho `keyring` crate
- Store API keys (OpenAI, Google, etc.) trong OS keychain (DPAPI/Keychain)
- Encrypt SQLite database với master key từ OS vault
- Device binding: generate device-id + keypair per machine
- Tauri commands: `set_api_key(provider: String, key: String)`, `get_api_key(provider: String)`
- GUI: Settings → API Keys management

**Success Criteria:**
- API keys không lưu plaintext trong SQLite
- SQLite database encrypted
- Keys chỉ accessible từ device đã bind

---

### Sprint 10 (Tuần 19-20): Refine + Polish Phase 2

**Mục tiêu:** Bug fixes, performance tuning, UX polish

**Tasks:**
- Code review + refactor
- Performance profiling: optimize translation latency
- UI/UX polish: animations, error states, loading indicators
- Write integration tests (end-to-end: audio → caption → notes → export)
- Documentation: user guide (cài đặt, sử dụng, troubleshooting)

**Success Criteria:**
- No critical bugs
- Performance targets met (latency < 2s)
- User guide hoàn chỉnh

---

## 4. Phase 3: Doc Translation + CLI (6-8 tuần)

### Sprint 11 (Tuần 21-22): Document Translation - DOCX/PPTX

**Mục tiêu:** Dịch DOCX/PPTX giữ nguyên format

**Tasks:**
- Implement `doc-translator` module
- DOCX parser/writer (`docx-rs` crate)
- PPTX parser/writer (custom hoặc `zip` + XML parsing)
- Pipeline: extract text → translate → inject back vào XML → rebuild file
- Tauri command: `translate_document(path: String, tgt_lang: String)`
- GUI: drag-drop file, progress bar

**Success Criteria:**
- DOCX/PPTX dịch giữ nguyên styles, layout
- Font, colors, images không bị phá

---

### Sprint 12 (Tuần 23-24): Document Translation - XLSX/PDF

**Mục tiêu:** Dịch XLSX/PDF text-based

**Tasks:**
- XLSX parser/writer (`calamine` + `rust_xlsxwriter`)
- PDF text extraction (`lopdf` hoặc `pdfium-render`)
- PDF output: text-based PDF rebuild (best-effort)
- Support OCR PDF: integrate `tesseract` (optional)

**Success Criteria:**
- XLSX dịch giữ formulas, formatting
- PDF text-based dịch thành công (layout best-effort)

---

### Sprint 13 (Tuần 25-26): CLI Implementation

**Mục tiêu:** Full CLI cho batch operations + headless mode

**Tasks:**
- CLI binary riêng (`src-tauri/src/cli/mod.rs`)
- Commands: `meeting`, `transcript`, `doc`, `models`
- Examples:
  - `rt-translator meeting --src auto --tgt vi,ja --overlay on`
  - `rt-translator doc translate --in a.pptx --tgt vi --out a_vi.pptx`
  - `rt-translator models list | pull | set-default`
- Headless mode: chạy meeting capture không cần GUI
- Output: stdout logs, export files

**Success Criteria:**
- CLI chạy được mọi tính năng chính
- Batch translate 10 files thành công
- Headless mode capture + export transcript

---

### Sprint 14 (Tuần 27-28): Testing + Documentation

**Mục tiêu:** Comprehensive testing, docs hoàn chỉnh

**Tasks:**
- Unit tests: coverage >70%
- Integration tests: E2E workflows
- Performance tests: latency benchmarks
- Security audit: key storage, data encryption
- Documentation: API docs (Rust docs), CLI manual, architecture diagrams

**Success Criteria:**
- All tests pass
- Security audit clean
- Docs đầy đủ, dễ hiểu

---

## 5. Phase 4: Mobile + Advanced (8-10 tuần)

### Sprint 15 (Tuần 29-30): Mobile Companion App - Setup

**Mục tiêu:** React Native/Flutter app skeleton

**Tasks:**
- Init React Native hoặc Flutter project
- Setup Rust FFI bindings (cho STT/translation core nếu cần)
- Basic UI: caption view, transcript sync từ desktop
- WebSocket server trên desktop app để sync data với mobile

**Success Criteria:**
- Mobile app hiển thị caption từ desktop real-time

---

### Sprint 16 (Tuần 31-32): Mobile - Mic Capture + Standalone Mode

**Mục tiêu:** Mobile capture mic, dịch standalone (không cần desktop)

**Tasks:**
- Audio capture trên iOS/Android (mic only, no system audio)
- Integrate STT/translation trên mobile (gọi Ollama server desktop hoặc cloud)
- Offline mode: bundle model nhỏ trên mobile (optional)

**Success Criteria:**
- Mobile app capture mic, hiển thị caption độc lập

---

### Sprint 17 (Tuần 33-34): Speaker Diarization

**Mục tiêu:** Phân biệt người nói trong meeting

**Tasks:**
- Integrate speaker diarization model (pyannote.audio qua FFI hoặc HTTP service)
- Label captions theo speaker (Speaker 1, Speaker 2, ...)
- GUI: color-code speakers, assign names

**Success Criteria:**
- Diarization chính xác >70% (manual test)
- UI hiển thị speaker labels

---

### Sprint 18 (Tuần 35-36): Glossary + Quality Scoring

**Mục tiêu:** Custom glossary, translation quality confidence

**Tasks:**
- Glossary management: user thêm thuật ngữ custom (source → target)
- Translation engine: inject glossary vào prompt
- Quality scoring: confidence score cho mỗi translation
- Highlight đoạn dịch nghi ngờ (low confidence)

**Success Criteria:**
- Glossary áp dụng đúng trong translation
- Confidence score hiển thị, highlight low-quality segments

---

### Sprint 19 (Tuần 37-38): Final Polish + Release Prep

**Mục tiêu:** Bug fixes, packaging, release

**Tasks:**
- Full app testing (desktop + mobile)
- Performance optimization
- Packaging: .msi/.dmg installer, bundle Ollama + Whisper models
- App store submission (iOS/Android)
- Marketing materials: landing page, demo video

**Success Criteria:**
- App chạy ổn định, no critical bugs
- Installer hoàn chỉnh, sign/notarize
- Mobile app approved on stores

---

## 6. Theo Dõi Tiến Độ

| Phase | Trạng thái | % Hoàn thành | Sprint hiện tại | Ghi chú |
|-------|------------|--------------|-----------------|---------|
| Phase 1: MVP Desktop | ✅ Complete | 100% | Sprint 5 ✅ | All 5 sprints complete: Setup, Audio, STT, Translation, UI+Export |
| Phase 2: Multi-lang + Notes + Auth | In Progress | ~40% | Sprint 7 ✅ | Sprint 6-7 complete: Multi-target language + Note Engine (2/5 sprints done) |
| Phase 3: Doc Translation + CLI | Not Started | 0% | - | |
| Phase 4: Mobile + Advanced | Not Started | 0% | - | |

**Cập nhật mỗi sprint:** Điền % hoàn thành, sprint hiện tại, blocking issues (nếu có)

---

## 7. Rủi Ro

### Phase 1 Risks
- **WASAPI loopback**: `cpal` có bug, có thể cần `windows-rs` direct API (mitigate: research trước khi implement)
- **whisper-rs streaming**: Crate chính không hỗ trợ partial results (mitigate: dùng `whisper-stream-rs` hoặc chunked processing)
- **Ollama latency**: Translation chậm với model lớn (mitigate: dùng model nhỏ 2-3B, optimize prompt)

### Phase 2 Risks
- **Parallel translation**: Resource contention với nhiều requests đồng thời (mitigate: rate limiting, task queue)
- **OAuth flow**: Google OAuth thay đổi policy (mitigate: theo dõi docs, test thường xuyên)
- **Encryption overhead**: Encrypt SQLite ảnh hưởng performance (mitigate: benchmark, chỉ encrypt sensitive tables)

### Phase 3 Risks
- **Format preservation**: PPTX/XLSX phức tạp, khó giữ nguyên format 100% (mitigate: best-effort, warn user)
- **PDF OCR**: Tesseract chậm, accuracy thấp với PDF scan kém (mitigate: optional feature, set expectations)

### Phase 4 Risks
- **Mobile audio capture**: iOS/Android giới hạn system audio capture (mitigate: document limitations, mic-only mode)
- **App store approval**: Rejection vì privacy concerns (mitigate: clear privacy policy, request minimal permissions)
- **Diarization accuracy**: Model phụ thuộc audio quality (mitigate: improve VAD, noise reduction)

---

## 8. Definition of Done (từ specs.md section 15)

### Tổng thể
- App chạy ổn định trên Windows/macOS
- Caption hiển thị real-time, latency < 2s
- Offline hoạt động đầy đủ (không cần API cloud)
- Notes + memo tự động cuối buổi
- Dịch file giữ format (DOCX/PPTX/XLSX/PDF text-based)
- Security: key vault + encrypted storage
- GUI + CLI đầy đủ tính năng

### Per-phase Done Criteria

**Phase 1 Done:**
- Audio capture + STT + translation 1 ngôn ngữ hoạt động
- Caption overlay + transcript export
- No crash trong 30-phút meeting test
- Unit tests pass, coverage >60%

**Phase 2 Done:**
- Multi-lang translation (tối thiểu 3 languages đồng thời)
- AI notes + memo export
- Auth + Google SSO hoạt động
- API keys encrypted trong OS keychain

**Phase 3 Done:**
- Dịch DOCX/PPTX/XLSX/PDF giữ format >90%
- CLI chạy được batch translate
- Integration tests pass

**Phase 4 Done:**
- Mobile app sync với desktop
- Speaker diarization accuracy >70%
- Glossary áp dụng đúng
- App submitted to stores (iOS/Android)
- Installer (.msi/.dmg) chạy được, bundle Ollama + models

---

## Ghi Chú

- **Sprint cadence**: 2 tuần/sprint, review cuối mỗi sprint
- **Team size**: Giả định 1-2 developers full-time
- **Adjust timeline**: Timeline có thể kéo dài nếu gặp blocking issues (WASAPI, model performance, etc.)
- **Prioritization**: Phase 1-2 critical, Phase 3-4 nice-to-have (có thể delay nếu cần)
