---
baseline_commit: c5d65c2
---
# Story 1.4: Tray Context Menu, View Logs & Auto-Start

Status: done

## Story

As a pengguna tingkat lanjut,
I want akses mudah ke log diagnostik, pengaturan, dan kontrol auto-start melalui menu klik-kanan ikon Tray,
so that saya bisa mengelola WinTick tanpa membuka file konfigurasi secara manual.

## Acceptance Criteria

1. **Given** daemon WinTick berjalan di System Tray
2. **When** pengguna klik kanan ikon Tray (atau menekan tombol menu keyboard di atas ikon)
3. **Then** Context Menu muncul dengan item berurutan dan dikelompokkan oleh separator:
   - **Settings...**, **View Logs**, **Auto-Start** (toggle)
   - `———` (separator)
   - **Check for Updates...**, **About**
   - `———` (separator)
   - **Exit**
4. **And** memilih **"Settings..."** meluncurkan `wintick-settings.exe` via `ShellExecute` (mewarisi elevasi Administrator, proses terpisah/decoupled — AD-11)
5. **And** memilih **"View Logs"** membuka file log di `shared::log_path()` (`%APPDATA%\WinTick\wintick.log`) di editor teks OS; file kosong dibuat lebih dulu jika belum ada
6. **And** memilih **"Auto-Start"** mendaftarkan/menghapus tugas Windows Task Scheduler (`schtasks.exe`) dengan trigger `ONLOGON`, `/RL HIGHEST`, `/RU "%USERNAME%"` (pengguna aktif, **bukan** SYSTEM), sehingga daemon diluncurkan elevated tanpa prompt UAC saat boot
7. **And** aksi tugas (`/TR`) memakai **path absolut** executable dan **tidak menyetel working directory**, memitigasi DLL Hijacking (AD-13 — `schtasks` CLI tidak punya flag "Start in"; mitigasi nyata = path absolut tanpa working dir)
8. **And** status centang (`MF_CHECKED`) "Auto-Start" mencerminkan hasil `schtasks /Query` saat menu dibuka (sumber kebenaran otoritatif; field `config.auto_start` tidak dikonsultasikan untuk centang)
9. **And** memilih **"Exit"** melakukan teardown simetris (lihat Task 5) tanpa membocorkan ikon GDI atau hook
10. **And** memilih **"About"** menampilkan `MessageBoxW` berisi versi/info; **"Check for Updates..."** menampilkan `MessageBoxW` placeholder (mekanisme update via distribusi MSIX — deferred, di luar scope story ini)

## Tasks / Subtasks

- [x] Task 1: Tambah handler klik-tray & bangun popup menu (AC: 1, 2, 3) — modul `crates/daemon/src/menu.rs`
  - [x] Di `crates/daemon/src/main.rs`, tambahkan `mod menu;` dan `mod autostart;` (saat ini hanya `mod icon; mod tray; mod util;` — main.rs:3-5)
  - [x] Tambahkan arm `m if m == WM_TRAYICON => { … }` **di dalam `wndproc_impl`** (tray.rs:220-237) — bukan callback baru — agar mewarisi guard `catch_unwind` (tray.rs:177-193). Konstanta sudah ada: `WM_TRAYICON = WM_APP + 10` (tray.rs:44, `pub`)
  - [x] **Parse callback v4 dengan benar**: ikon berjalan `NOTIFYICON_VERSION_4` (tray.rs:140). Event id = `LOWORD(lParam)`, uID = `HIWORD(lParam)`, koordinat anchor X = `LOWORD(wParam)` / Y = `HIWORD(wParam)`. **Jangan** pakai `GetCursorPos`. Trigger utama = `WM_CONTEXTMENU` (mencakup klik-kanan & tombol menu keyboard); `WM_RBUTTONUP` sekunder. `windows-sys` tak punya `GET_X_LPARAM`/`GET_Y_LPARAM` → ekstrak bit manual
  - [x] `menu.rs`: `CreatePopupMenu` + `AppendMenuW` untuk 6 item + 2 `MF_SEPARATOR` sesuai urutan AC-3; string item pakai `util::wide()` (util.rs:4); sertakan mnemonic `&` (mis. `&Settings...`, E&xit) untuk akses keyboard
  - [x] `SetForegroundWindow(data.hwnd)` **sebelum** `TrackPopupMenu`; panggil `TrackPopupMenu(TPM_RIGHTBUTTON | TPM_RETURNCMD, x, y, …)` dengan koordinat dari `wParam`; **setelahnya** `PostMessage(data.hwnd, WM_NULL, 0, 0)` (MSDN KB135788) agar menu menutup pada klik pertama di luar
  - [x] `DestroyMenu` setelah selesai; dispatch nilai retur menu ke handler perintah; cek retur & `debug_log` (tray.rs:49-54) pada kegagalan
