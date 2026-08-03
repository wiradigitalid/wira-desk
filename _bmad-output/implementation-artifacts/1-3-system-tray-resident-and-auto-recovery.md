---
baseline_commit: 720d155
---
# Story 1.3: System Tray Resident & Auto-Recovery

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a pengguna desktop,
I want WinTick bersembunyi di System Tray tanpa antarmuka konsol dan bisa pulih sendiri jika OS error,
so that layar saya bersih dari jendela background.

## Acceptance Criteria

1. **Given** daemon berjalan di background
2. **When** proses `explorer.exe` (Windows Taskbar) di-restart paksa
3. **Then** daemon menangkap pesan `TaskbarCreated`
4. **And** ikon Tray WinTick otomatis dirender kembali tanpa crash.

## Tasks / Subtasks

- [x] Task 1: Buat hidden message window sebagai jantung message loop daemon (AC: 1, 3)
  - [x] Modul baru `crates/daemon/src/tray.rs`
  - [x] Daftarkan window class `WinTickDaemonHiddenWindow` (dari `shared::constants::DAEMON_WINDOW_CLASS`) via `RegisterClassW` dengan `WndProc` kustom
  - [x] Buat window tersembunyi (tidak `WS_VISIBLE`, judul `DAEMON_WINDOW_TITLE`) via `CreateWindowExW` — dipakai juga oleh Settings untuk `WM_APP_RELOAD_CONFIG` (AD-5)
  - [x] Pindahkan message loop dari `main.rs` ke daemon message loop yang men-dispatch ke `WndProc`
- [x] Task 2: Registrasi & rendering ikon System Tray (AC: 4, UX-DR2)
  - [x] Bangun base `HICON` WinTick via GDI (di `icon.rs`) — glyph minimalis, tanpa aset biner eksternal
  - [x] Isi struct `NOTIFYICONDATAW` (uID tetap, `hWnd` = hidden window, `uCallbackMessage` = `WM_APP` + offset tray, `hIcon`, `szTip` = "WinTick") dan panggil `Shell_NotifyIconW(NIM_ADD, ...)`
  - [x] Set `uVersion = NOTIFYICON_VERSION_4` via `NIM_SETVERSION`
  - [x] Hapus ikon (`NIM_DELETE`) saat `WM_DESTROY`/shutdown agar tidak ada ikon hantu
- [x] Task 3: Auto-recovery `TaskbarCreated` (AC: 2, 3, 4, AD-10)
  - [x] Simpan hasil `RegisterWindowMessageW("TaskbarCreated")` (message id dinamis)
  - [x] Di `WndProc`, jika `msg == taskbar_created_id`, panggil ulang `Shell_NotifyIconW(NIM_ADD, ...)` untuk me-render ulang ikon
- [x] Task 4: Integrasi dengan `main.rs` (AC: All)
  - [x] Ganti pemanggilan `shared::hello_shared()` yang usang; `main` memanggil `tray::run_message_loop()` setelah hook terpasang
  - [x] Pastikan `UnhookWindowsHookEx` dan `NIM_DELETE` dipanggil saat loop keluar
- [x] Task 5: Verifikasi & pengujian manual (AC: All)
  - [x] `cargo build` (dev) dan `build.ps1 -Mode prod` sukses tanpa warning baru
  - [x] Jalankan daemon, konfirmasi ikon muncul di tray tanpa jendela/console
  - [x] `taskkill /f /im explorer.exe` lalu jalankan ulang explorer; konfirmasi ikon muncul kembali tanpa crash

## Dev Notes

### Arsitektur & Aturan Terkait
- **AD-1 (Actor / Message-Passing):** Worker/Main thread memiliki state tray secara eksklusif. Tidak ada shared mutable state.
- **AD-10 (Explorer Crash Recovery):** Message loop daemon WAJIB mendengarkan broadcast `TaskbarCreated` dan me-registrasi ulang ikon tray saat diterima. Ini satu-satunya mekanisme pemulihan ikon.
- **AD-11 / UX-DR1:** Daemon tidak boleh punya UI selain ikon tray. `#![windows_subsystem = "windows"]` sudah aktif.
- **NFR1/NFR3:** Gunakan `windows-sys` C-FFI murni. Bangun ikon via GDI runtime, hindari menambah aset yang membengkakkan biner.

### Detail Teknis
- `NOTIFYICONDATAW` memerlukan feature `windows-sys` `"Win32_UI_Shell"`. `Shell_NotifyIconW` ada di `Win32::UI::Shell`.
- GDI icon: feature `"Win32_Graphics_Gdi"` + `"Win32_UI_WindowsAndMessaging"` (`CreateIconIndirect`, `ICONINFO`).
- `RegisterWindowMessageW` + `GetModuleHandleW` sudah tersedia.
- Callback message tray dipilih `WM_APP + 10` (di luar range reload/health) untuk menghindari tabrakan dengan `WM_APP_RELOAD_CONFIG` dsb.
- Window class harus punya `hInstance` valid dan `lpfnWndProc` = fungsi `extern "system"`.

