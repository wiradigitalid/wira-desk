---
title: WinTick PRD
status: final
created: 2026-07-06
updated: 2026-07-10
---

# Product Requirements Document (PRD): WinTick

## 1. Product Vision & Goals
**Vision:** Membawa pengalaman navigasi mulus ala MacOS (`Cmd + \``) ke ekosistem Windows. WinTick hadir untuk memecahkan rasa frustrasi pengguna Windows (terutama pengguna yang sering berpindah dari MacOS) terhadap ketiadaan jalan pintas bawaan untuk berpindah antar jendela dari satu aplikasi yang sama secara cepat dan andal.

**Goals:**
- Menciptakan utilitas *App-Specific Window Cycler* super ringan yang beroperasi secara *invisible* dari System Tray.
- Mengungguli solusi alternatif (yang telah menjadi *abandonware*) dengan stabilitas mutlak berkat eksekusi level OS yang tidak akan macet meskipun berhadapan dengan aplikasi yang *hang*.
- Menyediakan fitur pengatur tata letak jendela (*window snapping & stacking*) terinspirasi *BetterSnapTool* sebagai nilai tambah yang membedakan WinTick dari utilitas rotasi biasa. [redacted: commercial strategy]

## 2. Target Audience & Personas
1. **Power Users & Switchers:** Pengguna aktif Windows yang juga terbiasa dengan MacOS dan merindukan efisiensi *shortcut* `Cmd + \``.
2. **Professionals:** Pengembang, desainer, atau pekerja kantoran yang sering membuka banyak jendela Chrome, Word, atau VSCode dan membutuhkan perpindahan instan tanpa kehilangan fokus layar.

**Skala Peluncuran:** Dimulai sebagai *Personal Use*. [redacted: commercial strategy]

## 3. Core Features (Functional Requirements)

### 3.1 Window Cycling (P1 — MUST)
| ID | Fitur | Deskripsi |
| :--- | :--- | :--- |
| **FR-1** | **App-Specific Window Cycling** | Rotasi jendela **hanya** memproses jendela milik aplikasi yang sama (berdasarkan PID/App Class). Bukan pengganti `Alt+Tab` global. |
| **FR-2** | **Spatial Layout Preservation** | Rotasi dikunci secara ketat hanya pada Monitor fisik dan *Virtual Desktop* yang persis sama dengan jendela aktif saat itu. |
| **FR-3** | **VM & Remote Desktop Bypass** | Input pintasan akan dibiarkan menembus (*bypassed*) secara alami tanpa diintersepsi jika jendela yang sedang aktif adalah Mesin Virtual atau *Remote Desktop*. Deteksi dilakukan berdasarkan *window class name* atau *process name* yang dikenal (misal: `VMwareUnityWindow`, `MobaXterm`, `mstsc.exe`, `vmconnect.exe`). Daftar ini dapat dikonfigurasi via `config.toml`. |
| **FR-4** | **UX Honesty (Anti-Skip)** | Dilarang menyembunyikan jendela *Not Responding*. Jendela tersebut wajib diangkat ke *Foreground* agar status kerusakannya terlihat transparan oleh pengguna. |
| **FR-5** | **Bypass Minimized & Ghost Windows** | Siklus rotasi wajib melompati (*skip*) jendela yang sedang dalam status *minimized*, *ghost windows* tersembunyi, dan jendela ber-style `WS_EX_TOOLWINDOW` / *system overlays*. Hanya merotasi jendela yang benar-benar terbuka secara visual di layar. |
| **FR-6** | **Precise Shortcut Matching** | Pintasan harus ditekan secara **presisi**. Jika pengguna menekan modifier tambahan yang tidak terdaftar (misal `Win+Shift+\`` saat yang terdaftar hanya `Win+\``), rotasi **tidak boleh aktif**. |
| **FR-7** | **Configurable Shortcut & Fallback** | Pintasan dapat dikonfigurasi melalui berkas `config.toml` yang disimpan di `%APPDATA%\WinTick`. Menyediakan *shortcut* bawaan ganda: `Win + \`` (Utama) dan `Alt + \`` (Cadangan/Fallback). Kedua pintasan bersifat default rilis; bukan *failover* otomatis saat *runtime*. |
| **FR-8** | **Administrator Elevation (UIPI Bypass)** | WinTick **wajib berjalan sebagai Administrator** agar bisa mengendalikan jendela *elevated* (Task Manager, CMD Admin) menembus batasan *User Interface Privilege Isolation* (UIPI) Windows. |

