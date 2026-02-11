# Chuẩn Mã Nguồn (Code Standards)

> Real-Time Multilingual Meeting Translator & Notetaker

## 1. Quy Tắc Rust

### Edition & Linting
```rust
// Cargo.toml
[package]
edition = "2021"

// src/lib.rs hoặc src/main.rs
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
```

### Naming Conventions
- **Files/modules**: `snake_case` (e.g., `audio_capture.rs`, `stt_engine.rs`)
- **Types/structs/enums**: `PascalCase` (e.g., `AudioCapture`, `TranslationEngine`)
- **Functions/variables**: `snake_case` (e.g., `start_meeting()`, `audio_buffer`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_BUFFER_SIZE`)

### Error Handling
- **Library code**: Dùng `thiserror` cho custom errors
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Capture failed")]
    CaptureFailed(#[from] cpal::BuildStreamError),
}
```

- **Binary/application code**: Dùng `anyhow` cho error propagation
```rust
use anyhow::{Context, Result};

fn start_capture() -> Result<()> {
    let device = get_device().context("Failed to get audio device")?;
    Ok(())
}
```

### Structured Logging
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(audio_data))]
async fn process_audio(audio_data: &[f32]) -> Result<String> {
    debug!("Processing {} samples", audio_data.len());
    info!("STT completed");
    Ok(transcript)
}
```

## 2. Quy Tắc TypeScript/React

### TypeScript Configuration
```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

### Component Standards
- **Chỉ dùng functional components**, không class components
- **Hooks**: Tuân theo Rules of Hooks (eslint-plugin-react-hooks)
```tsx
// Good
const CaptionOverlay: React.FC<Props> = ({ text, lang }) => {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    // side effects
  }, []);

  return <div>{text}</div>;
};

// Bad - không dùng class components
class CaptionOverlay extends React.Component { }
```

### State Management
- **Zustand** cho global state
```ts
// src/stores/meeting-store.ts
import { create } from 'zustand';

interface MeetingState {
  isActive: boolean;
  captions: Caption[];
  startMeeting: () => void;
  stopMeeting: () => void;
}

export const useMeetingStore = create<MeetingState>((set) => ({
  isActive: false,
  captions: [],
  startMeeting: () => set({ isActive: true }),
  stopMeeting: () => set({ isActive: false }),
}));
```

### ESLint + Prettier
```json
// .eslintrc.json
{
  "extends": ["react-app", "prettier"],
  "rules": {
    "no-console": "warn",
    "prefer-const": "error"
  }
}
```

## 3. Tổ chức File

### File Naming
- **kebab-case** cho tất cả files
- **Tên mô tả rõ ràng**, dài cũng được (để LLM tools dễ hiểu)
```
✓ Good:
  - audio-capture-wasapi.rs
  - translation-pipeline-streaming.ts
  - caption-overlay-component.tsx

✗ Bad:
  - audioCap.rs (camelCase)
  - trans.ts (tên ngắn không rõ nghĩa)
```

### File Size Limit
- **Max 200 lines/file** cho code files
- Nếu vượt quá: tách thành modules nhỏ hơn
```
// Before (250 lines)
audio_capture.rs

// After (modularized)
audio/
├── mod.rs           (50 lines - re-exports)
├── capture.rs       (100 lines - capture logic)
├── device.rs        (80 lines - device enumeration)
└── format.rs        (70 lines - format conversion)
```

### Module Structure
```
src-tauri/src/
├── main.rs                 # Entry point only
├── lib.rs                  # Public API exports
├── commands/               # Tauri IPC
│   ├── mod.rs
│   ├── meeting.rs
│   └── document.rs
└── audio/                  # Domain modules
    ├── mod.rs
    ├── capture.rs
    └── vad.rs
```

## 4. Git Workflow

### Conventional Commits
```bash
# Format: <type>(<scope>): <subject>

feat(audio): add WASAPI loopback capture
fix(stt): resolve whisper-rs memory leak
docs(readme): update installation steps
refactor(translation): extract provider gateway trait
test(audio): add VAD unit tests
chore(deps): bump tauri to v2.1.0
```

### Branching Strategy
- **Trunk-based**: `main` + short-lived feature branches
```bash
main                    # Always deployable
├── feature/audio-capture
├── feature/ollama-integration
└── hotfix/stt-crash
```

### Commit Rules
- Atomic commits (1 concern/commit)
- Không commit files nhạy cảm (.env, credentials)
- Pre-commit: format + lint check
- Pre-push: test suite pass

## 5. Testing

### Rust Tests
```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_capture_init() {
        let capture = AudioCapture::new();
        assert!(capture.is_ok());
    }
}

// Integration tests (tests/ directory)
#[tokio::test]
async fn test_full_stt_pipeline() {
    let audio = load_test_audio();
    let result = stt_engine.process(audio).await;
    assert_eq!(result.unwrap(), "expected transcript");
}
```

### React Tests (Vitest + Testing Library)
```tsx
// src/components/caption-overlay/__tests__/caption-overlay.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { CaptionOverlay } from '../caption-overlay';

describe('CaptionOverlay', () => {
  it('renders caption text', () => {
    render(<CaptionOverlay text="Hello" lang="en" />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

## 6. Error Handling Patterns

### Rust Result Chaining
```rust
// Good - chain với ? operator
fn process() -> Result<String, AudioError> {
    let device = get_device()?;
    let stream = build_stream(&device)?;
    let data = capture_audio(stream)?;
    Ok(format_output(data))
}

// Bad - unwrap() trong production code
fn process() -> String {
    let device = get_device().unwrap(); // ✗ panic risk
    // ...
}
```

### Structured Errors
```rust
// Custom error với context
#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Provider {provider} unavailable")]
    ProviderUnavailable { provider: String },

    #[error("Translation timeout after {seconds}s")]
    Timeout { seconds: u64 },
}
```

### Logging Levels
```rust
error!("Critical failure: {}", err);    // Recoverable errors
warn!("API rate limit reached");        // Warnings
info!("Meeting started: {}", id);       // Important events
debug!("Processing chunk {}", idx);     // Development info
trace!("Raw audio sample: {:?}", buf);  // Verbose details
```

## 7. Code Review Checklist

### Rust Code
- [ ] Không có `unwrap()`, `expect()` trong production paths
- [ ] Error types implement `std::error::Error`
- [ ] Public APIs có doc comments (`///`)
- [ ] Async functions return `Result<T, E>`
- [ ] Clippy warnings đã resolved

### TypeScript/React
- [ ] Props có TypeScript types rõ ràng
- [ ] Không mutation trực tiếp state
- [ ] useEffect có dependency array đúng
- [ ] Event handlers được memoize (useCallback)
- [ ] Components có display name

### General
- [ ] Commit message theo conventional format
- [ ] Tests coverage cho code mới
- [ ] Không commit sensitive files
- [ ] Code format đã chạy (rustfmt/prettier)
- [ ] Build success (cargo build/npm run build)

---

**Nguyên tắc vàng**: YAGNI (You Aren't Gonna Need It) - KISS (Keep It Simple) - DRY (Don't Repeat Yourself)
