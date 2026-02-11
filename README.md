# RT Translator

**Real-Time Multilingual Meeting Translator & Notetaker** - phần mềm desktop dịch hội thoại cuộc họp theo thời gian thực, tạo transcript đa ngữ, ghi chú AI tự động. Thiết kế offline-first với chi phí vận hành ~0$ bằng các mô hình AI chạy hoàn toàn local.

## Tính Năng Hiện Tại (Phase 2 - Sprint 7 Complete)

### ✅ Đã Hoàn Thành
- **Audio Capture**: Thu âm mic + system audio (Windows WASAPI, macOS CoreAudio)
- **Speech-to-Text**: Whisper.cpp local với Voice Activity Detection (VAD)
- **Multi-target Translation**: Dịch đồng thời tới 4 ngôn ngữ (Ollama LLM)
- **Live Caption Overlay**: Cửa sổ overlay always-on-top, transparent, multi-language display
- **Transcript Timeline**: Hiển thị transcript với tabbed view theo ngôn ngữ
- **AI Note-taking**: Tự động tạo notes theo 4 categories (Key Points, Decisions, Risks, Action Items)
- **Meeting Memo**: Xuất memo markdown với summary và action items
- **Export**: Transcript export .txt/.md/.json với đa ngôn ngữ
- **Dark Mode**: UI theme toggle
- **Settings Panel**: Multi-language selector (max 4), font size, overlay toggle

### 🚧 Đang Phát Triển (Phase 2 Remaining)
- Document translation (DOCX/PPTX/XLSX/PDF format-preserving)
- Batch document processing CLI
- Multi-provider support (OpenAI, DeepL, custom endpoints)

### 📋 Kế Hoạch (Phase 3-4)
- Authentication (email/password + Google OAuth)
- Security (OS keychain, data encryption)
- Speaker diarization
- Glossary management
- Mobile companion app

## Tech Stack

| Layer | Công Nghệ |
|-------|-----------|
| **Desktop Framework** | Tauri v2 (Rust + React webview) |
| **Backend** | Rust 2021 + tokio async runtime |
| **Frontend** | React 19 + TypeScript + Tailwind CSS v4 |
| **UI Components** | shadcn/ui + Radix UI |
| **State Management** | zustand |
| **Speech-to-Text** | whisper-rs (whisper.cpp binding) |
| **Translation/Notes** | Ollama HTTP API (localhost:11434) |
| **Audio Capture** | cpal + WASAPI (Windows) / CoreAudio (macOS) |
| **Audio Processing** | rubato (resampling), custom VAD (energy-based) |
| **Database** | rusqlite (SQLite with WAL mode) |
| **HTTP Client** | reqwest (streaming support) |

## Prerequisites

