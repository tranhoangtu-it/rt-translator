# Product Development Requirements (PDR)

## 1. Tổng Quan Sản Phẩm

**RT Translator** là phần mềm đa nền tảng (Desktop Windows/macOS + Mobile companion) giúp:

- **Dịch hội thoại cuộc họp theo thời gian thực** từ audio input/output mà không cần join meeting
- **Hiển thị live caption + lưu transcript** song ngữ/đa ngữ sau họp
- **Ghi chú AI liên tục** và xuất memo/meeting minutes cuối buổi
- **Dịch file mọi định dạng** (DOCX/PPTX/XLSX/PDF/TXT/MD/HTML/CSV/JSON) với format giữ nguyên hoàn toàn

### Mục Tiêu Tối Thượng

1. **Độ chính xác cực cao**: Mục tiêu tối ưu bằng model mạnh + glossary + human-in-the-loop edit
2. **Độ trễ thấp nhất**: Caption hiển thị <1-2s (near real-time)
3. **Chi phí vận hành ~0$**: Offline-first với Whisper local + Ollama local, cloud chỉ optional
4. **Nhỏ – gọn – nhẹ**: Desktop app không phụ thuộc heavy infrastructure

## 2. Đối Tượng Người Dùng

### Personas

**Persona 1: Remote Worker**
- **Role**: Developer, Designer, PM làm việc từ xa
- **Pain Points**: Tham gia meeting đa quốc gia, khó theo kịp khi native language khác English
- **Needs**: Real-time caption Vietnamese/English, transcript để review sau, notes tự động
- **Tech Savvy**: Cao, sẵn sàng cài tools local

**Persona 2: Enterprise Team Lead**
- **Role**: Quản lý team quốc tế
- **Pain Points**: Cần meeting minutes chuẩn, action items tracking, nhưng không muốn share audio lên cloud vì compliance
- **Needs**: Offline translation + secure storage, export memo professional format
- **Tech Savvy**: Trung bình, cần UI đơn giản

**Persona 3: Content Translator**
- **Role**: Freelancer dịch tài liệu
- **Pain Points**: Dịch DOCX/PPTX mất format, phải re-layout thủ công rất lâu
- **Needs**: Batch translate documents giữ nguyên format 100%, support nhiều định dạng
- **Tech Savvy**: Trung bình, ưu tiên workflow nhanh

**Persona 4: Student/Researcher**
- **Role**: Học sinh/sinh viên tham gia online lectures/seminars
- **Pain Points**: Giảng viên nói nhanh, accent khó hiểu, cần transcript để ôn tập
- **Needs**: Free tool, chạy offline, caption real-time + transcript export
- **Tech Savvy**: Cao, nhạy cảm về giá

## 3. Use Cases

### UC1: Real-time Meeting Translation
**Actor**: Remote Worker
**Flow**:
1. User tham gia Zoom/Meet/Teams meeting bình thường
2. Mở RT Translator, chọn source language (auto-detect) và target languages (vi, ja)
3. App capture system audio (speaker) + mic input
4. Hiển thị live caption overlay trên màn hình, có thể resize/move
5. Sau meeting: export transcript song ngữ (original + translated) dạng DOCX/PDF

**Success Criteria**: Caption delay <2s, transcript chính xác >95%, UI không che khuất meeting window

### UC2: AI Meeting Notes & Memo
**Actor**: Enterprise Team Lead
**Flow**:
1. Start meeting session trong RT Translator
2. AI tự động ghi note theo sections: Key Points, Decisions, Risks, Action Items
3. User có thể edit notes real-time nếu cần điều chỉnh
4. Cuối meeting: Generate meeting memo với summary đa ngôn ngữ + action items table
5. Export memo dạng DOCX/PDF, email cho team

**Success Criteria**: Notes coverage >80% nội dung quan trọng, action items có assignee/deadline nếu mentioned

### UC3: Document Translation Format-preserving
**Actor**: Content Translator
**Flow**:
1. Drag & drop file PPTX vào RT Translator
2. Chọn target language (Vietnamese)
3. App dịch text trong slides, giữ nguyên layout/fonts/images/animations
4. Preview before export, cho phép edit manual nếu cần
5. Export file PPTX mới với tên `original_vi.pptx`

**Success Criteria**: Format preservation 100%, không phá style/layout, hỗ trợ DOCX/PPTX/XLSX/PDF (text-based)

### UC4: CLI Batch Processing
**Actor**: Content Translator (batch workflow)
**Flow**:
```bash
rt-translator doc translate --in-dir ./docs --tgt vi,ja --out-dir ./translated --format pptx,docx
```
Dịch toàn bộ folder documents sang 2 ngôn ngữ, headless mode

**Success Criteria**: CLI stable, progress reporting, error handling graceful

## 4. Phạm Vi

### In-Scope (MVP)

**Phase 1 - Desktop Core**:
- Audio capture (Windows WASAPI loopback + macOS CoreAudio)
- STT streaming (Whisper local model)
- Translation 1 target language (Ollama local LLM)
- Live caption overlay UI (always-on-top, transparent)
- Transcript export (.txt, .md)

