# Minutes of Meeting (MoM) — Diskusi Perencanaan Proyek WinTick

* **Tanggal**: Sabtu, 4 Juli 2026
* **Waktu Pertemuan**: 20:29:22 — 21:18:58 (Local Time)
* **Nama Proyek**: WinTick (macOS Command+~ Window Switcher to Windows 11)
* **Peserta**: 
  - `kodesh87` (Product Owner / User)
  - `Antigravity` (AI Agent - Brainstorming Coach / Facilitator)
* **Status Sesi**: Lengkap & Selesai (Final)

---

## 1. Ringkasan Eksekutif (Executive Summary)
Pertemuan ini merupakan sesi curah pendapat (*brainstorming*) terpandu untuk mengadopsi shortcut macOS `Command+~` (berpindah antar jendela aktif dari aplikasi yang sama) agar dapat berjalan di Windows 11. Tujuan produk ini (bernama **WinTick**) adalah menghadirkan switcher yang sangat ringan, andal, hemat memori (< 10MB RAM), berjalan di taskbar/system tray, dan minim konfigurasi. Diskusi mengeksplorasi kebutuhan pengguna, batasan keamanan Windows (SmartScreen, UIPI, keyboard hooks), arsitektur teknis berbasis Rust, otomatisasi build lokal, serta jalur distribusi melalui Microsoft Store. [redacted: marketing strategy]

---

## 2. Hasil Keputusan Prioritas Fitur (MoSCoW)
Berdasarkan hasil konsensus akhir sesi curah pendapat, prioritas fitur dan spesifikasi untuk proyek **WinTick** disepakati sebagai berikut:

### 🔴 MUST (Harus Ada untuk Rilis MVP)
1. **Shortcut Window Switcher**: Pintasan pemindah jendela macOS-style (`Win + `` atau fallback `Alt + `` jika `Win` diblokir OS).
2. **Bahasa Pemrograman Rust**: Memakai Rust guna menjamin performa super ringan dengan konsumsi memori RAM < 10MB (bahkan diupayakan < 1MB) dan CPU ~0%.
3. **Hak Akses Admin**: Menjalankan aplikasi dengan hak Administrator demi menembus batasan UIPI agar bisa mengontrol jendela elevated (seperti Task Manager/CMD Admin).
4. **Otomatisasi Build Lokal**: Skrip otomatisasi build lokal (dev & prod build) untuk mempermudah kompilasi mandiri karena proyek bersifat *closed-source*.

### 🟡 SHOULD (Sangat Penting / Tahap Berikutnya)
1. **BetterSnapTool Snapping**: Pintasan kustom pemindahan & pemosisian jendela (`Ctrl+Win+Panah` untuk snap setengah, `Ctrl+Win+Enter` untuk layar penuh, `Ctrl+Win+Shift+Enter` untuk pemindahan antar monitor).
2. **Overlapping Stack Layout**: Desain layout menumpuk jendela 50% width dengan offset horizontal (kiri, tengah, kanan) khusus monitor kecil agar tepi jendela tetap terlihat & dapat diklik.
3. **Arsitektur Hook Asinkronus**: Menggunakan thread asinkron yang memisahkan pembacaan hook keyboard dari pemrosesan logika agar Windows tidak mencabut global hook secara sepihak.
4. **Distribusi Microsoft Store**: Mendaftarkan aplikasi ke Microsoft Store agar terbebas dari peringatan blokir Windows SmartScreen. [redacted: commercial strategy]

### 🟢 COULD (Mungkin Ditambahkan jika Waktu Cukup)
1. **Desktop Virtual Layout**: Layout snap yang independen dan berbeda untuk setiap desktop virtual.
2. [redacted: marketing strategy]

### ⚫ WON'T (Ditunda untuk Versi Masa Depan)
* *Tidak ada fitur/spesifikasi yang ditunda sepenuhnya.*

---

