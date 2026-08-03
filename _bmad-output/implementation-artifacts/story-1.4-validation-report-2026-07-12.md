# Story Context Validation Report — Story 1.4

- **Story:** 1.4 — Tray Context Menu, View Logs & Auto-Start
- **File validated:** `_bmad-output/implementation-artifacts/1-4-tray-context-menu-view-logs-and-auto-start.md`
- **Baseline commit:** 720d155
- **Validated:** 2026-07-12 (fresh-context re-analysis; kodesh87)
- **Method:** 4 parallel research passes (epics+PRD, architecture spine+SPEC, previous story 1.3 + live daemon code, UX + design-system). All load-bearing code claims re-verified directly against source.
- **Verdict:** **READY-FOR-DEV WITH FIXES.** No blocker to starting, but 4 Critical items would cause resource leaks / wrong behavior / compile failure if implemented as written, and 5 spec discrepancies need a decision before or during dev.

---

## 🚨 CRITICAL (Must Fix) — would cause a regression, wrong behavior, or compile failure

### C1. Exit item leaks GDI icons + the keyboard hook (breaks the cleanup contract)
- **Story says** (Task 5, line 42): `Exit → NIM_DELETE lalu PostQuitMessage(0)`.
- **Reality:** `TrayData` has **no `Drop` impl**; `cleanup()` (`crates/daemon/src/tray.rs:92-112`) does `delete_icon` + `DestroyIcon`×3 + `UnhookWindowsHookEx`, and runs **only** from `WM_DESTROY` (`tray.rs:231-235`) or the `GetMessageW == -1` error path. Calling `PostQuitMessage(0)` directly from the Exit handler **skips `DestroyIcon`×3 and `UnhookWindowsHookEx`** → GDI + hook leak on every clean exit.
- **Fix:** Exit must call **`DestroyWindow(data.hwnd)`** → triggers `WM_DESTROY` → `cleanup()` (does `NIM_DELETE` + `DestroyIcon` + unhook) → `PostQuitMessage(0)`. Do **not** post-quit directly.

### C2. Right-click detection + popup coordinates are wrong for NOTIFYICON_VERSION_4
- **Story says** (Task 1, line 28): dispatch when `lParam = WM_RBUTTONUP/WM_CONTEXTMENU`.
- **Reality:** the icon runs under **`NOTIFYICON_VERSION_4`** (`tray.rs:140`). Under v4 the callback packs data differently: **event id = `LOWORD(lParam)`**, icon uID = `HIWORD(lParam)`, and the **anchor X/Y come from `wParam`** (X = LOWORD, Y = HIWORD) — **not** `GetCursorPos`. `windows-sys` has no `GET_X_LPARAM`/`GET_Y_LPARAM`, so extract manually. `WM_CONTEXTMENU` (in `LOWORD(lParam)`) is the primary trigger (covers both right-click and the keyboard menu key); `WM_RBUTTONUP` is secondary.
- Story 1.3's own code review flagged this exact risk for 1.4 (see `tray.rs:141-145` NIM_SETVERSION note).
- **Also:** there is currently **no `WM_TRAYICON` arm** in `wndproc_impl`'s `match` (`tray.rs:220-237`) — a click does nothing today. 1.4 must add `m if m == WM_TRAYICON => { … }` **inside `wndproc_impl`** (so it inherits the `catch_unwind` FFI guard at `tray.rs:177-193`). Constant already exists: `WM_TRAYICON = WM_APP + 10` (`tray.rs:44`, `pub`).

### C3. AC-6 "Start in dikosongkan" is not executable via `schtasks` CLI and contradicts Task 4
- **Story:** AC-6 (line 21) requires the scheduled task's **`Start in`** parameter be emptied; Task 4 (line 38) says "Task Scheduler tidak punya 'Start in', jadi gunakan path absolut."
- **Reality:** `schtasks.exe` has **no `Start in` flag** — `<WorkingDirectory>` is a GUI/XML-only field. The concrete, verifiable mitigation (per **AD-13** and SPEC "Administrator Elevation & Hardening") is an **absolute `/TR` executable path with no working directory set**.
- **Fix:** Rewrite AC-6 to the actual mechanism: *"Task action uses the absolute exe path (`/TR "<full path>"`) and sets no working directory, mitigating DLL Hijacking."* Make AC-6 and Task 4 say the same thing.

### C4. New modules won't compile — `main.rs` module declarations omitted
- **Story** creates `crates/daemon/src/menu.rs` and `crates/daemon/src/autostart.rs` (lines 55-56) but the "Files to modify" list (lines 54-58) names only `tray.rs` + `Cargo.toml`.
- **Reality:** `main.rs` declares only `mod icon; mod tray; mod util;` (`main.rs:3-5`). Without adding **`mod menu; mod autostart;`** to `main.rs`, the new modules are not part of the crate.
- **Fix:** add `[MODIFY] crates/daemon/src/main.rs` (add `mod menu; mod autostart;` + wire the command handlers).