### Bắt Buộc
- **Rust Toolchain**: rustc 1.80+ và cargo → [rustup.rs](https://rustup.rs)
- **Node.js**: v20+ và npm 10+
- **Ollama**: Cài và chạy service tại `localhost:11434` → [ollama.com](https://ollama.com)
  - Pull model translation: `ollama pull llama3.2:3b`
- **Windows**: Visual Studio 2022 Build Tools (C++ workload)
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)

### Model Files
- **Whisper model**: Auto-download từ HuggingFace CDN khi chạy lần đầu
- **LLM model**: Cần pull manual qua Ollama CLI (ví dụ: `llama3.2:3b`)

### Khuyến Nghị
- RAM: 8GB+ (16GB recommended cho models lớn)
- CPU: Multi-core (Intel i5/AMD Ryzen 5+)
- GPU: Optional (CUDA/Metal cho Whisper acceleration)

## Cài Đặt & Chạy

### Development Mode

```bash
# Clone repository
git clone <repository-url>
cd Translator

# Cài đặt dependencies frontend
npm install

# Chạy Tauri dev
npm run tauri dev
# hoặc
cargo tauri dev
```

### Production Build

```bash
# Build production bundle
npm run tauri build

# Output:
# Windows: src-tauri/target/release/bundle/msi/
# macOS: src-tauri/target/release/bundle/dmg/
```

### Quick Start

1. Khởi động Ollama service và pull model:
   ```bash
   ollama pull llama3.2:3b
   ```

2. Chạy RT Translator:
   ```bash
   npm run tauri dev
   ```

3. Click "Start Meeting" → app sẽ tự download Whisper model lần đầu

4. Chọn target languages trong Settings panel (max 4)

5. Live caption xuất hiện trong overlay window, transcript hiển thị trong main window

## Cấu Trúc Thư Mục

```
rt-translator/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs            # Tauri app entry point
│   │   ├── lib.rs             # Library exports
│   │   ├── commands/          # Tauri IPC commands
│   │   │   ├── audio.rs       # Audio device & capture commands
│   │   │   ├── stt.rs         # STT model & meeting commands
│   │   │   ├── translation.rs # Multi-lang translation commands
│   │   │   ├── export.rs      # Transcript export commands
│   │   │   ├── overlay.rs     # Overlay window commands
│   │   │   └── notes.rs       # Note CRUD & memo commands
│   │   ├── audio/             # Audio capture engine
│   │   │   ├── capture.rs     # Dual-stream capture manager
│   │   │   ├── device.rs      # Device enumeration
│   │   │   ├── resampler.rs   # Audio resampling (rubato)
│   │   │   └── vad.rs         # Voice Activity Detection
│   │   ├── stt/               # Speech-to-text engine
│   │   │   ├── whisper.rs     # whisper-rs wrapper
│   │   │   ├── model_manager.rs # HuggingFace download
│   │   │   └── pipeline.rs    # STT orchestrator
│   │   ├── translation/       # Translation pipeline
│   │   │   └── pipeline.rs    # Ollama-based translation
│   │   ├── notes/             # AI note-taking engine
│   │   │   ├── note_engine.rs # Incremental summarizer
│   │   │   ├── prompt_templates.rs # LLM prompts
│   │   │   └── memo_builder.rs # Memo formatter
│   │   ├── providers/         # LLM provider abstraction
│   │   │   ├── traits.rs      # Provider trait (RPITIT)
│   │   │   └── ollama.rs      # Ollama HTTP client
│   │   └── storage/           # SQLite persistence
│   │       ├── migrations.rs  # Database schema (V1-V4)
│   │       ├── transcript_store.rs # Transcript CRUD
│   │       └── note_store.rs  # Note CRUD
│   └── Cargo.toml
├── src/                       # React frontend
│   ├── main.tsx               # Main app entry
│   ├── overlay/
│   │   ├── overlay.tsx        # Overlay window entry
│   │   └── overlay.html       # Overlay HTML template
│   ├── components/
│   │   ├── app-layout.tsx     # Main app layout
│   │   ├── audio-device-selector.tsx # Device UI
│   │   ├── settings-panel.tsx # Settings (lang, font, overlay)
│   │   ├── transcript-timeline.tsx # Tabbed transcript view
│   │   ├── notes-panel.tsx    # 4-tab notes UI
│   │   ├── note-card.tsx      # Note card component
│   │   ├── export-button.tsx  # Transcript export UI
│   │   ├── memo-export-button.tsx # Memo export UI
│   │   └── ui/                # shadcn/ui components
│   ├── hooks/
│   │   ├── use-audio-capture.ts # Audio capture hook
│   │   ├── use-stt-events.ts  # STT event listener
│   │   ├── use-translation-events.ts # Translation event listener
│   │   ├── use-auto-translation.ts # Auto-translate hook
│   │   └── use-note-events.ts # Note event listener
│   ├── stores/
│   │   └── app-store.ts       # zustand state management
│   └── types/
│       ├── index.ts           # Shared TypeScript types
│       └── notes.ts           # Note types
├── docs/                      # Documentation
│   ├── project-overview-pdr.md # Product requirements
│   ├── system-architecture.md # System design
│   ├── code-standards.md      # Coding conventions
│   ├── codebase-summary.md    # Module index
│   ├── development-roadmap.md # Phase/sprint tracking
│   └── design-guidelines.md   # UI/UX guidelines
├── plans/                     # Implementation plans
└── package.json
```

## Tài Liệu

- **[Product Overview](docs/project-overview-pdr.md)**: Tổng quan sản phẩm, use cases, success metrics
- **[System Architecture](docs/system-architecture.md)**: Tech stack, component design, data flow
- **[Codebase Summary](docs/codebase-summary.md)**: Module index, file structure, dependencies
- **[Code Standards](docs/code-standards.md)**: Coding conventions, best practices
- **[Development Roadmap](docs/development-roadmap.md)**: Progress tracking (Sprint 1-7 complete, ~40% Phase 2)
- **[Design Guidelines](docs/design-guidelines.md)**: UI/UX patterns, caption overlay specs

## Development Workflow

### Before Commit
```bash
# Run linting
cargo fmt
cargo clippy

# Type check frontend
npm run build
```

### Testing
```bash
# Backend tests
cargo test

# Frontend (TBD)
npm test
```

### Commit Format
Sử dụng Conventional Commits:
```
feat: add multi-language transcript export
fix: resolve overlay window race condition
docs: update system architecture diagram
refactor: extract NoteCard component
```

## Trạng Thái Phát Triển

### Current: Phase 2 (~40% complete)
- ✅ Sprint 1: Project scaffold + Tauri setup
- ✅ Sprint 2: Audio capture (mic + system loopback)
- ✅ Sprint 3: STT pipeline (Whisper + VAD)
- ✅ Sprint 4: Translation engine (Ollama integration)
- ✅ Sprint 5: Caption overlay + transcript export
- ✅ Sprint 6: Multi-target language support
- ✅ Sprint 7: AI note-taking + memo export
- 🚧 Sprint 8-12: Document translation, multi-provider, CLI

### Roadmap
- **Phase 1 (Complete)**: Desktop core with single-lang translation
- **Phase 2 (In Progress)**: Multi-lang, notes, document translation
- **Phase 3 (Planned)**: Auth, security, glossary
- **Phase 4 (Planned)**: Speaker diarization, mobile companion

Xem chi tiết trong [development-roadmap.md](docs/development-roadmap.md)

## Đóng Góp

Chúng tôi hoan nghênh mọi đóng góp! Vui lòng:

1. Fork repository và tạo branch mới: `git checkout -b feature/amazing-feature`
2. Tuân thủ coding standards trong `docs/code-standards.md`
3. Viết tests cho mọi tính năng mới
4. Commit với Conventional Commit format
5. Push branch và tạo Pull Request

**Quy tắc quan trọng:**
- Đọc `.claude/rules/development-rules.md` trước khi bắt đầu
- Chạy `cargo fmt` và `cargo clippy` trước commit
- Không commit sensitive data (.env, API keys, credentials)
- Đảm bảo tất cả tests pass

## Giấy Phép

[Chưa xác định - sẽ cập nhật sau khi project owner quyết định]

---

**Status**: Early development, features có thể thay đổi. Sprint 7 (AI Notes) hoàn thành 2026-02-11.

**Contact**: Xem issues/discussions trong repository để trao đổi với team.
