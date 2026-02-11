# Hướng Dẫn Build & Distribution

> Real-Time Multilingual Meeting Translator & Notetaker

## 1. Prerequisites

### Build Tools
| Tool | Version | Mục đích |
|------|---------|----------|
| Rust | 1.80+ | Core backend |
| Node.js | 20+ | Frontend build |
| npm | 10+ | Package manager |
| Tauri CLI | 2.x | Desktop bundler |

### Platform-specific
**Windows:**
- Visual Studio 2022 Build Tools (C++ workload)
- WiX Toolset 3.x (cho .msi installer)

**macOS:**
- Xcode Command Line Tools
- Apple Developer certificate (cho code signing)

## 2. Development Build

```bash
# Cài đặt dependencies
npm install

# Cài đặt Tauri CLI
cargo install tauri-cli

# Chạy development mode (hot reload)
cargo tauri dev

# Chỉ build frontend
npm run dev

# Chỉ build backend
cd src-tauri && cargo build
```

## 3. Production Build

```bash
# Build production bundle
cargo tauri build

# Output locations:
# Windows: src-tauri/target/release/bundle/msi/*.msi
# Windows: src-tauri/target/release/bundle/nsis/*.exe
# macOS:   src-tauri/target/release/bundle/dmg/*.dmg
# macOS:   src-tauri/target/release/bundle/macos/*.app
```

### Build Flags
```bash
# Release build with optimizations
cargo tauri build --release

# Debug build
cargo tauri build --debug

# Specific target
cargo tauri build --target x86_64-pc-windows-msvc
```

## 4. Windows Packaging (.msi / .exe)

### Tauri Bundler Config (`tauri.conf.json`)
```json
{
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "identifier": "com.rt-translator.app",
    "icon": ["icons/icon.ico"],
    "windows": {
      "wix": {
        "language": "vi-VN"
      }
    }
  }
}
```

### Code Signing (Optional)
- Certificate: `.pfx` file từ CA hoặc self-signed
- Sign command: `signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com app.exe`

## 5. macOS Packaging (.dmg)

### Code Signing & Notarization
```bash
# Sign app
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  target/release/bundle/macos/RT\ Translator.app

# Notarize
xcrun notarytool submit rt-translator.dmg \
  --apple-id your@email.com \
  --team-id TEAM_ID \
  --password @keychain:AC_PASSWORD
```

## 6. Bundled Assets Strategy

### Ollama Runtime
- **Option A**: Yêu cầu user cài Ollama riêng (recommended cho MVP)
- **Option B**: Embed Ollama binary trong installer (tăng size ~200MB)
- App detect Ollama tại `localhost:11434`, hiển thị setup guide nếu chưa có

### Whisper Model
- Bundle model mặc định: `ggml-base.en.bin` (~150MB) hoặc `ggml-small.bin` (~500MB)
- Lưu tại: `%APPDATA%/rt-translator/models/` (Win) hoặc `~/Library/Application Support/rt-translator/models/` (mac)
- Download thêm model: qua Settings UI hoặc CLI `rt-translator models pull`

### LLM Models (Ollama)
- Không bundle (quá lớn, 2-7GB/model)
- First-run wizard: hướng dẫn user pull model (`ollama pull qwen2.5:3b`)
- Recommend models: `qwen2.5:3b` (translation), `llama3.2:3b` (notes)

## 7. CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/build.yml (placeholder)
name: Build & Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        os: [windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm install
      - run: cargo tauri build
      - uses: actions/upload-artifact@v4
        with:
          name: bundle-${{ matrix.os }}
          path: src-tauri/target/release/bundle/
```

## 8. Model Management

### Cache Locations
| OS | Path |
|----|------|
| Windows | `%APPDATA%\rt-translator\models\` |
| macOS | `~/Library/Application Support/rt-translator/models/` |

### Download & Update
```bash
# CLI commands
rt-translator models list              # Liệt kê models installed
rt-translator models pull whisper:base # Tải Whisper model
rt-translator models set-default stt whisper:base
```

### Checksum Verification
- Mỗi model download kèm SHA256 checksum
- App verify trước khi load model
- Corrupt model → re-download tự động

## 9. Environment Variables

| Variable | Default | Mô tả |
|----------|---------|-------|
| `RT_OLLAMA_URL` | `http://localhost:11434` | Ollama API endpoint |
| `RT_LOG_LEVEL` | `info` | Logging level (trace/debug/info/warn/error) |
| `RT_DATA_DIR` | OS default | Custom data directory |
| `RT_MODEL_DIR` | OS default | Custom model directory |

---

> **Lưu ý**: Document này sẽ được cập nhật chi tiết hơn khi Tauri project được khởi tạo và build pipeline hoạt động.