---

## ⚠️ SPEC DISCREPANCIES — need a decision (may change the visible menu)

### D1. Menu order: PRD FR-16 contradicts epics + UX + the story
- **PRD FR-16** (prd.md:56): `Settings... → Check for Updates... → About → View Logs → Auto-Start → Exit`
- **Epics FR-16, epics Story 1.4 AC, UX EXPERIENCE.md, and the story:** `Settings... → View Logs → Auto-Start → Check for Updates... → About → Exit`
- The same 6 items appear everywhere; only the order differs. The story matches **2 of 3** source families (epics + UX) — PRD FR-16 is the outlier.
- **Recommendation:** keep the story's order (epics/UX authority) and reconcile PRD FR-16 separately. **Needs your confirmation** — it's the only remaining PRD contradiction.

### D2. Separators / grouping
- Design-system `TrayMenu.jsx` (lines 12, 15) inserts **two separators** (after Auto-Start, after About) → 3 groups. Story + EXPERIENCE.md are a **flat** list.
- **Recommendation:** adopt the two separators (cheap; matches the Fluent design intent). `AppendMenuW(MF_SEPARATOR, …)`.

### D3. Ellipsis character
- `Settings...`/`Check for Updates...` (ASCII three dots, in EXPERIENCE.md/PRD/AC) vs `Settings…`/`Check for Updates…` (U+2026, in design-system JSX). Win32 menu strings are literal — these are different strings.
- **Recommendation:** pin one. Suggest ASCII `...` (simplest, matches the AC text) unless Fluent parity with U+2026 is wanted.

### D4. "Check for Updates..." and "About" have no defined behavior in ANY source doc
- FR-16 mandates both items but no PRD/epics/UX text defines their action (only tangential MSIX note, prd.md:83).
- Story decides: About → `MessageBoxW` version/info; Check for Updates → placeholder MessageBox (update mechanism deferred).
- **Recommendation:** accept the story's placeholder decision but state it explicitly in the AC (behavior minimal / update mechanism deferred to distribution/MSIX). Confirm acceptable.

### D5. CAP-10 second home (`settings/ui_settings.rs`) not addressed
- CAP-10 map + FR-13 place the Auto-Start toggle in **both** `daemon/tray.rs` and `settings/ui_settings.rs`. The story implements only the tray surface.
- **Recommendation:** add a one-line note that the settings-side toggle is **deferred to Epic 4** (Story 4.x) — so the omission is intentional, not a miss.

---

## ⚡ SHOULD ADD — dev guardrails (prevent reinvention & subtle bugs)

- **E1 — Reuse `shared::log_path()` (AD-12).** View Logs must call `shared::log_path()` (`crates/shared/src/config.rs:162`, re-exported `lib.rs:14`) instead of hand-building `%APPDATA%\WinTick\wintick.log`. `daemon` already depends on `shared`. AD-12 makes `shared` the owner of the `%APPDATA%` path.
- **E2 — `show_message_box` is private in `main.rs`; reuse `util::wide`.** About/Check-for-Updates (Task 5) **cannot** call `show_message_box` (`main.rs:48`, not `pub`, and it hardcodes `MB_ICONERROR`). Lift a shared MessageBox helper into `util.rs` (or write one in `menu.rs`) using **`util::wide()`** (`util.rs:4`) for all wide strings.
- **E3 — Menu dismissal fix.** After `SetForegroundWindow` + `TrackPopupMenu`, add **`PostMessage(hwnd, WM_NULL, 0, 0)`** (MSDN KB135788) so the menu dismisses on the first outside click on a hidden/tool window. Story has the `SetForegroundWindow` half (line 29) but omits this.
- **E4 — Commit to one schtasks mechanism.** Task 4 (line 37) says "`CreateProcessW`/`std::process::Command`" — pick one. Recommended: **`std::process::Command` + `std::os::windows::process::CommandExt::creation_flags(CREATE_NO_WINDOW)`** — pure `std`, no FFI, no new feature.
- **E5 — Cargo.toml task is a no-op; correct it.** Story line 58 ("tambah `Win32_System_Threading` … jika perlu") is misleading — it's already enabled (`daemon/Cargo.toml:18`). **All** needed features (`Win32_UI_WindowsAndMessaging`, `Win32_UI_Shell`, `Win32_System_Threading`, `Win32_System_LibraryLoader`, `Win32_Foundation`) are already present. State "no windows-sys change required."
- **E6 — `windows-sys` version drift.** Spine Stack table says `0.61.x`; the code baseline pins **`0.52`** (`daemon/Cargo.toml:13`). Story is silent. Instruct the dev to **stay on 0.52** for this story (do not bump mid-story); flag the spine/code drift for a later reconciliation.
- **E7 — "View Logs" open verb.** `ShellExecuteW(NULL,"open", …)` on `.log` uses the registered handler — may not be a text editor, or none registered. AC-4 says "editor teks bawaan OS" and the verification step assumes Notepad. Use `notepad.exe "<path>"` (or handle the no-handler case) and **create the empty file if absent** before opening.
- **E8 — Settings item: pin AD-11 contract.** State that "Settings..." launches `wintick-settings.exe` via **`ShellExecute` inheriting Admin elevation** (AD-11), decoupled process. Path via `GetModuleFileNameW`-relative resolution is fine (unconstrained by arch).
- **E9 — Checkmark source of truth.** AC-7 uses `schtasks /Query` as truth, but `shared::GeneralConfig.auto_start: bool` (`config.rs:28`) already persists a flag. State that `schtasks /Query` is **authoritative** for the checkmark (config field not consulted here) to avoid a dual-source-of-truth bug.

