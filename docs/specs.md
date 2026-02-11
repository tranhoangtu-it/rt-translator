# Specification: Real-Time Multilingual Meeting Translator & Notetaker

## 1. Mục tiêu sản phẩm

**Real-Time Meeting Translator & Notetaker** là phần mềm đa nền tảng cho phép:

* **Dịch hội thoại cuộc họp theo thời gian thực** từ audio input/output (không join meeting).
* **Hiển thị live caption + lưu transcript sau họp**.
* **Ghi chú AI liên tục** và **xuất memo/meeting minutes** cuối buổi.
* **Dịch file mọi định dạng** và **giữ nguyên format đầu ra y hệt đầu vào**.

### Mục tiêu tối thượng

* **Độ chính xác cực cao** (mục tiêu 100% là “con số đẹp”, thực tế: tối ưu bằng mô hình + từ điển + hậu xử lý).
* **Độ trễ thấp nhất có thể** (near real-time).
* **Chi phí vận hành ~0$** (ưu tiên chạy local + chỉ dùng cloud khi user tự cấu hình key).
* **Nhỏ – gọn – nhẹ**.

---

## 2. Phạm vi nền tảng

### 2.1 Desktop

* **Windows**: full-feature.
* **macOS**: full-feature.

### 2.2 Mobile

* **iOS / Android**: phiên bản companion (hiển thị caption, xem transcript, note, memo; tuỳ cấu hình cho phép capture audio từ mic để “dịch hội thoại trực tiếp”).

> Lưu ý: Capture **system audio** trên mobile bị giới hạn bởi OS; chiến lược là **companion** + **tether** với desktop, hoặc chỉ capture mic.

---

## 3. Use cases

1. Người dùng tham gia cuộc họp Zoom/Meet/Teams bình thường → app chạy ngoài, thu audio output + mic → hiển thị caption dịch.
2. Sau họp: xuất transcript song ngữ/đa ngữ + memo + action items.
3. Dịch tài liệu: drop file bất kỳ → xuất file cùng định dạng, format giữ nguyên.
4. CLI chạy batch: dịch file hàng loạt, hoặc chạy phiên dịch headless.

---

## 4. Yêu cầu chức năng

### 4.1 Capture audio “ngoài cuộc họp” (không join meeting)

* Thu được **system output audio** (speaker) + **microphone input**.
* Cho phép chọn nguồn audio:

  * Output device (loa/tai nghe)
  * Input device (microphone)
* Có chế độ:

  * **Single stream**: trộn mic+system thành 1 luồng.
  * **Dual stream**: xử lý riêng mic và system (tốt hơn để phân vai người nói).

### 4.2 STT (Speech-to-Text) streaming

* Nhận dạng giọng nói theo thời gian thực.
* Có **voice activity detection (VAD)** để cắt đoạn hợp lý.
* Hỗ trợ đa ngôn ngữ; auto-detect ngôn ngữ nguồn hoặc người dùng set cố định.

### 4.3 Translation real-time

* Dịch STT output sang **n ngôn ngữ đích**.
* Hiển thị caption:

  * 1 ngôn ngữ đích hoặc nhiều ngôn ngữ song song.
  * Có timestamp.
  * Cho phép “refine” bản dịch (dịch nhanh trước, cập nhật lại khi đủ ngữ cảnh).

### 4.4 Live Caption UI + Transcript

* UI overlay always-on-top, resize, transparency.
* Transcript sau họp:

  * timeline theo phút/giây
  * nguyên văn + dịch
  * export: .txt/.md/.docx/.pdf/.json/.csv

### 4.5 AI Note-taking (giống Google Noter)

* AI tạo note liên tục, phân mục:

  * Key points
  * Decisions
  * Risks
  * Action Items (assignee + deadline nếu suy ra được)
* Cho phép user **edit real-time**.
* Cuối buổi tạo **Meeting Memo** (biên bản) + summary nhiều ngôn ngữ.

