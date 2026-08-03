---
status: final
created: 2026-07-06
updated: 2026-07-06
sources:
  - ../../prds/prd-WinTick-2026-07-06/prd.md
  - ../mom-2026-07-06-ux-design.md
colors:
  tray_alert: "#E81123"      # Red dot/cross for warnings
typography:
  ui_font: "Segoe UI Variable, Segoe UI, sans-serif"
components:
  tray_icon: System Tray icon representing WinTick presence and state.
  settings_dialog: Native Win32 standard dialog for configuration.
---

# WinTick Visual Design

## Brand & Style
WinTick follows the native Windows 11 design language. The goal is to feel indistinguishable from a first-party Windows background utility. It favors invisibility over presence; the main feature has literally zero UI.

## Colors
WinTick mengandalkan tema native OS pengguna (Light/Dark mode) untuk segala jendela dialog.
Satu-satunya warna custom yang disuntikkan secara statis adalah `{colors.tray_alert}` yang digunakan sebagai *overlay* state ikon System Tray (titik merah untuk log/peringatan, silang merah untuk kegagalan hook).

## Typography
Jendela Settings dan dialog interaktif wajib menggunakan `{typography.ui_font}` secara murni untuk menyatu dengan dialog bawaan Windows.

## Layout & Spacing
Standar Win32 dialog spacing untuk aplikasi terpisah `wintick-settings.exe`. Layout bersifat modular dan dikelompokkan berdasarkan fitur (*Window Cycling*, *Snapping*, *Stack Layout*).

## Components
### System Tray Icon
Jangkar visual utama aplikasi. Menggunakan aset `.ico` minimalis (16x16 / 32x32).

### Settings Window
Sebuah program mandiri yang terpisah (`wintick-settings.exe`) menampilkan layout yang bersih, baik berbasis *tab* maupun grup vertikal. Setiap grup fitur mengandung:
- *Toggle switch* (Enable/Disable).
- *Input field* khusus *shortcut* (menangkap penekanan tombol keyboard secara langsung, bukan mengetik teks).

## Do's and Don'ts
- **Do** jadikan pengalaman pemindahan jendela 100% *invisible* (tak kasat mata, nol animasi, nol overlay).
- **Don't** gunakan framework GUI yang berat (Electron, WPF, React) yang akan merusak target RAM statis < 2MB. UI harus di-decouple atau menggunakan Win32 asli.
- **Do** terapkan *UX Honesty*; jika jendela target sedang *Not Responding*, jangan disembunyikan. Tetap fokuskan jendela tersebut.