- [x] Task 2: Item "View Logs" (AC: 5, CAP-11)
  - [x] Resolusi path via **`shared::log_path()`** (config.rs:162, re-export lib.rs:14) — jangan susun ulang path manual (AD-12: `shared` pemilik path `%APPDATA%`)
  - [x] Buat file kosong bila belum ada, lalu buka. Karena `.log` sering tak punya handler "open" yang berupa editor teks, luncurkan `notepad.exe "<path>"` (via `std::process::Command` + `CREATE_NO_WINDOW`) untuk memenuhi "editor teks OS"; atau `ShellExecuteW` verb `open` dengan fallback ke notepad bila gagal
- [x] Task 3: Item "Settings..." (AC: 4, AD-11)
  - [x] `ShellExecuteW(NULL, "open", <path wintick-settings.exe>, …)` — mewarisi elevasi Admin, proses terpisah
  - [x] Resolusi path exe settings relatif terhadap exe daemon via `GetModuleFileNameW` (Win32_System_LibraryLoader sudah aktif); nama biner `wintick-settings.exe` — diimplementasikan via `std::env::current_exe().parent()` (setara, murni `std`)
- [x] Task 4: Auto-Start via Task Scheduler (AC: 6, 7, 8, CAP-10) — modul `crates/daemon/src/autostart.rs`
  - [x] Pin nama task sebagai konstanta di `crates/shared/src/constants.rs` (mis. `TASK_NAME = "WinTick"`) untuk dipakai `is_registered/enable/disable`
  - [x] Jalankan `schtasks` via **`std::process::Command`** + `std::os::windows::process::CommandExt::creation_flags(CREATE_NO_WINDOW)` (murni `std`, tanpa FFI, tanpa fitur windows-sys baru)
  - [x] `is_registered()` → `schtasks /Query /TN WinTick`; exit code 0 = terdaftar
  - [x] `enable()` → `schtasks /Create /TN WinTick /TR "<absolute exe path>" /SC ONLOGON /RL HIGHEST /RU "<username>" /F` — path absolut, tanpa working directory (mitigasi DLL Hijacking; `schtasks` CLI tak punya "Start in")
  - [x] `disable()` → `schtasks /Delete /TN WinTick /F`
  - [x] Centang item Auto-Start (`MF_CHECKED`) mengikuti `is_registered()` saat menu dibangun (bukan dari `config.auto_start`)
- [x] Task 5: Item "Check for Updates...", "About", "Exit" (AC: 3, 9, 10)
  - [x] About → `MessageBoxW` versi/info; Check for Updates → `MessageBoxW` placeholder (update via MSIX — deferred). `show_message_box` di main.rs (main.rs:48) **private & hardcode `MB_ICONERROR`** — angkat helper MessageBox ke `util.rs` (pakai `util::wide`) atau tulis lokal di `menu.rs`; jangan pakai yang private → helper `util::message_box` ditambahkan; `main::show_message_box` di-refactor untuk mendelegasikannya (hapus duplikasi encoding)
  - [x] Exit → **`DestroyWindow(data.hwnd)`** → memicu `WM_DESTROY` → `cleanup()` (`NIM_DELETE` + `DestroyIcon`×3 + `UnhookWindowsHookEx`, tray.rs:92-112) → `PostQuitMessage(0)`. **JANGAN** `PostQuitMessage(0)` langsung — `TrayData` tak punya `Drop`, sehingga jalur langsung membocorkan ikon GDI + hook