### 4.6 Document Translation giữ nguyên format

* Input: “file bất kỳ” (ưu tiên support tốt nhất cho: DOCX, PPTX, XLSX, PDF, TXT/MD, HTML, CSV/JSON).
* Output: **cùng định dạng**, **layout/format y hệt**.
* Quy tắc:

  * Chỉ thay text, không phá style.
  * Với PDF scan: OCR + layout reconstruction (best-effort).

### 4.7 Model management (multi-model, bắt buộc Ollama local)

* Bắt buộc có **Ollama local**.
* **Bundle sẵn** runtime + model mặc định để dùng ngay.
* Có setting chọn model theo tác vụ:

  * STT model
  * Translate model
  * Summarize/Note model
* “Phải setting mới sử dụng được”:

  * Cloud providers không hoạt động nếu chưa nhập key.
  * Local providers yêu cầu download/bundle ok.

### 4.8 Kết nối nhiều AI provider

* Provider interface chuẩn hoá để dễ thêm:

  * Ollama (local)
  * OpenAI-compatible
  * Google/DeepL (optional)
  * Custom HTTP endpoint

### 4.9 Auth: đăng ký/đăng nhập + Google SSO

* Email/password + Google OAuth.
* Mục tiêu: tài khoản để sync settings, history (tuỳ chọn).

### 4.10 Key management per machine + security cao

* Mã hoá key tại máy (OS keychain/DPAPI/Keyring).
* Device binding:

  * Mỗi máy có device-id + keypair.
  * Token cấp theo device.
* Dữ liệu họp mặc định lưu local, cloud sync là opt-in.

---

## 5. Non-functional requirements (NFR)

* **Latency**: caption hiển thị < 1–2s (mục tiêu), degrade gracefully.
* **Accuracy**: tối đa hoá bằng model mạnh + glossary + post-edit.
* **Offline-first**: mọi tính năng lõi chạy offline.
* **Resource cap**: giới hạn CPU/RAM theo cấu hình.
* **Privacy**: không gửi audio/text ra ngoài nếu user không bật cloud.
* **Reliability**: tự khôi phục nếu provider lỗi; fallback sang provider khác.

---

## 6. Kiến trúc tổng thể

### 6.1 High-level components

1. **Audio Capture Layer**
2. **Streaming Pipeline Core** (STT → segmentation → translation)
3. **Caption Renderer** (overlay + transcript view)
4. **Note Engine** (summarization + action items)
5. **Document Translation Engine**
6. **Provider Gateway** (Ollama/local + cloud)
7. **Auth & Key Vault**
8. **Storage** (local + optional sync)
9. **GUI app** + **CLI tool**

### 6.2 Data flow

Audio (system+mic)
→ VAD/Chunker
→ STT streaming
→ Text segmentation (sentence/phrase)
→ Translation (multi-target)
→ Caption UI + Transcript store
→ Note engine (incremental)
→ Memo generation end-of-meeting

---

## 7. Module breakdown (bảng)

| Module             | Trách nhiệm              | Input             | Output               | Ghi chú                                       |
| ------------------ | ------------------------ | ----------------- | -------------------- | --------------------------------------------- |
| AudioCapture       | Lấy system audio + mic   | OS devices        | PCM stream           | Windows WASAPI loopback; macOS virtual driver |
| VAD/Segmenter      | Cắt đoạn theo speech     | PCM stream        | audio chunks         | giảm trễ, tăng chất lượng STT                 |
| STT Engine         | Speech-to-text streaming | audio chunks      | partial + final text | Whisper/whisper.cpp/other                     |
| Translation Engine | Dịch multi-lang          | text segments     | translated text      | LLM-based + refine                            |
| Caption Renderer   | Overlay + transcript UI  | translated text   | UI + logs            | always-on-top                                 |
| Note Engine        | Note + summary           | transcript stream | notes + memo         | editable                                      |
| Doc Translator     | Dịch file giữ format     | file              | file                 | per-format pipeline                           |
| Provider Gateway   | Kết nối AI               | requests          | responses            | plugins                                       |
| Auth               | Login/SSO                | creds             | token                | OAuth Google                                  |
| Key Vault          | Lưu key an toàn          | key               | encrypted key        | Keychain/DPAPI                                |
| Storage            | Lưu local/sync           | events            | db/files             | SQLite + file store                           |
| CLI                | Automations              | args              | outputs              | batch & headless                              |

