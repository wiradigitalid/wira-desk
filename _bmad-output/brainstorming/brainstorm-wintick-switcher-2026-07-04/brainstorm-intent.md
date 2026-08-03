# WinTick — Brainstorm Intent

**Tujuan Dokumen**: Input ringkas hasil sesi curah pendapat untuk diumpankan ke `bmad-prd`, `bmad-spec`, atau `bmad-create-epics-and-stories` sebagai fondasi pengembangan.

---

## Pernyataan Masalah
Pengguna macOS yang berpindah ke Windows 11 kehilangan satu fitur produktivitas kritis: shortcut `Command+~` yang memungkinkan berpindah antar jendela dari **aplikasi yang sama** (bukan antar aplikasi). Windows 11 tidak memiliki padanan bawaan untuk ini. Satu-satunya alternatif komersial yang ada (NeoSmart EasySwitch) sudah tidak aktif dikembangkan sejak 2022.

## Proposisi Nilai
Sebuah aplikasi latar belakang Windows 11 berukuran sangat kecil (< 10MB RAM) yang:
1. Mengadopsi perilaku `Command+~` macOS secara tepat dan andal.
2. Berjalan otomatis sejak Windows dihidupkan dan dapat dipantau dari system tray.
3. Menyediakan fitur snap dan manajemen jendela yang dapat dikustomisasi.

---

## Target Pengguna
- Pengguna macOS yang bermigrasi ke Windows 11 (profesional, developer, desainer).
- Pengguna multi-monitor dengan setup DPI berbeda.
- Pengguna yang mengelola banyak jendela dari satu aplikasi (misal: multi-jendela browser, aplikasi chat).

---

## Keputusan Teknis (Settled)
| Aspek | Keputusan |
| :--- | :--- |
| Bahasa Pemrograman | **Rust** (via crate `windows`) |
| Target RAM | < 10MB (aspirasi < 1MB, maks toleransi 20MB) |
| Target CPU | ~0% saat idle |
| Format Konfigurasi | **TOML** lokal di folder aplikasi |
| Hak Akses | **Run as Administrator** (diperlukan untuk UIPI) |
| Shortcut Utama | `Win + `` (backtick) |
| Shortcut Fallback | `Alt + `` jika `Win` diblokir OS |
| Shortcut Snap | `Ctrl+Win+Panah`, `Ctrl+Win+Enter` (fullscreen), `Ctrl+Win+Shift+Enter` (pindah monitor) |
| Build | Skrip otomasi lokal dev/prod (closed-source) |
| Distribusi | **Microsoft Store** (untuk melewati SmartScreen) |
| Pemasaran | [redacted: marketing strategy] |

---

## Prioritas Fitur (MoSCoW)

### 🔴 MUST — Fitur MVP
1. **Window Switcher** (`Win + `` / `Alt + ``): Mendeteksi jendela dari proses yang sama (`GetForegroundWindow` → `GetWindowThreadProcessId` → `EnumWindows` → `SetForegroundWindow`), membawa jendela berikutnya ke foreground **tanpa** auto-minimize atau transisi.
2. **Rust Engine**: Executable mandiri berbasis Rust dengan binary linking statis, menjaga ukuran kecil tanpa runtime dependencies eksternal.
3. **Elevated Privilege**: Manifest `requireAdministrator` agar dapat menembus UIPI dan mengontrol semua jendela termasuk yang elevated.
4. **Build Automation**: Skrip PowerShell/Cargo lokal (`build.ps1`) untuk kompilasi `debug` dan `release` secara terpisah.

### 🟡 SHOULD — Fitur Penting
5. **Snap Keyboard Shortcuts** (BetterSnapTool-style): `Ctrl+Win+←/→` (snap kiri/kanan 50%), `Ctrl+Win+Enter` (fullscreen), `Ctrl+Win+Shift+Enter` (pindah monitor). Semua dapat dikustomisasi via TOML. Harus **DPI-aware** menggunakan `MonitorFromWindow` dan ukuran pixel nyata monitor target.
6. **Overlapping Stack Layout**: Menempatkan hingga N jendela dengan ukuran 50% width, offset horizontal berurutan (kiri → tengah → kanan) sehingga tepi setiap jendela tetap visible dan dapat diklik. Berguna untuk menumpuk aplikasi chat di monitor kecil.
7. **Async Hook Architecture**: Thread keyboard hook (`WH_KEYBOARD_LL`) hanya bertugas meneruskan event ke channel; seluruh logika (pencarian jendela, pemindahan fokus, snap) dieksekusi di thread terpisah. Mencegah Windows memutus hook karena respons lambat.
8. **Microsoft Store Distribution**: Mempublikasikan aplikasi ke Microsoft Store untuk menghilangkan peringatan SmartScreen pada mesin pengguna baru.

### 🟢 COULD — Fitur Tambahan
9. **Per-Virtual-Desktop Layout**: Menyimpan konfigurasi snap layout yang berbeda untuk setiap virtual desktop Windows 11.
10. [redacted: marketing strategy]

### ⚫ WON'T — Ditunda
*(Tidak ada fitur yang ditunda sepenuhnya pada tahap ini)*

---

## Batasan & Risiko Penting
- **Keyboard hook** menggunakan `WH_KEYBOARD_LL` (global low-level hook), bukan `RegisterHotKey`. Ini perlu dikomunikasikan dengan transparan kepada pengguna untuk menghindari persepsi keylogger.
- **DPI multi-monitor**: Semua operasi posisi/ukuran jendela harus menggunakan Win32 API yang DPI-aware (`GetDpiForMonitor`, koordinat per-monitor) bukan koordinat layar virtual yang bergantung pada DPI primer.
- **Proyek closed-source**: GitHub Actions tidak dapat digunakan. Proses build harus bisa dijalankan sepenuhnya secara lokal dari mesin developer.
- **Code signing** belum tersedia pada tahap ini; Microsoft Store menjadi solusi distribusi utama untuk melewati SmartScreen. [redacted: cost figure]

---

## Langkah Berikutnya yang Direkomendasikan
1. **`bmad-spec`** — Buat dokumen spesifikasi teknis (SPEC) dari intent ini sebagai kontrak implementasi.
2. **`bmad-prd`** — Buat PRD lengkap dengan user stories dan acceptance criteria berbasis MoSCoW di atas.
3. **`bmad-create-epics-and-stories`** — Pecah menjadi epic + story siap implementasi untuk Developer Agent.