### 3.2 System Integration (P1 — MUST)
| ID | Fitur | Deskripsi |
| :--- | :--- | :--- |
| **FR-9** | **Background Tray-Resident Mode** | Aplikasi wajib berjalan sebagai *background task* yang sepenuhnya *tray-resident* dari System Tray Windows. Implementasi ikon tray **wajib menggunakan murni Win32 API** tanpa framework GUI pihak ketiga demi menggaransi target RAM. |
| **FR-10** | **Auto-Recovery System Tray** | Sistem memantau *broadcast* pesan `TaskbarCreated` OS untuk merender ulang ikon System Tray secara otomatis bila *explorer.exe* *restart*. |
| **FR-11** | **3-Tier Error Protocol** | Protokol penanganan error tiga lapis: **(Tier 1 — Startup Fatal)** kegagalan terminal saat *startup* (misal Hook diblokir AV) hanya boleh memunculkan **maksimal 1x** `MessageBox` peringatan sebelum aplikasi mematikan diri. **(Tier 2 — Runtime Warning)** kegagalan operasional *runtime* non-fatal **mutlak dilarang memunculkan pop-up**; wajib menggunakan *silent logging* internal dan menampilkan *overlay* titik merah kecil pada ikon System Tray. **(Tier 3 — Runtime Critical)** jika *keyboard hook* mati saat *runtime* (terdeteksi oleh *heartbeat monitor*), ikon System Tray berubah ke silang merah besar dan wajib mengirimkan **tepat 1x** *Toast Notification* Windows untuk memperingatkan pengguna (karena ikon tray dapat tersembunyi di *overflow menu*). |
| **FR-12** | **View Logs via Tray Menu** | Menu klik-kanan pada ikon System Tray wajib menyediakan opsi **"View Logs"** yang memungkinkan pengguna mengakses berkas *silent log* untuk diagnostik mandiri. |
| **FR-13** | **Auto-Start on Boot** | Aplikasi dapat diatur untuk langsung berjalan otomatis saat OS Windows dinyalakan. Mekanisme: **Windows Task Scheduler** — tugas terjadwal dengan pemicu *at logon* (`ONLOGON`) untuk pengguna `%USERNAME%` aktif dan hak *Run with highest privileges* (`/RL HIGHEST`), sehingga daemon diluncurkan *elevated* secara sunyi tanpa prompt UAC saat boot. Aksi tugas wajib menunjuk path absolut executable dan parameter *Start in* dikosongkan sebagai mitigasi *DLL Hijacking*. Toggle aktivasi/deaktivasi disediakan melalui menu klik-kanan ikon System Tray. |

### 3.3 Window Snapping & Layout (P2 — SHOULD)
| ID | Fitur | Deskripsi |
| :--- | :--- | :--- |
| **FR-14** | **Window Snapping Shortcuts** | Pintasan *keyboard* untuk pemosisian jendela yang presisi (contoh: `Ctrl+Win+Panah` untuk *snap* setengah layar, `Ctrl+Win+Enter` untuk *full-screen*, `Ctrl+Win+Shift+Enter` untuk pemindahan antar monitor). Semua operasi *snapping* wajib menghormati skala DPI dari monitor target untuk menghindari ketidakakuratan piksel. |
| **FR-15** | **Overlapping Stack Layout** | Algoritma *layout* khusus untuk monitor kecil: menumpuk maksimal 3 jendela dengan ukuran **50% lebar layar**, ditata dengan jeda/offset *horizontal* (Kiri, Tengah, Kanan) sehingga tepian tiap jendela tetap terlihat dan bisa diklik dengan *mouse*. Penghitungan dimensi dan posisi wajib memperhitungkan DPI monitor. |