---

## 8. Công nghệ đề xuất (Go hoặc Rust)

### 8.1 Option A: Rust-centric (khuyến nghị)

* **Core + Providers**: Rust
* **GUI**: Tauri (Rust backend + Web UI)
* **Mobile**: Rust core → FFI (Swift/Kotlin) hoặc React Native/Flutter gọi native module
* **STT**: whisper.cpp binding hoặc gọi service local
* **LLM**: Ollama (HTTP) + optional llama.cpp

### 8.2 Option B: Go-centric

* **Core**: Go
* **GUI**: Fyne hoặc Wails
* **Mobile**: companion app (Flutter/ReactNative) + Go backend service optional
* **STT/LLM**: gọi whisper.cpp / ollama qua HTTP

---

## 9. Chiến lược “cost ~ 0$”

* Offline-first: Whisper local + Ollama local.
* Cloud chỉ là optional; user tự nhập key.
* Không bắt buộc server trung tâm.
* Nếu cần sync: hỗ trợ “bring your own storage” (Google Drive/WebDAV/S3 user).

---

## 10. Security design

* **Key storage**: OS secure storage.
* **Data encryption**: encrypt-at-rest (SQLite + file) với master key từ OS vault.
* **Network**: TLS, certificate pinning (tuỳ chọn).
* **RBAC** (enterprise optional).
* **Audit log** (optional).

---

## 11. Packaging & Distribution

* Windows: .msi / .exe installer.
* macOS: .dmg notarized.
* Bundled assets:

  * Ollama runtime
  * Whisper model mặc định (size cân bằng)
  * 1–2 LLM nhỏ cho translate/notes
* Model lớn: tải theo nhu cầu.

---

## 12. CLI design (ví dụ)

* `rt-translator meeting --src auto --tgt vi,ja --overlay on`
* `rt-translator transcript export --format docx --meeting <id>`
* `rt-translator doc translate --in a.pptx --tgt vi --out a_vi.pptx`
* `rt-translator models list | pull | set-default`

---

## 13. Phân tích rủi ro & giới hạn thực tế

* **"100%"**: thực tế khó đạt tuyệt đối; cần:

  * glossary + domain adaptation
  * human-in-the-loop edit
  * confidence score + highlight đoạn nghi ngờ
* System audio capture:

  * macOS cần driver (BlackHole/Loopback) hoặc quyền cao hơn.
  * mobile bị hạn chế.
* Dịch “mọi định dạng” là bài toán rộng: cần roadmap theo format.

---

## 14. Roadmap đề xuất

### Phase 1 (MVP desktop)

* Audio capture (Win/mac)
* STT streaming (Whisper)
* 1 target language translation (Ollama)
* Overlay captions + transcript export

### Phase 2

* Multi-target languages
* Note-taking + memo
* Auth + Google SSO
* Key vault

### Phase 3

* Doc translation: DOCX/PPTX/XLSX/PDF (text-based)
* CLI full

### Phase 4

* Mobile companion
* Advanced: speaker diarization, glossary UI, quality scoring

---

## 15. Định nghĩa “Done”

* Chạy ổn định Win/mac, caption real-time.
* Offline hoạt động đầy đủ (không cần API).
* Notes + memo cuối buổi.
* Dịch file giữ format (ít nhất DOCX/PPTX/XLSX/PDF text-based).
* Security: key vault + encrypted storage.
* GUI + CLI.