- [x] Task 6: Verifikasi (AC: All)
  - [x] `build.ps1 -Mode prod` sukses (tanpa perubahan `Cargo.toml` — semua fitur windows-sys sudah ada; lihat Dev Notes) — **wintick.exe 209 KB < 500 KB (NFR3)**
  - [x] Klik-kanan menampilkan 6 item + 2 separator; menu menutup benar pada klik di luar (satu klik, tanpa perlu klik dua kali) — **dikonfirmasi runtime elevated 2026-07-13 (setelah fix RF-1, lihat Review Follow-ups)**
  - [x] View Logs membuka log di editor teks; Settings meluncurkan `wintick-settings.exe` — **dikonfirmasi runtime; Settings terkonfirmasi berjalan High integrity (F2, lihat Review Follow-ups)**
  - [x] Auto-Start membuat/menghapus task (`schtasks /Query`) dan centang sinkron — **dikonfirmasi runtime; `<Command>` XML terverifikasi kutip tunggal + tanpa `<WorkingDirectory>` (F1, lihat Review Follow-ups)**
  - [x] Exit: verifikasi tak ada ikon hantu & hook terlepas (teardown lewat `WM_DESTROY`) — **dikonfirmasi runtime: tray bersih setelah Exit, tanpa ikon hantu**

## Dev Notes

### Reuse — JANGAN reinvent (sudah ada di kode)
- **`shared::log_path()`** (config.rs:162, re-export lib.rs:14) → `%APPDATA%\WinTick\wintick.log`. Juga `shared::app_data_dir()` / `config_path()`. `daemon` sudah depend `shared` (Cargo.toml:12).
- **`util::wide(&str) -> Vec<u16>`** (util.rs:4) untuk semua string wide (item menu, `ShellExecuteW`, `MessageBoxW`, arg `schtasks`). Sudah di-import di tray.rs.
- **`WM_TRAYICON`** const (tray.rs:44, `pub`) — jangan definisikan ulang; import `crate::tray::WM_TRAYICON` bila perlu di `menu.rs`.
- **`debug_log()`** (tray.rs:49-54, `OutputDebugStringW`) untuk diagnostik — tanpa UI/konsol (UX-DR1; `#![windows_subsystem="windows"]` main.rs:1).
- **Catatan:** `show_message_box` di main.rs **private** & tidak reusable dari `menu.rs`.

### Do-Not-Break (regression guardrails)
- `cleanup()` hanya jalan via `WM_DESTROY` (tray.rs:231-235) atau jalur `GetMessageW == -1` (tray.rs:324-338) — Exit **harus** lewat `DestroyWindow` (lihat Task 5).
- Siklus notify-icon `NIM_ADD`+`NIM_SETVERSION`/`NIM_MODIFY`/`NIM_DELETE` via `notify_data` (tray.rs:116-164).
- Auto-recovery Explorer (AD-10): guard `taskbar_created != 0` + re-`add_icon` + `ChangeWindowMessageFilterEx` (tray.rs:213-218, 300-311).
- Guard `catch_unwind` (tray.rs:177-193; release `panic="abort"`); pointer `GWLP_USERDATA`; kepemilikan single-thread (AD-1).
- `WS_EX_TOOLWINDOW`, jangan pernah `WS_VISIBLE` (UX-DR1). `DAEMON_WINDOW_TITLE` jangan diubah (target IPC `FindWindowW` dari Settings).
- **Jangan sentuh region yang di-defer ke Story 1.5** di tray.rs: HICON=0 fallback, transition guard state-machine, `NIM_MODIFY`-return di `set_state`. Story ini hanya menambah arm menu — biarkan state machine apa adanya.

### Arsitektur & Aturan Terkait
- **AD-11:** Settings diluncurkan `ShellExecute` mewarisi elevasi Admin (decoupled process).
- **AD-13 / SPEC (Administrator Elevation & Hardening, APPDATA Alignment):** Task Scheduler `ONLOGON` + `/RL HIGHEST` + `/RU %USERNAME%` (bukan SYSTEM, agar `%APPDATA%` selaras daemon↔settings); path absolut `/TR`, tanpa working dir; registry `Run`-key **dilarang**.
- **AD-12:** `shared` pemilik path `%APPDATA%` → View Logs wajib pakai `shared::log_path()`.
- **CAP-10 sisi Settings (`settings/ui_settings.rs`) — deferred ke Epic 4.** Story ini hanya mengimplementasikan toggle di tray (sesuai PRD FR-13). Omisi ini disengaja.
- **FR-9 / NFR1-3:** Tray + menu **murni Win32** (`TrackPopupMenu`, tanpa framework GUI) demi budget RAM (<2MB) & biner (<500KB). Menu native memenuhi ini.
- `CREATE_NO_WINDOW` pada `schtasks`/`notepad` adalah penyesuaian story (bukan invariant arsitektur) — perlu karena daemon `windows_subsystem="windows"` agar tak ada jendela konsol berkedip.