### 3.4 Configuration & Onboarding (P1 — MUST)
| ID | Fitur | Deskripsi |
| :--- | :--- | :--- |
| **FR-16** | **Tray Context Menu Lengkap** | Menu klik-kanan pada ikon System Tray wajib menyediakan item lengkap dengan urutan: **Settings...**, **View Logs**, **Auto-Start** (toggle) — separator — **Check for Updates...**, **About** — separator — **Exit**. |
| **FR-17** | **Interactive First-Run Simulation** | Saat aplikasi dijalankan untuk pertama kalinya (*first-run*), modul `wintick-settings.exe` wajib meluncurkan simulasi interaktif yang memandu pengguna mempraktikkan pintasan `Win + \`` menggunakan *dummy window* di dalam UI. Simulasi wajib menyertakan tombol **"Skip Tutorial"** agar pengguna berpengalaman dapat melewati panduan. |
| **FR-18** | **Shortcut Capturer (Listening Mode)** | Kotak input pintasan pada jendela Settings wajib menggunakan mode "*Listening*" yang menangkap kombinasi *keyboard* fisik secara langsung, bukan menerima ketikan teks biasa. |

### 3.5 UX & Accessibility (P1 — MUST)
| ID | Fitur | Deskripsi |
| :--- | :--- | :--- |
| **FR-19** | **Adaptive Theming (Light/Dark Mode)** | Seluruh elemen visual pada jendela dialog (Settings, About, Onboarding) wajib mengikuti tema *native* OS pengguna (Light atau Dark mode) secara otomatis. |
| **FR-20** | **Keyboard Navigation (Accessibility)** | Seluruh elemen interaktif di dalam UI Settings (`wintick-settings.exe`) wajib dapat dinavigasi secara penuh menggunakan tombol *Tab* tanpa memerlukan *mouse*. |
| **FR-21** | **Screen Reader Support (Accessibility)** | Elemen *toggle* dan kotak *input shortcut* di dalam UI Settings wajib mendeskripsikan status aktif/non-aktifnya dengan jelas kepada *screen reader* via *UI Automation* bawaan Windows. |

## 4. Architecture & Non-Functional Requirements (NFR)
Batasan arsitektur ekstrem ini diturunkan dari spesifikasi *Advanced Elicitation* untuk menggaransi target kinerja (<500KB Biner, <2MB RAM Statis):

### 4.1 Concurrency & Communication
- **Two-Thread Architecture:** Pemisahan mutlak utas (*thread*). **Utas Hook** berjalan dengan prioritas `THREAD_PRIORITY_TIME_CRITICAL` untuk mencegat pintasan secara asinkron dan wajib menyelesaikan *callback hook* (hingga `CallNextHookEx`) dalam **< 10ms** — *budget* yang mengikat — demi meredam ancaman pencabutan `LowLevelHooksTimeout` OS (300ms). Angka ini adalah batas eksekusi *callback*, bukan target latensi rotasi yang dirasakan pengguna (lihat Success Metrics). **Utas Worker** mengeksekusi logika rotasi *Z-Order* secara terpisah.
- **Zero-Allocation Ring Buffer (16 Slot):** Komunikasi antar-utas wajib menggunakan antrean *lock-free* statis berkapasitas **maksimal 16 slot** berbasis tipe primitif murni `u8` (sertifikasi trait `Copy`) tanpa alokasi memori dinamis (*heap-allocated objects* dilarang keras). Jika *buffer* penuh, input baru langsung **dibuang (*drop*)** tanpa *blocking*.
- **Anti-Macro Throttle:** Utas Hook secara hulu akan mengabaikan input rentetan pintasan yang tidak wajar (jeda < 50ms) agar memori *buffer* tidak pernah meluap, membuang input tanpa memperlambat sistem.

