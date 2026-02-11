# Hướng Dẫn Thiết Kế UI/UX

> Real-Time Multilingual Meeting Translator & Notetaker

## 1. Nguyên Tắc Thiết Kế Tổng Quan

### 1.1 Triết Lý Cốt Lõi
- **Tối giản (Minimal)**: Loại bỏ mọi yếu tố không cần thiết, tập trung vào chức năng chính
- **Chức năng trước tiên (Functional)**: UI phục vụ mục đích rõ ràng, không trang trí thừa
- **Không gây xao nhãng (Distraction-free)**: Thiết kế không cản trở công việc người dùng
- **Offline-first visual cues**: Hiển thị rõ ràng trạng thái kết nối và nguồn dữ liệu (local/cloud)

### 1.2 Mục Tiêu Trải Nghiệm
- Người dùng có thể tham gia cuộc họp mà không bị phân tâm bởi UI
- Thông tin dịch xuất hiện kịp thời, dễ đọc, không che khuất nội dung quan trọng
- Điều khiển nhanh, trực quan, không cần đào tạo

## 2. Caption Overlay UI

### 2.1 Đặc Tính Cửa Sổ
- **Always-on-top**: Luôn hiển thị trên mọi cửa sổ ứng dụng khác
- **Transparency**: Điều chỉnh độ trong suốt 0-100% (mặc định 15%)
  - 0% = hoàn toàn trong suốt (chỉ thấy text)
  - 100% = không trong suốt
- **Resizable**: Kéo thay đổi kích thước, tối thiểu 300x100px
- **Draggable**: Kéo thả tự do đến vị trí mong muốn
- **Borderless**: Không viền cửa sổ, góc bo tròn 8px

### 2.2 Typography & Text Display
- **Font size controls**: 12px - 48px (mặc định 18px)
- **Line spacing**: 1.5x cho dễ đọc
- **Font family**: System fonts (Inter cho Windows, SF Pro cho macOS)
- **Text shadow**: Bóng mờ nhẹ để text nổi bật trên nền phức tạp

### 2.3 Multi-language Layout
**Stacked Mode (Xếp chồng)**
```
[Ngôn ngữ gốc]
[Ngôn ngữ dịch 1]
[Ngôn ngữ dịch 2]
```
- Mỗi ngôn ngữ một dòng, font size có thể khác nhau
- Ngôn ngữ chính (target) to hơn, ngôn ngữ phụ nhỏ hơn

**Side-by-side Mode (Song song)**
```
[Ngôn ngữ gốc]  |  [Ngôn ngữ dịch]
```
- Chia đôi màn hình, dấu phân cách mờ

### 2.4 Auto-scroll & Manual Control
- **Auto-scroll**: Tự động cuộn text mới nhất vào view
- **Manual scroll lock**: Click chuột vào caption → lock auto-scroll, hiển thị icon khóa
- Unlock: Click icon hoặc scroll về cuối

### 2.5 Controls Bar (Ẩn/Hiện)
- Hover vào caption → hiển thị thanh control nhỏ (opacity 80%)
- Buttons: Font size ±, Transparency slider, Lock/Unlock, Close
- Keyboard shortcuts: Ctrl+[ giảm font, Ctrl+] tăng font, Ctrl+L lock scroll

## 3. Main Application Window

### 3.1 Layout Tổng Thể
```
┌─────────────────────────────────────┐
│ [Sidebar] │ [Content Area]          │
│           │                         │
│ Meetings  │ ┌─────────────────────┐ │
│ Documents │ │ Transcript View     │ │
│ Settings  │ │                     │ │
│           │ └─────────────────────┘ │
│           │ ┌─────────────────────┐ │
│           │ │ Notes Panel         │ │
│           │ └─────────────────────┘ │
└─────────────────────────────────────┘
```

### 3.2 Sidebar Navigation
- **Width**: 240px (collapsible xuống 64px, chỉ icon)
- **Items**:
  - Meetings (lịch sử + live sessions)
  - Documents (file translation)
  - Settings (cấu hình)
- Icon + label, highlight active item với accent color