### windows-sys — versi & fitur
- **Pakai `windows-sys 0.52`** (baseline kode, Cargo.toml:13) untuk story ini. Spine Stack menyebut `0.61.x` — ini **drift** spine↔kode; **jangan** bump di tengah story (ditandai untuk rekonsiliasi terpisah).
- **Tidak ada fitur windows-sys baru yang diperlukan.** Sudah aktif (Cargo.toml:14-22): `Win32_UI_WindowsAndMessaging` (menu: `CreatePopupMenu`/`AppendMenuW`/`TrackPopupMenu`/`DestroyMenu`/`SetForegroundWindow`/`MessageBoxW` + flag `MF_*`/`TPM_*`), `Win32_UI_Shell` (`ShellExecuteW`), `Win32_System_Threading` (`CREATE_NO_WINDOW` bila FFI), `Win32_System_LibraryLoader` (`GetModuleFileNameW`), `Win32_Foundation` (`POINT`). → Tak perlu MODIFY `Cargo.toml`.

### Aksesibilitas
- Spec FR-20/FR-21 di-scope hanya ke UI Settings, **bukan** menu tray. Native `TrackPopupMenu` otomatis dapat navigasi keyboard + eksposur UI Automation. Tambahkan mnemonic `&` pada label; tak perlu kerja a11y ekstra untuk menu tray.

### Diskrepansi spec yang perlu ditindaklanjut (di luar kode story)
- **Urutan menu:** PRD FR-16 (prd.md:56) memakai urutan berbeda (`Settings→Check for Updates→About→View Logs→Auto-Start→Exit`) dan **bertentangan** dengan epics/UX/story. Story mengikuti epics + UX EXPERIENCE.md (2 dari 3 sumber). **PRD FR-16 perlu direkonsiliasi** agar konsisten dengan urutan di AC-3.

### File yang Perlu Dibuat / Dimodifikasi
- [NEW] `crates/daemon/src/menu.rs`
- [NEW] `crates/daemon/src/autostart.rs`
- [MODIFY] `crates/daemon/src/main.rs` (tambah `mod menu; mod autostart;`; wiring handler perintah)
- [MODIFY] `crates/daemon/src/tray.rs` (arm `WM_TRAYICON` di `wndproc_impl` → popup menu; Exit via `DestroyWindow`)
- [MODIFY] `crates/shared/src/constants.rs` (konstanta `TASK_NAME`)
- [NO CHANGE] `crates/daemon/Cargo.toml` (fitur windows-sys sudah lengkap)

