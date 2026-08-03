# Minutes of Meeting (MoM) — Diskusi Proyek WinTick

**Metadata**
- **Tanggal**: 2026-07-07
- **Waktu Mulai**: 06:29 WIB
- **Waktu Selesai**: 07:23 WIB
- **Topik**: Architecture Coaching — Penyusunan Architecture Spine
- **Peserta**: kodesh87, AI Agent (Antigravity)

## 1. Ringkasan Eksekutif (Executive Summary)
Sesi ini merupakan sesi *coaching* interaktif untuk menyusun **Architecture Spine** proyek WinTick secara bertahap. Dimulai dari paradigma desain yang sudah ditetapkan (Actor/Message-Passing), sesi ini menyelesaikan **13 keputusan arsitektural (AD)** melalui 14 ronde diskusi. Pengguna berperan sebagai pengambil keputusan, sementara agen menyajikan opsi-opsi beserta analisis dan rekomendasi. Beberapa koreksi fundamental datang dari pengguna — khususnya terkait kelemahan filter PID pada aplikasi multi-process (Ronde 2), koreksi batas RAM (Ronde 7), penyempurnaan model error agar tidak *spam* Toast (Ronde 8), dan penolakan *over-engineering* de-elevation (Ronde 9). Sesi ditutup dengan finalisasi dokumen `ARCHITECTURE-SPINE.md` berstatus `final`.

## 2. Alur Percakapan (Flow of Speech)

**Ronde 1**: Pengguna meminta inisialisasi MoM dan menyetujui Opsi B untuk Inter-Thread Communication (Hook thread menangani throttle + u8 Enum). Agen mencatat keputusan ke `.memlog.md`.

**Ronde 2**: Pengguna mengoreksi asumsi agen terkait filter berbasis PID — menunjukkan bahwa aplikasi modern (Electron/Chromium) memiliki banyak PID per instance. Contoh konkret: AntiGravity 3 instance = PID berbeda-beda, masing-masing 12 proses. Agen menerima koreksi; keputusan ditetapkan pada *Brute-force Just-in-Time Traversal* Z-Order tanpa cache, menerima biaya iterasi demi akurasi substansi aplikasi.

**Ronde 3**: Pengguna langsung menetapkan Event Messaging (Win32 Window Message) untuk config reload, menolak file watcher karena terlalu *costly*. Keputusan cepat tanpa perdebatan.

**Ronde 4**: Pengguna mempertanyakan apakah OS secara otomatis mengelola bypass shortcut untuk VM/RDP. Agen menjelaskan perbedaan fundamental `RegisterHotKey` (OS-managed, pengalaman pengguna dengan Snagit) vs `SetWindowsHookEx WH_KEYBOARD_LL` (self-managed, yang WAJIB dipakai WinTick per SPEC). Karena WinTick menggunakan low-level hook, ia mencegat SEMUA input sebelum VM/RDP melihatnya — bypass manual diperlukan.

**Ronde 5**: Berdasarkan elaborasi Ronde 4, pengguna memilih Opsi A (VM/RDP Bypass di Hook Thread). Hook mengecek class name jendela aktif terhadap daftar bypass sebelum intercept; jika cocok, langsung `CallNextHookEx`.

**Ronde 6**: Pengguna menyetujui Opsi C untuk Same-Application Identity: Exe Name sebagai primary identity + Class Name sebagai exclusion filter untuk ghost windows. PID ditolak sebagai primary (karena Ronde 2).

**Ronde 7**: Pengguna mengoreksi batas RAM — target ideal <2MB, tapi batas keras *acceptable* <10MB (konfirmasi ditemukan di PRD prd.md baris 108). Agen memperbarui constraint. Pengguna juga menyetujui `egui` sebagai GUI framework untuk `wintick-settings.exe`.

**Ronde 8**: Pengguna menyempurnakan model error. Setuju dengan startup MessageBox, tapi menolak fokus pada Toast untuk runtime — terlalu mengganggu. Menetapkan Protokol 3-Tingkat: (1) Startup Fatal → MessageBox + exit, (2) Runtime Warning → silent log + tray icon titik merah (unread log), (3) Runtime Critical → tray icon silang merah + Toast minimal 1x saja. Toast HANYA untuk skenario kritis.

**Ronde 9**: Pengguna menantang asumsi agen tentang perlunya de-elevation untuk `wintick-settings.exe`. Argumen: "kita yang buat, kita yang pastikan aman." Agen mengakui ini over-engineering. Keputusan: `ShellExecute` biasa, warisan Admin diterima.