**Phase 2 - Enhanced Features**:
- Multi-target translation (n languages simultaneously)
- AI note-taking + memo generation
- Auth (email/password + Google SSO)
- Key vault (OS keychain integration)
- Settings UI (model selection, device config)

**Phase 3 - Document Translation**:
- DOCX/PPTX/XLSX/PDF (text-based) format-preserving translation
- Batch processing via CLI
- Document preview + manual edit before export

### Out-of-Scope (Post-MVP)

- **Mobile App**: iOS/Android companion (Phase 4)
- **Speaker Diarization**: Phân biệt người nói trong dual stream (Phase 4)
- **Glossary Management UI**: Custom terminology dictionary (Phase 4)
- **Quality Scoring**: Confidence score + highlight low-confidence segments (Phase 4)
- **PDF OCR**: Dịch PDF scan với layout reconstruction (Phase 5)
- **Video Subtitle**: Generate SRT/VTT từ video files (Future)
- **Cloud Sync**: Optional backup/sync qua user's cloud storage (Future)

### MVP vs Full Product

| Feature | MVP (Phase 1-2) | Full Product |
|---------|----------------|--------------|
| Desktop (Win/mac) | ✅ Full | ✅ Full |
| Mobile companion | ❌ | ✅ |
| Offline translation | ✅ Single target | ✅ Multi-target |
| Notes + memo | ✅ Basic | ✅ + Speaker diarization |
| Doc translation | ❌ | ✅ DOCX/PPTX/XLSX/PDF/HTML |
| CLI | ✅ Basic commands | ✅ Full automation |
| Cloud providers | ✅ Manual key config | ✅ + Managed fallback |

## 5. Chỉ Số Thành Công

### Performance Metrics

- **Latency**: Caption hiển thị <2s sau khi audio input (mục tiêu <1s cho English)
- **Accuracy**: Transcript WER (Word Error Rate) <10% cho English, <15% cho Vietnamese
- **Translation Quality**: BLEU score >40 (tương đương human translation baseline)
- **Resource Usage**: CPU <30% average, RAM <2GB (không tính model size)
- **Offline-first**: 100% core features hoạt động không cần internet

### Business Metrics

- **Cost**: Chi phí vận hành ~0$ (không dùng cloud API mặc định)
- **Adoption**: User retention >70% sau 1 tháng (MVP target)
- **NPS**: Net Promoter Score >50 (user satisfaction)
- **Time-to-value**: User có thể bắt đầu dịch meeting trong <5 phút sau cài đặt

### Quality Metrics

- **Stability**: Crash-free rate >99.5%
- **Security**: Không có data leak incidents, pass security audit
- **Format Preservation**: Document translation format match >95% (visual QA)

## 6. Dependencies & Constraints

### Technical Dependencies

- **Ollama**: Bắt buộc cài sẵn và chạy service tại `localhost:11434`
- **Whisper Model**: Bundle model mặc định (base/small), hoặc user tải model lớn theo nhu cầu
- **OS APIs**: WASAPI (Windows), CoreAudio (macOS) cho audio loopback capture
- **Rust Toolchain**: rustc 1.80+, cargo, build tools (MSVC/Xcode)
- **Node.js**: v20+ cho frontend build (dev dependency)

### Platform Constraints

- **Windows**: WASAPI loopback capture cần quyền admin lần đầu (setup audio driver)
- **macOS**: CoreAudio virtual device (có thể cần cài BlackHole/Loopback) cho system audio capture
- **Mobile**: System audio capture bị OS hạn chế → chỉ capture mic, hoặc companion mode tether với desktop

### Regulatory Constraints

- **Privacy**: Không gửi audio/text ra ngoài nếu user không bật cloud providers (GDPR-compliant design)
- **Data Residency**: Dữ liệu mặc định lưu local, cloud sync là opt-in
- **Licensing**: Whisper model (MIT), Ollama (MIT), các thư viện Rust/Node open-source compatible

### Resource Constraints

- **Team Size**: Nhỏ (1-3 developers) → ưu tiên MVP scope hẹp, extend sau
- **Timeline**: MVP target 3-4 tháng (Phase 1-2)
- **Budget**: ~0$ infrastructure cost → không dùng paid cloud services cho core features

## 7. Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| WASAPI capture không stable | High | Medium | Fallback sang virtual audio cable, cung cấp setup guide chi tiết |
| Whisper latency cao | High | Medium | Dùng streaming variant (whisper-stream-rs), optimize chunking strategy |
| Ollama model chất lượng không đủ | High | Low | Cho phép switch sang cloud providers (OpenAI/DeepL) với user's key |
| Format preservation không perfect | Medium | High | Per-format pipeline test suite, human QA trước release |
| Audio sync drift (mic vs system) | Medium | Medium | Timestamp-based sync + periodic re-calibration |

## 8. Tài Liệu Chi Tiết

Xem **[specs.md](./specs.md)** cho:
- Yêu cầu chức năng đầy đủ (sections 4.1-4.10)
- Module breakdown chi tiết (section 7)
- Kiến trúc tổng thể (section 6)
- Security design (section 10)
- Roadmap phases (section 14)

Xem **[system-architecture.md](./system-architecture.md)** cho:
- Tech stack mapping (section 4)
- Component diagram + data flow (sections 2-3)
- IPC design (section 5)
- Directory structure (section 7)