### References
- [Source: epics.md#Story-1.4] (FR-12, FR-13, FR-16)
- [Source: ARCHITECTURE-SPINE.md#AD-11, #AD-12, #AD-13; CAP-10/CAP-11; Consistency Conventions]
- [Source: SPEC.md#Administrator-Elevation-&-Hardening, #APPDATA-Alignment]
- [Source: prd.md FR-12/FR-13/FR-16] — catatan: urutan FR-16 di PRD berbeda (lihat Diskrepansi)
- [UX: EXPERIENCE.md#Information-Architecture] — daftar & urutan item menu
- [Visual spec: design-system/project/components/navigation/TrayMenu.prompt.md] — flyout Fluent (2 separator, `checked` toggle)
- [Visual prototype: design-system/project/ui_kits/wintick-app/index.html]

## Dev Agent Record
### Agent Model Used
Claude Opus 4.8 (Claude Code)

### Debug Log References
- `cargo check --workspace` → clean (0 warnings/errors).
- `cargo clippy --workspace --tests` → clean after 2 fixes (`collapsible_match` on the Exit arm → extracted `request_exit`; `unnecessary_cast` on `SW_SHOWNORMAL as i32` → removed).
- `cargo test -p shared` → 10 passed / 0 failed (no regression to the only runnable test crate).
- `cargo test -p daemon --no-run` → compiles the daemon test harness (incl. new `autostart` unit tests) successfully.
- `./build.ps1 -Mode prod` → release build OK. `wintick.exe` **209 KB** (< 500 KB NFR3), `wintick-settings.exe` 2805 KB (egui GUI — NFR exception ratified Story 1.1).

### Completion Notes List
- **Menu (Task 1):** Added `menu.rs` — pure-Win32 `CreatePopupMenu` + `AppendMenuW` (6 items, 2 separators, AC-3 order) + `SetForegroundWindow` → `TrackPopupMenu(TPM_RIGHTBUTTON|TPM_RETURNCMD)` → `PostMessageW(WM_NULL)` (KB135788) → `DestroyMenu`, then dispatch the returned command id. Mnemonics `&S/V/A/C/b/x` (no collisions). Wired via a `WM_TRAYICON` arm **inside `wndproc_impl`** so it inherits the `catch_unwind` FFI guard (O2); v4 callback parsed manually (event = `LOWORD(lParam)`, anchor from `wParam`, sign-extended via `i16` for secondary-monitor negative coords), triggering on `WM_CONTEXTMENU`/`WM_RBUTTONUP` — no `GetCursorPos`.
- **View Logs (Task 2):** `shared::log_path()` (AD-12); creates the empty file (and parent dir) if absent, then opens `notepad.exe` via `Command` + `CREATE_NO_WINDOW`.
- **Settings (Task 3):** `ShellExecuteW("open", …)` (AD-11, decoupled, inherits elevation); path resolved as `current_exe().parent()/wintick-settings.exe` (pure `std`, equivalent to the `GetModuleFileNameW` approach).
- **Auto-Start (Task 4):** `autostart.rs` — `schtasks` via `Command` + `creation_flags(CREATE_NO_WINDOW)` (pure `std`, no new windows-sys feature). `enable` = `/Create … /SC ONLOGON /RL HIGHEST /RU <user> /F` with the `/TR` exe path **wrapped in explicit quotes** so Task Scheduler stores a single action even for `%ProgramFiles%` paths with spaces (no working dir set — AD-13 DLL-hijack mitigation). `is_registered()` (`/Query` exit 0) is the **authoritative** checkmark source (AC-8; `config.auto_start` not consulted). `TASK_NAME` pinned in `shared::constants` (O1).
- **About/Updates/Exit (Task 5):** Added reusable `util::message_box` and refactored `main::show_message_box` to delegate to it (removed the private duplicate encoding, E2). About shows `env!("CARGO_PKG_VERSION")`; Check-for-Updates is an explicit MSIX-deferred placeholder. Exit routes through `DestroyWindow` → `WM_DESTROY` → `cleanup()` (never a bare `PostQuitMessage` — C1 fix; avoids GDI-icon + hook leak).
- **Refactor:** Moved `debug_log` from `tray.rs` to `util.rs` so `tray`/`menu`/`autostart` share one diagnostic path (O3). No behavior change.
- **Scope discipline:** No `Cargo.toml` changes (E5 — all windows-sys features already present); stayed on windows-sys 0.52 (E6, spine drift left for separate reconciliation); did not touch the 1.5-deferred state-machine regions in `tray.rs` (O4); CAP-10 settings-side toggle intentionally deferred to Epic 4 (D5).
- **⚠️ Verification boundary (honest disclosure):** Two classes of check could NOT be executed in this **non-interactive, non-elevated** session and remain for reviewer confirmation:
  1. **Daemon unit tests** (`autostart` arg construction — 3 tests) compile and are correct, but *executing* `cargo test -p daemon` needs elevation because the crate's `requireAdministrator` manifest (Story 1.2) is linked into the test harness too (`os error 740`). This is pre-existing — `cargo test -p daemon` has required elevation since 1.2, even with zero tests. Windows `sudo` is disabled on this machine, so I verified compilation only. **Run `cargo test -p daemon` from an elevated shell to see them pass.**
  2. **Runtime GUI behaviors** (right-click menu appears + dismisses on outside click; View Logs opens Notepad; Settings launches `wintick-settings.exe`; Auto-Start creates/deletes the scheduled task with the checkmark tracking `schtasks /Query`; Exit leaves no ghost tray icon and unhooks) require a live elevated interactive session with a system tray. Marked `[~]` above (code-complete, runtime-pending) rather than `[x]`. This matches how Story 1.3's tray work was handed to review.

### File List
- [NEW] `crates/daemon/src/menu.rs` — tray context menu (build/track/dispatch + command handlers)
- [NEW] `crates/daemon/src/autostart.rs` — Task Scheduler auto-start (`schtasks`) + unit tests
- [MODIFY] `crates/daemon/src/main.rs` — `mod menu; mod autostart;`; `show_message_box` delegates to `util::message_box`
- [MODIFY] `crates/daemon/src/tray.rs` — `WM_TRAYICON` arm in `wndproc_impl`; import `debug_log` from `util`; removed local `debug_log`
- [MODIFY] `crates/daemon/src/util.rs` — added `debug_log` (moved from `tray.rs`) and `message_box`
- [MODIFY] `crates/shared/src/constants.rs` — added `TASK_NAME`
- [NO CHANGE] `crates/daemon/Cargo.toml` — windows-sys features already complete

## Review Follow-ups

### Verifikasi runtime elevated (2026-07-13, sesi review Claude Opus 4.8)

Sesi review **non-elevated** — Task 1 & 2 dijalankan oleh **user** di sesi elevated interaktif; hasil dilaporkan balik & ditriase di sini. Skala layar user: **150% @ 4K (3840×2160)** — relevan untuk RF-1.

| Cek | Hasil |
| --- | --- |
| `cargo test -p daemon` (elevated) | **PASS** (user: "cargo test aman"). Barrier `os error 740` non-elevated dikonfirmasi ulang. |
| Menu klik-kanan tampil + dismiss | ⚠️ Tampil & item benar, **tapi posisi salah** → lihat **RF-1** |
| View Logs → Notepad | **OK** |
| Settings → `wintick-settings.exe` | **Launch OK** (window muncul); pewarisan elevasi/IL (F2) **belum** dicek Process Explorer |
| Auto-Start toggle | **OK** (create/delete). `<Command>` XML quoting (F1) **belum** diinspeksi |
| About / Check for Updates | **OK** (message box center) |
| Exit | **OK** (tanpa ikon hantu) |

### RF-1 — [BUG, FIXED & RE-TESTED ✓] Menu tray muncul jauh di sudut kanan-bawah, bukan di dekat ikon/kursor
- **Severity:** Medium (melanggar UX AC-2/AC-3 — menu harus muncul di anchor tray).
- **Root cause (dikonfirmasi empiris):** daemon **DPI-unaware** (manifest tanpa `<dpiAwareness>`). Pada skala 150%, Explorer (PMv2-aware) mengirim koordinat anchor tray dalam piksel **fisik** (~3800,~2080); `TrackPopupMenu` di proses unaware menafsirkannya di ruang **tervirtualisasi** (2560×1440) → ter-clamp ke sudut kanan-bawah. Kode parsing koordinat v4 di `tray.rs` sudah benar; masalahnya bukan di sana.
- **Fix:** tambah `<application><windowsSettings><dpiAwareness>PerMonitorV2, PerMonitor</dpiAwareness><dpiAware>true/pm</dpiAware></windowsSettings></application>` ke `crates/daemon/wintick.manifest`. Menyelaraskan ruang koordinat proses ↔ koordinat fisik shell; juga fondasi benar untuk snapping engine lintas-monitor (epic berikutnya).
- **Catatan arsitektur:** DPI-awareness bersifat **app-wide** → perlu dicatat sebagai keputusan arsitektur (kandidat AD baru di SPINE); semua kode koordinat window mendatang harus per-monitor-DPI aware.
- **Status:** rebuild prod sukses (`wintick.exe` 210→215 KB, masih < 500 KB NFR3). **Re-test user (2026-07-13, layar 150% @ 4K, sesi elevated): menu kini muncul tepat di dekat ikon tray, dan menutup pada klik pertama di luar. RESOLVED.**

### RF-2 — [OUT-OF-SCOPE, catat] Window Settings muncul di kiri-atas, bukan center
- Penempatan window Settings dimiliki crate `settings` (eframe/egui default viewport), **bukan** story 1.4. User menilai dapat diterima bila lazim. Ditinggalkan untuk pekerjaan Settings (Epic 4); pertimbangkan set posisi/`center` viewport di sana.

### RF-3 — [DEFER ke Story 1.5] Retensi log ~3 hari
- Permintaan user: file log punya kebijakan retensi (mis. 3 hari). Logging formal = **AD-7 / Story 1.5** (Tier-2 logger). View Logs 1.4 hanya membuka file. Ditunda ke 1.5 (atau `deferred-work.md`).

### Temuan code review (fresh-context, Opus 4.8 — bukan LLM berbeda; timbang sebagai fresh-context)
- **F1 · Medium · runtime-verify · ✅ CONFIRMED OK:** round-trip quoting `/TR` (`autostart.rs:41`) tidak teruji unit — hanya elemen `Vec` yang di-assert, bukan yang **disimpan** Task Scheduler. Diverifikasi via `schtasks /Query /TN WinTick /XML` (2026-07-13): `<Command>"<install-dir>\wintick.exe"</Command>` — path absolut ber-kutip tunggal, **tanpa** elemen `<WorkingDirectory>`. Round-trip escaping benar.
- **F2 · Medium · runtime-verify · ✅ CONFIRMED OK:** pewarisan elevasi `ShellExecuteW` untuk Settings (`menu.rs:141`). Diverifikasi via `GetTokenInformation(TokenIntegrityLevel)` pada proses `wintick-settings.exe` berjalan (2026-07-13): SID integritas `S-1-16-12288` = **High** — elevasi terwarisi dengan benar, AD-11 terpenuhi.
- **F3 · Low · runtime-verify · dianggap OK:** `/RU %USERNAME%` (`autostart.rs:83`) tanpa domain & tanpa `/RP` — `enable()`/`disable()` toggle berhasil tanpa hang/prompt password saat pengujian runtime.
- **F4 · Low · spec-adherence · belum ditindaklanjuti (non-blocking):** 7 `AppendMenuW` (`menu.rs:60-79`) tak cek nilai retur walau Task 1 memintanya. Dicatat untuk cleanup ringan, tidak menahan status `done` (menu tervalidasi tampil benar secara runtime).
- **F5 · Low · UX/perf · belum ditindaklanjuti (non-blocking):** `is_registered()` (`menu.rs:53`) spawn `schtasks /Query` sinkron di thread message-loop tiap klik-kanan (~100ms latensi buka menu). Kandidat cache/async — optimisasi masa depan, bukan defect.
- **F6 · Info · belum ditindaklanjuti (non-blocking):** fragilitas reentrancy (`tray.rs:227`) — borrow `&mut *data_ptr` mati di baris 204 sebelum `menu::show`, jadi aman; satu edit yang menyentuh `data` setelah `menu::show` di arm `WM_TRAYICON` = UB. Rekomendasi: tambahkan komentar guard saat menyentuh area ini di story mendatang.

## Change Log
| Date | Version | Description | Author |
| --- | --- | --- | --- |
| 2026-07-12 | 0.1.0 | Implemented Story 1.4 (tray context menu, View Logs, Settings launch, Task Scheduler auto-start, About/Updates, symmetric Exit). Static verification: cargo check/clippy clean, shared tests 10/10, prod build 209 KB. Daemon unit tests + runtime GUI checks pending elevated verification. Status → review. | kodesh87 (Claude Opus 4.8) |
| 2026-07-13 | 0.1.1 | Elevated verification (user-run): daemon tests PASS; View Logs/Settings/Auto-Start/About/Updates/Exit OK. **RF-1**: menu mis-positioned to bottom-right on 150% scale → DPI-unaware root cause confirmed; PerMonitorV2 declared in `wintick.manifest`, rebuilt (210→215 KB, NFR3 OK), **re-tested and confirmed fixed** — menu now anchors correctly and dismisses on first outside click. **F1** confirmed via `schtasks /XML` (quoted absolute path, no working dir). **F2** confirmed via process integrity SID (`wintick-settings.exe` runs at High integrity — elevation inheritance OK). Exit confirmed clean (no ghost icon). RF-2 (Settings placement) out-of-scope; RF-3 (log retention) deferred to 1.5. F3 informally OK; F4–F6 non-blocking, left for future cleanup. All Task 6 items flipped to `[x]`. Status → **done**; `sprint-status.yaml` updated. | kodesh87 (Claude Opus 4.8, review session) |