**Ronde 10**: Pengguna meminta agen mengaudit seluruh artefak untuk menemukan dimensi yang belum dibahas.

**Ronde 11**: Agen menyajikan 4 dimensi tersisa: (1) Explorer Crash Recovery — `TaskbarCreated` listener, (2) Hook Health Monitoring — heartbeat periodik, (3) Virtual Desktop Isolation — API `IVirtualDesktopManager`, (4) Cargo Workspace — 3 crate (daemon/settings/shared).

**Ronde 12**: Pengguna menyetujui dimensi 10 dan 13. Mempertanyakan interval heartbeat (5 detik terlalu sering?) dan meminta riset ulang apakah Virtual Desktop API benar-benar *undocumented*. Agen melakukan riset web — hasilnya: `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` adalah **API resmi Microsoft** yang didokumentasikan publik di `shobjidl_core.h` sejak Windows 10. Koreksi informasi awal agen.

**Ronde 13**: Pengguna menyetujui heartbeat 10 detik dan API resmi `IVirtualDesktopManager`. Seluruh 13 dimensi arsitektural telah ditetapkan.

**Ronde 14**: Pengguna memerintahkan finalisasi. Agen memverifikasi versi dependensi terkini via web (windows-sys v0.61.x, egui v0.35.x, toml v1.1.x), menyusun dan menulis `ARCHITECTURE-SPINE.md` lengkap dengan 12 AD, 4 diagram Mermaid, Capability Map, Consistency Conventions, Stack, Deferred, dan RAM Budget. Status: `final`.

## 3. Kesimpulan & Keputusan Utama (Conclusions & Key Decisions)

| # | Dimensi | Keputusan (AD) |
| --- | --- | --- |
| AD-1 | Design Paradigm | Actor / Message-Passing — setiap unit punya state sendiri, komunikasi via pesan satu arah |
| AD-2 | Inter-Thread Comm | Hook handles throttle (<50ms) + u8 Enum via lock-free ring buffer 16 slot |
| AD-3 | Z-Order Traversal | Stateless Just-in-Time — iterasi Z-Order tanpa cache, biaya diterima |
| AD-4 | App Identity | Exe Name primary + Class Name exclusion filter (PID ditolak) |
| AD-5 | Config Reload | Explicit IPC Signal via `WM_APP_RELOAD_CONFIG` Win32 message |
| AD-6 | VM/RDP Bypass | Hook Thread evaluation — cek bypass list sebelum intercept |
| AD-7 | Error Model | Protokol 3-Tingkat: MessageBox (startup) / Silent Log+titik merah (warning) / Silang merah+Toast 1x (critical) |
| AD-8 | Hook Health | Heartbeat 10 detik + auto re-register |
| AD-9 | VDesktop Isolation | API resmi `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` |
| AD-10 | Explorer Recovery | `TaskbarCreated` broadcast listener + re-register tray icon |
| AD-11 | Settings Binary | `egui` GUI + `ShellExecute` biasa (warisan Admin) |
| AD-12 | Project Structure | Cargo Workspace: 3 crate (daemon, settings, shared) |

**Constraint Penting:**
- RAM daemon: target ideal <2MB, batas keras <10MB
- Binary size: target 250KB–400KB, max 500KB
- Stack: Rust stable + `windows-sys` v0.61.x + `egui` v0.35.x + `toml` v1.1.x

**Koreksi Fundamental oleh Pengguna:**
1. PID tidak bisa dijadikan primary identity — aplikasi multi-process (Ronde 2)
2. Batas RAM 2MB adalah target ideal, bukan hard limit — 10MB adalah hard limit (Ronde 7)
3. Toast notification hanya untuk critical, bukan setiap error (Ronde 8)
4. De-elevation adalah over-engineering — kita yang buat kodenya (Ronde 9)

## 4. Daftar Tindakan Lanjut (Action Items)
| Tindakan / Tugas | Penanggung Jawab | Status |
| :--- | :--- | :--- |
| Adopsi Architecture Spine sebagai companion ke SPEC via `bmad-spec` | kodesh87 + AI | [ ] Belum Selesai |
| Pecah arsitektur menjadi Epics & Stories via `bmad-create-epics-and-stories` | kodesh87 + AI | [ ] Belum Selesai |
| Finalisasi MoM sesi arsitektur | AI | [x] Selesai |