### 4.2 Window Enumeration & Safety
- **Stateless Z-Order (Larangan Cache):** WinTick **dilarang mutlak meng-*cache*** status Z-Order jendela secara internal. Evaluasi urutan jendela wajib dilakukan secara *real-time* pada setiap *keypress* untuk menghindari desinkronisasi dengan interaksi *mouse* pengguna.
- **Kernel-API Sterilization:** Operasi penyaringan `EnumWindows` **mutlak dilarang** menggunakan Win32 API sinkron lintas-proses (seperti `SendMessage`, `GetWindowText`). Penyaringan hanya boleh bergantung pada operasi pembacaan memori Kernel *non-blocking* (`IsWindowVisible`, `GetWindowLong`, `GetWindowThreadProcessId`, `SetWindowPos`) demi memutus rantai *cascading hang*.
- **Graceful Fail on Invalid Target:** Jika jendela target keburu ditutup atau menjadi *invalid* selama jeda asinkron antara *hook* dan *worker*, utas Worker wajib mengabaikan target tersebut secara *graceful* dan langsung melompat ke jendela berikutnya di Z-Order tanpa *crash* atau *error*.

### 4.3 Build & Distribution
- **Compiler Aggressiveness:** Mempertahankan ekosistem *Standard Library* (`std`) Rust demi *Thread Safety* yang terjamin, namun mengutamakan *crate* FFI murni `windows-sys` (bukan `windows` yang rawan bloat abstraksi). Diperkuat dengan profil rilis sangat agresif (`lto=true`, `opt-level="z"`, `strip=true`, `panic="abort"`). Target ukuran biner final: **250KB — 400KB**.
- **Build Automation:** Skrip `build.ps1` di *root* proyek wajib menyediakan mode `dev` (debug + logging) dan `prod` (optimized release), termasuk pengecekan dependensi Rust toolchain.
- **Distribusi:** Rilis awal menargetkan distribusi murni via Microsoft Store (*MSIX packaging*) untuk memotong peringatan *SmartScreen* Defender Windows dan memberikan proses instalasi/update *one-click* yang terpercaya bagi pengguna akhir.

## 5. User Journeys

**Skenario A: Rian — Rotasi Dokumen Multi-Monitor Terisolasi**
Rian, seorang *full-stack developer*, membuka 3 dokumen Word di Monitor Kiri dan 2 dokumen Word di Monitor Kanan.
1. Fokus kursor Rian saat ini berada di salah satu dokumen di Monitor Kanan.
2. Rian menekan *shortcut* WinTick (`Win + \``).
3. Hanya 2 dokumen Word di Monitor Kanan yang bergantian maju ke depan secara instan.
4. Efek *Spatial Preservation* aktif; fokus Rian tidak dilempar kaget ke Monitor Kiri, menjaga keutuhan tata letak ruang kerja spasialnya.

**Skenario B: Maya — Konfrontasi Aplikasi "Not Responding"**
Maya, seorang desainer grafis, membuka banyak jendela Chrome untuk riset visual.
1. Salah satu tab Chrome tiba-tiba membeku (*Not Responding*).
2. Saat siklus rotasi WinTick menabrak jendela tersebut, jendela *hang* itu tetap dipaksa maju ke layar penuh (penerapan *UX Honesty*).
3. Maya menekan *shortcut* WinTick sekali lagi dengan cepat.
4. Siklus instan melompati jendela *hang* tersebut berkat proteksi isolasi *non-blocking API* Kernel; rotasi berjalan sempurna tanpa penundaan.

**Skenario C: Budi — Interaksi dengan Jendela Elevated**
Budi, seorang *sysadmin*, membuka 2 jendela CMD: satu CMD biasa dan satu CMD *Run as Administrator*.
1. Budi menekan *shortcut* WinTick.
2. Berkat hak Administrator dan *UIPI Bypass*, WinTick dapat merotasi fokus ke CMD *elevated* dengan mulus, tanpa ditolak oleh OS.