## 3. Daftar Tindakan Lanjut (Action Items)
| Tindakan / Tugas | Penanggung Jawab | Status | Referensi |
| :--- | :--- | :--- | :--- |
| Menyusun dokumen spesifikasi teknis (`SPEC.md`) berdasarkan prioritas MoSCoW | Amelia (Senior Dev) | `[ ] Belum Mulai` | `bmad-spec` |
| Meriset Windows API untuk `SetWindowsHookEx` (keyboard hook) secara asinkron | Winston (Architect) | `[ ] Belum Mulai` | Win32 SDK |
| Menginisialisasi repositori proyek berbasis Rust | Amelia (Senior Dev) | `[ ] Belum Mulai` | Cargo.toml |
| Membuat skrip build lokal otomatis (`build.ps1`) untuk kompilasi dev/prod | Amelia (Senior Dev) | `[ ] Belum Mulai` | PowerShell |

---

## 4. Kronologi Percakapan Lengkap (Rangkuman Ronde)
* **Ronde 0**: Inisiasi sesi brainstorming. Menentukan 4 teknik pemicu (Role Playing, Six Thinking Hats, Disney Method, $0 Mandate).
* **Ronde 1 (Power User)**: Mengungkap ekspektasi shortcut yang andal, rasa frustasi jika shortcut melambat, serta tuntutan kemudahan instalasi, auto-start, system tray icon, dan kehandalan aplikasi.
* **Ronde 2 (IT Security)**: Membahas ketakutan administratif terkait keylogger/malware. Disepakati pembuktian keamanan melalui open source/developer terpercaya dan perlunya code signing.
* **Ronde 3 (Developer)**: Mengulas pemilihan bahasa pemrograman. Disepakati Rust/Go (C/C++ jika terpaksa) guna meminimalkan konsumsi RAM.
* **Ronde 4 (White Hat - Fakta)**: Mengumpulkan data bahwa `Win + `` bebas konflik di Windows 11. Mencatat kompetitor (EasySwitch) tidak aktif sejak 2022. Menetapkan target RAM < 1MB dan CPU ~0%.
* **Ronde 5 (Black Hat - Risiko)**: Menganalisis SmartScreen, UIPI (hak admin diperlukan untuk jendela elevated), dan risiko Windows memutus keyboard hook jika respons lambat.
* **Ronde 6 (Dreamer - Impian)**: Mengusulkan fitur snap BetterSnapTool dan layout bertumpuk (overlapping stack 50% width) inovatif untuk monitor kecil.
* **Ronde 7 (Realist - Teknis)**: Menyepakati Rust dengan target RAM < 10MB (max 20MB), transanti fokus jendela instan tanpa efek minimize, dan TOML untuk penyimpanan konfigurasi lokal.
* **Ronde 8 (Critic - Celah)**: Menyepakati perlunya build dev/prod lokal otomatis, penanganan DPI multi-monitor yang akurat, serta `Alt + `` sebagai shortcut alternatif cadangan.
* **Ronde 9 ($0 Mandate - Biaya)**: Memilih opsi rilis Microsoft Store untuk memotong SmartScreen, menolak GitHub Actions karena proyek saat ini bersifat closed-source. [redacted: marketing strategy]
* **Ronde 10 (Konvergensi)**: Penyusunan prioritas MoSCoW final dan pembuatan risalah rapat (MoM).
* **Ronde 11 (Party Mode - Spesifikasi Teknis)**: Membedah `SPEC.md` untuk mencari celah fungsional. Diputuskan bahwa: (1) Jendela ter-minimize akan dilewati (hanya switch ke jendela aktif/visible), (2) File `config.toml` disimpan di lokasi yang aman seperti `%APPDATA%` agar tidak memicu konflik hak akses admin, (3) Pintasan harus dtekan secara presisi tanpa tambahan modifier tak terduga, dan fallback (`Alt + ``) bertindak sebagai default rilis bukan *failover* otomatis saat runtime.