### 3.3 Transcript View
- **Timeline**: Thanh timeline dọc bên trái với timestamp (MM:SS)
- **Content**: Transcript theo dòng thời gian, mỗi đoạn có:
  - Speaker label (nếu có diarization)
  - Ngôn ngữ gốc (màu default)
  - Ngôn ngữ dịch (màu accent nhạt)
- **Search**: Ctrl+F tìm kiếm full-text
- **Export**: Button xuất ra .txt/.md/.docx/.pdf/.json/.csv

### 3.4 Notes Panel (Editable)
**Cấu trúc phân mục:**
- **Key Points** (Điểm chính)
- **Decisions** (Quyết định)
- **Risks** (Rủi ro)
- **Action Items** (Công việc cần làm)
  - Checkbox + assignee + deadline

**Tính năng:**
- **Real-time edit**: Click vào bất kỳ đâu để sửa
- **Auto-save**: Lưu tự động mỗi 3s
- **AI suggestions**: Đề xuất nội dung (có thể accept/reject)
- **Export**: Xuất Meeting Memo cuối buổi

## 4. Color Scheme

### 4.1 Light Mode
```
Background:      #FFFFFF (pure white)
Surface:         #F5F5F5 (subtle gray)
Text Primary:    #1A1A1A (near black)
Text Secondary:  #6B6B6B (medium gray)
Border:          #E0E0E0 (light gray)
Accent:          #0066FF (vibrant blue)
Success:         #10B981 (green)
Warning:         #F59E0B (amber)
Error:           #EF4444 (red)
```

### 4.2 Dark Mode
```
Background:      #0A0A0A (near black)
Surface:         #1A1A1A (dark gray)
Text Primary:    #F5F5F5 (near white)
Text Secondary:  #A0A0A0 (light gray)
Border:          #2A2A2A (subtle border)
Accent:          #3B82F6 (blue)
Success:         #10B981 (green)
Warning:         #F59E0B (amber)
Error:           #EF4444 (red)
```

### 4.3 Accent Color Usage
- Active navigation item
- Primary buttons
- Focus states
- Links
- Progress indicators

## 5. Typography

### 5.1 Font Families
- **UI Text**: Inter (Windows), SF Pro (macOS), system-ui fallback
- **Code/Timestamps**: JetBrains Mono, Consolas, monospace fallback
- **Caption Overlay**: System default (performance reason)

### 5.2 Font Scale
```
Heading 1:  28px / bold
Heading 2:  24px / semibold
Heading 3:  20px / semibold
Body Large: 16px / regular
Body:       14px / regular
Body Small: 12px / regular
Caption:    11px / regular
```

## 6. Component Library

### 6.1 Technology Stack
- **shadcn/ui**: Headless component library (không theme mặc định, custom dễ dàng)
- **Tailwind CSS**: Utility-first CSS framework
- **Radix UI**: Primitives cho accessibility

### 6.2 Core Components
- Button (Primary, Secondary, Ghost, Destructive)
- Input (Text, Number, Search)
- Select / Dropdown
- Checkbox / Toggle
- Slider (cho volume, transparency)
- Dialog / Modal
- Toast notifications
- Tabs
- Tooltip

## 7. Accessibility (WCAG AA)

### 7.1 Contrast Ratios
- Text normal: ≥4.5:1
- Text large (≥18px): ≥3:1
- UI components: ≥3:1

### 7.2 Keyboard Navigation
- Tab order hợp lý
- Focus visible rõ ràng (outline accent color)
- Shortcuts: Không conflict với OS/browser defaults

### 7.3 Screen Reader Support
- ARIA labels cho tất cả controls
- Live regions cho caption updates
- Semantic HTML (nav, main, aside, article)

### 7.4 Motion & Animation
- Respect `prefers-reduced-motion`
- Animations ≤300ms
- Disable parallax/complex effects khi user bật reduced motion

## 8. Window Sizes & Responsive Breakpoints

### 8.1 Minimum Sizes
- **Main window**: 800x600px
- **Caption overlay**: 300x100px

### 8.2 Recommended Sizes
- **Main window**: 1200x800px (default)
- **Caption overlay**: 600x120px (1 language), 600x200px (multi-language)

### 8.3 Responsive Rules
- Sidebar collapsible dưới 1024px width
- Notes panel stack dưới transcript view khi height < 600px
- Font sizes scale down 10% ở breakpoint nhỏ nhất