## 6. User Education (Filosofi Spasial)
Karena perilaku rotasi per-monitor WinTick (*Spatial Layout Preservation*) sangat berbeda dari perilaku agresif `Alt+Tab` bawaan Windows yang merotasi semua jendela lintas-monitor, PRD mewajibkan penyertaan bab **Edukasi Pengguna** pada antarmuka aplikasi (misal *tooltip* pada ikon tray), berkas *README*, dan/atau laman depan produk, yang secara eksplisit menjabarkan:
- **Apa itu Filosofi Spasial:** WinTick hanya merotasi jendela yang berada di monitor dan desktop virtual yang sama, bukan seluruh layar.
- **Mengapa:** Mencegah efek *whiplash* (layar melompat kaget) dan melindungi tata letak kerja pengguna.
- **Perbedaan dengan Alt+Tab:** Memperjelas bahwa ini adalah fitur desain yang disengaja, bukan bug.

## 7. Out of Scope / Future Opportunities
- **Visual Switcher Overlay:** Tidak akan ada antarmuka menu grafis transparan (seperti Alt+Tab bawaan Windows) yang muncul memakan ruang layar. Seluruh rotasi bersifat tak kasat mata (*invisible*).
- **Sinkronisasi Cloud:** Konfigurasi akan murni berbasis berkas `config.toml` lokal; tidak ada transmisi data atau telemetri.
- **Desktop Virtual Layout (Independen per VDesktop):** Layout *snap* yang independen dan berbeda untuk setiap *desktop virtual* (saat ini layout berlaku seragam). Ditandai sebagai ruang eksplorasi untuk versi mendatang.
- [redacted: marketing strategy]

## 8. Success Metrics
| Metrik | Target | Counter-Metric |
| :--- | :--- | :--- |
| Ukuran Biner (.exe) | < 500KB (target riil: 250KB–400KB) | Tidak boleh mengorbankan fitur inti demi ukuran |
| RAM Statis | < 2MB (batas keras: < 10MB) | Tidak boleh menggunakan `#![no_std]` yang merusak *thread safety* |
| Latensi Rotasi (*perceived*, end-to-end) | Sub-milidetik (< 1ms) — target persepsi pengguna atas perpindahan fokus; **berbeda** dari *budget* eksekusi hook callback < 10ms (Bab 4.1) | Tidak boleh menyembunyikan jendela *hang* demi kecepatan |
| CPU Idle | ~0% | — |
| Hook Stability | Zero dropout selama masa hidup proses | Tidak boleh menggunakan `RegisterHotKey` yang kalah prioritas |
## 9. Glossary
| Istilah | Definisi |
| :--- | :--- |
| **Z-Order** | Urutan tumpukan jendela dari depan ke belakang yang dikelola oleh Windows Desktop Window Manager. |
| **Hook Thread** | Utas berpriorititas `TIME_CRITICAL` yang bertugas mencegat input keyboard OS via `WH_KEYBOARD_LL`. |
| **Worker Thread** | Utas kedua yang memproses logika rotasi jendela (enumerasi, penyaringan, dan `SetWindowPos`). |
| **Ring Buffer** | Antrean sirkular statis *lock-free* berkapasitas 16 slot yang menjembatani komunikasi Hook Thread → Worker Thread. |
| **UIPI** | *User Interface Privilege Isolation* — mekanisme keamanan Windows yang mencegah proses non-admin mengendalikan jendela milik proses admin. |
| **Spatial Preservation** | Filosofi desain WinTick di mana rotasi jendela dikunci hanya pada Monitor fisik dan Virtual Desktop yang sama. |
| **UX Honesty** | Prinsip bahwa jendela *Not Responding* tidak boleh disembunyikan; pengguna berhak melihat status kerusakan. |
| **Ghost Window** | Jendela bayangan yang dibuat Windows saat aplikasi *hang*, biasanya ber-style `WS_EX_TOOLWINDOW`. |
| **LowLevelHooksTimeout** | Batas waktu OS Windows (~300ms) sebelum secara sepihak mencabut *global keyboard hook* yang tidak merespons. |