---

## ✨ OPTIMIZATIONS (Nice to Have)

- **O1 — Pin the task name.** Story hardcodes `/TN WinTick` inline in `autostart.rs`. AD-13 doesn't fix a name; put it in `shared/src/constants.rs` for reuse across `is_registered/enable/disable`.
- **O2 — Keep all menu handling inside `wndproc_impl`** so it inherits the existing `catch_unwind` guard (`tray.rs:177-193`); don't add a second unguarded `extern "system"` callback (release is `panic="abort"`).
- **O3 — Follow 1.3's return-value + `debug_log` discipline** for `TrackPopupMenu`/`ShellExecuteW`/`schtasks` (check the return, `debug_log` on failure via `tray.rs:49-54`). No console output (`#![windows_subsystem = "windows"]`, UX-DR1).
- **O4 — Do NOT touch the 1.5-deferred regions** in `tray.rs` (HICON=0 fallback, state-machine transition guards, `NIM_MODIFY`-in-`set_state`). Just add the menu arm; leave the state machine alone.
- **O5 — FR-9 pure-Win32 constraint.** State that the menu must be pure Win32 (`TrackPopupMenu`, no GUI framework) to protect NFR1 RAM/NFR3 binary. Native menu satisfies this.
- **O6 — Accessibility note.** Tray-menu a11y is scoped to Settings only in the specs (FR-20/21 do not target the tray menu). A native `TrackPopupMenu` gives keyboard nav + UI Automation for free — add `&` mnemonics to item labels and state this explicitly so the omission isn't read as a miss.
- **O7 — Ignore the stale comment.** `constants.rs:19` calls `DAEMON_WINDOW_CLASS` "message-only", but the window is actually top-level (`CreateWindowExW`, required for `TaskbarCreated` + `SetForegroundWindow`). Trust the code.

---

## ✅ What the story already gets right
- `WM_TRAYICON = WM_APP + 10` matches the live constant (`tray.rs:44`) and collides with no defined `WM_APP_*` id (`shared/src/constants.rs`).
- Auto-Start flags (`ONLOGON`, `/RL HIGHEST`, `/RU %USERNAME%`, absolute `/TR`) match **AD-13 / SPEC** exactly; correctly placed in the tray menu (per PRD FR-13), not SYSTEM.
- Menu item set (6 items) is complete and matches FR-16 / epics / UX.
- `SetForegroundWindow` before `TrackPopupMenu` and `TPM_RIGHTBUTTON | TPM_RETURNCMD` are correct.
- Scope correctly excludes the 1.5 state-machine work.

---

## Must-preserve (regression guardrails) — do not break
- Message loop `-1` vs `0` split (`tray.rs:324-338`) — manual `cleanup()` on `-1`.
- Notify-icon lifecycle `NIM_ADD`+`NIM_SETVERSION` / `NIM_MODIFY` / `NIM_DELETE` via `notify_data`.
- Explorer-restart recovery (AD-10): `taskbar_created` guard + re-`add_icon` + `ChangeWindowMessageFilterEx` (`tray.rs:213-218, 300-311`).
- `catch_unwind` FFI guard; `GWLP_USERDATA` state pointer; single-thread ownership (AD-1).
- `WS_EX_TOOLWINDOW`, never `WS_VISIBLE` (UX-DR1 invisibility).
- `DAEMON_WINDOW_TITLE` unchanged (Settings IPC `FindWindowW` target).