### File yang Perlu Dimodifikasi / Dibuat
- [NEW] `crates/daemon/src/tray.rs` — window class, hidden window, tray icon add/delete, TaskbarCreated handling, message loop.
- [NEW] `crates/daemon/src/icon.rs` — pembuatan `HICON` via GDI (base + hook untuk overlay di Story 1.5).
- [MODIFY] `crates/daemon/src/main.rs` — panggil `tray::run_message_loop`, bersihkan `hello_shared`.
- [MODIFY] `crates/daemon/Cargo.toml` — tambah feature `Win32_UI_Shell`, `Win32_Graphics_Gdi`.

### References
- [Source: architecture ARCHITECTURE-SPINE.md#AD-10] (TaskbarCreated Listener)
- [Source: epics.md#Story-1.3] (FR-9, FR-10)
- [Source: DESIGN.md#System-Tray-Icon] (UX-DR2)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 (Claude Code)

### Debug Log References

Backfilled during closing ceremony. Original implementation shipped in commit `d614953` — session was terminated before Dev Agent Record was populated. Verification (2026-07-09):
- `cargo build` (dev): finished clean, no warnings (3m54s cold).
- `cargo clippy`: clean, no warnings.
- `build.ps1 -Mode prod`: finished in 26.8s. Artifacts:
  - `wintick.exe` = **114 KB** (NFR3 target < 500 KB ✅)
  - `wintick-settings.exe` = 2805 KB
- No `TODO` / `unimplemented!` / `todo!` markers in `crates/`.

### Completion Notes List

- **Task 1 (hidden message window):** Implemented `crates/daemon/src/tray.rs`. `RegisterClassW` registers `WinTickDaemonHiddenWindow` with a custom `wndproc`. Window is top-level (`WS_OVERLAPPED`, never `WS_VISIBLE`) — required so it receives the `TaskbarCreated` broadcast (message-only windows do not). Message loop lives in `run_message_loop()` and dispatches via `TranslateMessage` / `DispatchMessageW`.
- **Task 2 (tray icon):** `NOTIFYICONDATAW` populated with `NIF_ICON | NIF_MESSAGE | NIF_TIP`, `uCallbackMessage = WM_APP + 10` (avoids collision with `WM_APP_RELOAD_CONFIG` etc.), `szTip = "WinTick"`. `NIM_ADD` + `NIM_SETVERSION` to `NOTIFYICON_VERSION_4`. `NIM_DELETE` fires from `WM_DESTROY` so no ghost icons on shutdown. Icons rasterised at runtime via GDI (`icon.rs`) — three variants (`base`, `with_warning`, `with_critical`) preloaded for AD-7 / UX-DR2 state machine; only `Normal` used in this story.
- **Task 3 (auto-recovery, AD-10):** `RegisterWindowMessageW("TaskbarCreated")` cached in `TrayData.taskbar_created`. `wndproc` short-circuits on `msg == taskbar_created` and re-runs `add_icon()` to restore the tray icon after Explorer restart.
- **Task 4 (main integration):** `main.rs` removed the stale `shared::hello_shared()` call and now hands the keyboard hook handle to `tray::run_message_loop(hook_handle)`. Cleanup path: `WM_DESTROY` → `NIM_DELETE` → `DestroyIcon` (all three) → `UnhookWindowsHookEx(hook_handle)` → `PostQuitMessage(0)`. Loop reclaims `TrayData` box on exit.
- **Task 5:** Static build verification done (dev + prod). Runtime verification (tray icon visible; Explorer crash → icon recovers) still requires an interactive elevated session — pending user.
- **Architecture alignment:** AD-1 respected (single thread owns tray state via `GWLP_USERDATA`), AD-10 satisfied (dynamic message id + re-`NIM_ADD`), AD-11 / UX-DR1 satisfied (`#![windows_subsystem = "windows"]` — no console), NFR3 satisfied (114 KB, no external icon asset).
- **Scope note:** Commit `d614953` also introduced the shared crate foundation (`commands.rs`, `config.rs`, `shortcut.rs`) that is out of Story 1.3's declared file scope but lands here as prerequisite scaffolding. Called out here for the code reviewer.

### File List

Story-1.3 scope (declared in Dev Notes):
- `crates/daemon/src/tray.rs` — NEW — window class, hidden window, tray icon add/modify/delete, `TaskbarCreated` handler, message loop.
- `crates/daemon/src/icon.rs` — NEW — GDI-rasterised `HICON` (base / warning / critical).
- `crates/daemon/src/util.rs` — NEW — UTF-16 helpers (`wide`, `fill_wide_buf`) used by tray/icon.
- `crates/daemon/src/main.rs` — MODIFIED — removed `hello_shared`, wired `tray::run_message_loop(hook_handle)`.
- `crates/daemon/Cargo.toml` — MODIFIED — added `Win32_UI_Shell`, `Win32_Graphics_Gdi` features.

Prerequisite scaffolding landed in same commit (out of declared scope, flag to reviewer):
- `crates/daemon/build.rs` — NEW — `embed-resource` invocation.
- `crates/shared/src/constants.rs` — NEW — `DAEMON_WINDOW_CLASS`, `DAEMON_WINDOW_TITLE`, `WM_APP_*`, etc.
- `crates/shared/src/commands.rs` — NEW — command enum scaffolding (future story).
- `crates/shared/src/config.rs` — NEW — config scaffolding (future story).
- `crates/shared/src/shortcut.rs` — NEW — shortcut parsing scaffolding (future story).
- `crates/shared/src/lib.rs` — MODIFIED — module re-exports.
- `crates/settings/src/main.rs` — MODIFIED — placeholder wiring.
- `build.ps1` — MODIFIED — added prod-mode + MSVC env import.

## Change Log

- 2026-07-09 — Closing-ceremony backfill after mid-session termination. All Story-1.3 code was already committed in `d614953` (2026-07-08). Verified tasks 1–4 and 5a against the committed code, populated Dev Agent Record + File List, checked completed boxes.
- 2026-07-09 — User `kodesh87` executed manual runtime tests T5.2 (tray icon visible, no console window) and T5.3 (taskkill explorer + relaunch → tray icon recovers, no crash). Both PASS. Status raised to `review`; ready for `bmad-code-review`.
- 2026-07-09 — `bmad-code-review` run (Blind Hunter + Edge Case Hunter + Acceptance Auditor, all 3 layers). 12 patch findings, 4 deferred, 10 dismissed. See Review Findings section below.
- 2026-07-09 — All 12 patches applied. Files touched: `crates/daemon/src/tray.rs` (rewrite for `catch_unwind` + UIPI filter + retval checks + `-1` cleanup path + `WS_EX_TOOLWINDOW` + `NIF_SHOWTIP`), `crates/daemon/src/icon.rs` (zero-init mask bitmap + `GetDC(0)` guard), `crates/daemon/src/util.rs` (`N == 0` guard). Verification: `cargo build` clean, `cargo clippy -- -D warnings` clean, `build.ps1 -Mode prod` clean → `wintick.exe` = 115 KB (was 114 KB, still ≪ NFR3 target 500 KB). Runtime re-verification pending user (behavior-affecting changes: `NIF_SHOWTIP` → hover tooltip now shows; `ChangeWindowMessageFilterEx` → defensive UIPI hardening).

## Tasks / Subtasks — Review Findings

### [Review][Patch] (12) — actionable now

- [x] **[Med] UIPI hardening: `ChangeWindowMessageFilterEx` for TaskbarCreated** [crates/daemon/src/tray.rs:216] — daemon runs elevated (Story 1.2); Explorer runs at Medium integrity. Manual T5.3 passed on Windows 11 (auto-allows TaskbarCreated), but hardened Windows configs will silently block the broadcast, breaking AD-10 auto-recovery. Call `ChangeWindowMessageFilterEx(hwnd, taskbar_created, MSGFLT_ALLOW, NULL)` right after `CreateWindowExW` succeeds.
- [x] **[Med] Add `NIF_SHOWTIP` to `uFlags`** [crates/daemon/src/tray.rs:82] — under `NOTIFYICON_VERSION_4`, the standard hover tooltip is suppressed unless `NIF_SHOWTIP` is set. `szTip="WinTick"` is currently dead.
- [x] **[Med] Check `Shell_NotifyIconW(NIM_ADD, ...)` return** [crates/daemon/src/tray.rs:94, 146] — both the initial add and the `TaskbarCreated` re-add discard the BOOL return. On race with Explorer startup, `NIM_ADD` can return FALSE and the icon never appears with no diagnostic. Log to Windows event log / trace, and consider a bounded retry (e.g. 3× at 500 ms).
- [x] **[Med] Wrap `wndproc` body in `std::panic::catch_unwind`** [crates/daemon/src/tray.rs:124-174] — `extern "system"` fn; panic unwinding through FFI = UB. Release profile has `panic="abort"` (workspace Cargo.toml:13), but debug builds still unwind. Wrap the body, return `DefWindowProcW` on caught panic.
- [x] **[Med] Distinguish `GetMessageW` error (`-1`) from `WM_QUIT` (`0`)** [crates/daemon/src/tray.rs:232-235] — loop currently exits on `r <= 0`. On `-1` (Win32 message loop error), `WM_DESTROY` never dispatches, so `NIM_DELETE` / `DestroyIcon` / `UnhookWindowsHookEx` are skipped. Split the branch: on `-1`, log then call cleanup manually.
- [x] **[Med] Check `Shell_NotifyIconW(NIM_SETVERSION, ...)` return** [crates/daemon/src/tray.rs:96] — if this fails, click semantics fall back to pre-v4 format. Story 1.4 (context menu) will read wrong coordinates from `WM_TRAYICON`.
- [x] **[Low] Guard `taskbar_created != 0` before message match** [crates/daemon/src/tray.rs:145] — `RegisterWindowMessageW` returning 0 (documented failure) would make `msg == data.taskbar_created` match `WM_NULL` and spuriously re-add the icon.
- [x] **[Low] Zero-initialize the mask bitmap** [crates/daemon/src/icon.rs:156] — `CreateBitmap(..., NULL)` produces undefined mask bits. Modern shell uses alpha, but legacy `DrawIcon` paths (Alt-Tab thumbnails, third-party shells) will show garbage. Pass a 128-byte zero buffer.
- [x] **[Low] Guard `fill_wide_buf` against `N == 0`** [crates/daemon/src/util.rs:12-19] — `N - 1` underflows to `usize::MAX`; out-of-bounds write on any zero-length buffer. No current caller uses `N=0` but the utility is public.
- [x] **[Low] Unhook keyboard hook on `CreateWindowExW` failure branch** [crates/daemon/src/tray.rs:218-225] — the failure path destroys the three icons but never `UnhookWindowsHookEx`. OS reclaims on `ExitProcess`, but the asymmetry contradicts the story's own comment on symmetric cleanup.
- [x] **[Low] Check `GetDC(0)` return** [crates/daemon/src/icon.rs:142] — if `NULL`, `ReleaseDC(0, NULL)` is UB per API contract. Downstream `CreateDIBSection` also fails but the guard is missing.
- [x] **[Low] Add `WS_EX_TOOLWINDOW` extended style** [crates/daemon/src/tray.rs:204] — window is never shown, but omitting the flag leaves it Alt-Tab eligible if any future path accidentally `ShowWindow`s it. Consistent with UX-DR1 "invisibility over presence."

### [Review][Defer] (4) — punted to Story 1.5 or later

- [x] **[Med] `HICON = 0` fallback** [crates/daemon/src/tray.rs:196-206] — deferred to Story 1.5. AD-7 Tier-1 startup-fatal protocol lands there and will handle GDI exhaustion via a single `MessageBox` + exit.
- [x] **[Med] State-machine transition guards** [crates/daemon/src/tray.rs:150-160] — deferred to Story 1.5. No guard against Critical→Warning downgrade (a late warning after hook death would overwrite the Tier-3 X icon); no Critical→Normal recovery message defined. Story 1.5 introduces the actual message dispatchers and can codify the transitions.
- [x] **[Low] `NIM_MODIFY` return in `set_state`** [crates/daemon/src/tray.rs:104] — deferred to Story 1.5. Silent no-op if the icon was already removed by an Explorer restart. Story 1.5 will re-`NIM_ADD` if `NIM_MODIFY` returns FALSE.
- [x] **[Low] DPI awareness (`WM_DPICHANGED`)** [crates/daemon/src/tray.rs] — deferred, cross-cutting. Icons fixed at 32×32; monitor changes leave the tray glyph slightly blurry. Not blocking any AC.

### [Review][Dismissed] (10, for the record) — noise / handled / dupes

- Dupes across reviewers (NIM_ADD retry, catch_unwind, fill_wide_buf N==0, RWMW=0, HICON=0) — merged.
- `hwnd = 0` race between `CreateWindowExW` and `data.hwnd = hwnd` — no `TaskbarCreated` producer active in that microsecond window; benign.
- `RegisterClassW` return discarded — a failure surfaces immediately via `CreateWindowExW == 0` and the function bails.
- `DestroyIcon` ordering vs Shell repaint at shutdown — Shell stops using the icon on `NIM_DELETE`.
- `shared::hello_shared()` — verified fully removed (crates/shared/src/lib.rs no longer defines it).
- Scope creep (shared `commands.rs`/`config.rs`/`shortcut.rs`, settings `main.rs`, daemon `build.rs`) — disclosed in Dev Agent Record → Completion Notes; `build.rs` is legitimate Story 1.2 residue (embed-resource for `wintick.rc` UAC manifest); the shared modules are prerequisite scaffolding used by future stories and cannot be usefully rolled back.
- `SetWindowLongPtrW` return check — intentionally not distinguished (initial `GWLP_USERDATA = 0` makes ambiguity by convention).
