# Work Handover Prompt — Story 1.4 (Tray Context Menu, View Logs & Auto-Start)

> **How to use this file:** paste the whole document as the opening prompt for a fresh session (ideally a **different LLM** than the one that implemented the story). It is self-contained — it tells you the state, what is proven, what is not, and exactly what to do next.

---

## Mission

Story 1.4 is **code-complete and at status `review`**. Two things remain before it can move to `done`, and **both require an elevated, interactive Windows session** (the implementing session was non-interactive and non-elevated, so it could not do them):

1. Run the daemon's own unit tests (`cargo test -p daemon`) — they only *execute* under elevation.
2. Manually verify the runtime GUI behaviors and then run an adversarial code review.

Your job: perform that verification, run `code-review`, apply any findings, and close the story out.

---

## Current state (facts)

| Item | Value |
| --- | --- |
| Story file | `_bmad-output/implementation-artifacts/1-4-tray-context-menu-view-logs-and-auto-start.md` |
| Story status | `review` (sprint-status.yaml: `1-4-... : review`) |
| Baseline commit | `c5d65c2` (recorded in the story frontmatter) |
| Validation report | `_bmad-output/implementation-artifacts/story-1.4-validation-report-2026-07-12.md` |
| Toolchain | Rust 1.96.1, MSVC; build via `./build.ps1 -Mode {dev|prod}` (loads vcvars64 automatically) |
| Constraint | The daemon crate embeds a `requireAdministrator` manifest (Story 1.2). Every binary it builds — **including the test harness** — needs elevation to run (`os error 740` otherwise). This is pre-existing, not a bug. |

---

## What was implemented

**New files**
- `crates/daemon/src/menu.rs` — pure-Win32 tray context menu: `CreatePopupMenu` + `AppendMenuW` (6 items + 2 separators in AC-3 order, `&` mnemonics) → `SetForegroundWindow` → `TrackPopupMenu(TPM_RIGHTBUTTON | TPM_RETURNCMD)` → `PostMessageW(WM_NULL)` (KB135788) → `DestroyMenu`, then dispatch the returned command id to per-item handlers.
- `crates/daemon/src/autostart.rs` — Task Scheduler integration via `std::process::Command` + `creation_flags(CREATE_NO_WINDOW)` (pure `std`, no FFI, no new windows-sys feature). Includes 3 pure-logic unit tests for the `schtasks` argument construction.

**Modified files**
- `crates/daemon/src/main.rs` — added `mod menu; mod autostart;`; `show_message_box` now delegates to `util::message_box`.
- `crates/daemon/src/tray.rs` — added the `WM_TRAYICON` arm **inside `wndproc_impl`** (inherits the `catch_unwind` FFI guard); imports `debug_log` from `util` (local copy removed).
- `crates/daemon/src/util.rs` — added `debug_log` (moved here from `tray.rs`) and a reusable `message_box`.
- `crates/shared/src/constants.rs` — added `TASK_NAME = "WinTick"`.
- `crates/daemon/Cargo.toml` — **unchanged** (all windows-sys features were already present; stayed on 0.52).

**Key design decisions (the subtle ones — verify these hold up)**
- **Exit teardown:** Exit calls `DestroyWindow(hwnd)` → `WM_DESTROY` → `cleanup()` (`NIM_DELETE` + `DestroyIcon`×3 + `UnhookWindowsHookEx`) → `PostQuitMessage`. It does **not** call `PostQuitMessage` directly — `TrayData` has no `Drop`, so the direct path would leak the GDI icons + the keyboard hook.
- **v4 callback parsing:** the icon runs `NOTIFYICON_VERSION_4`. The arm reads event = `LOWORD(lParam)` and anchor X/Y from `wParam` (LOWORD/HIWORD), sign-extended through `i16` for negative coords on a secondary monitor. It triggers on `WM_CONTEXTMENU` (covers right-click + keyboard menu key) and `WM_RBUTTONUP`. No `GetCursorPos`.
- **schtasks `/TR` quoting:** the executable path is wrapped in explicit quotes (`format!("\"{exe}\"")`) so Task Scheduler stores it as a single action. Without this, a `%ProgramFiles%\WinTick\wintick.exe` path (has a space) would be split into exe + args. No working directory is set (AD-13 DLL-hijack mitigation).
- **Auto-Start checkmark:** driven by `schtasks /Query` (`autostart::is_registered()`), **not** `config.auto_start` — single authoritative source (AC-8).
- **Auto-Start flags:** `/SC ONLOGON /RL HIGHEST /RU <%USERNAME%> /F` — runs as the active user (not SYSTEM) so `%APPDATA%` stays aligned between daemon and settings, elevated without a UAC prompt at logon (AD-13).

---

## Verification status

**Done (static, in the implementing session):**
- `cargo check --workspace` — clean.
- `cargo clippy --workspace --tests` — clean.
- `cargo test -p shared` — 10/10 pass.
- `cargo test -p daemon --no-run` — test harness (incl. `autostart` tests) compiles.
- `./build.ps1 -Mode prod` — OK; `wintick.exe` **209 KB** (< 500 KB NFR3).

**NOT done (your job — needs elevation / interactivity):**
- Executing `cargo test -p daemon`.
- All runtime GUI behaviors (marked `[~]` in the story's Task 6).

---

## Your tasks (in order)

### 1. Elevated daemon unit tests
Open an **elevated** terminal (Run as Administrator) at the repo root and run:
```powershell
cargo test -p daemon
```
Expect the 3 `autostart` tests to pass (`create_args_wraps_exe_path_in_quotes`, `create_args_carries_ad13_hardening_flags`, `query_and_delete_target_the_pinned_task_name`). If `cargo test` still reports `os error 740`, your shell is not actually elevated.

### 2. Runtime GUI verification (elevated)
Build and launch the daemon as Administrator:
```powershell
./build.ps1 -Mode prod
Start-Process ".\target\release\wintick.exe" -Verb RunAs
```
Then confirm each Task-6 behavior:
- [ ] Right-click the tray icon (and press the keyboard menu key) → the menu shows **exactly**: `Settings...`, `View Logs`, `Auto-Start` — separator — `Check for Updates...`, `About` — separator — `Exit`.
- [ ] The menu dismisses on the first click outside it.
- [ ] **View Logs** creates `%APPDATA%\WinTick\wintick.log` if absent and opens it in Notepad.
- [ ] **Settings...** launches `wintick-settings.exe` elevated (build it too if missing; it's the `settings` crate).
- [ ] **Auto-Start** toggles the scheduled task — verify with `schtasks /Query /TN WinTick`; the checkmark must match the task's presence when the menu is reopened. Inspect the stored action (`schtasks /Query /TN WinTick /XML`) and confirm the `<Command>` is the quoted absolute exe path with no `<WorkingDirectory>`.
- [ ] **Exit** removes the tray icon with no ghost left behind, and the process terminates cleanly (no leaked hook). Reopen Task Manager / check the tray after exit.

### 3. Code review
Run `code-review` (the `bmad-code-review` skill) — **use a different LLM than the implementer**. Pay special attention to the watch-outs below. Triage findings into the story's `Review Follow-ups` and address them via `bmad-dev-story` (review-continuation mode).

### 4. Close out
When tests pass, all Task-6 boxes are genuinely checked, and review findings are resolved:
- Flip the story's Task-6 `[~]` items to `[x]` and set Status → `done`.
- Set `1-4-tray-context-menu-view-logs-and-auto-start: done` in `sprint-status.yaml`.
- Update `/3p.md` (codebase) and `docs/3p.md` (docs) per the constitution's 3P routing.

---

## Watch-outs (highest-value review targets)

1. **Reentrancy during `TrackPopupMenu`.** It runs a modal message loop, so messages (`WM_APP_LOG_WARNING`, `WM_APP_HOOK_DEAD`, `TaskbarCreated`) can be dispatched to the window while the outer `wndproc_impl` frame is still on the stack holding `&mut *data_ptr`. The `WM_TRAYICON` arm deliberately does **not** touch `data` after that point, so there is no torn access — but confirm no future edit reintroduces a `data` use after `menu::show`.
2. **Exit path is nested.** Selecting Exit calls `DestroyWindow` from inside `menu::show`, which synchronously dispatches `WM_DESTROY` (runs `cleanup()`) before returning. `cleanup()` is idempotent and does not free the heap `Box` (the message loop does, after `WM_QUIT`). Verify no use-after-free.
3. **`ShellExecuteW` elevation inheritance.** AC-4/AD-11 assumes the launched `wintick-settings.exe` inherits Admin elevation from the elevated daemon. Confirm this actually happens on the target machine (check the child process's integrity level), since `ShellExecute` can route through the shell.
4. **`schtasks` output suppression.** `run_schtasks` nulls stdout/stderr and uses `CREATE_NO_WINDOW`; confirm no console window flashes when toggling Auto-Start.

---

## Referenced documents
- Story: [1-4-tray-context-menu-view-logs-and-auto-start.md](1-4-tray-context-menu-view-logs-and-auto-start.md)
- Validation report: [story-1.4-validation-report-2026-07-12.md](story-1.4-validation-report-2026-07-12.md)
- Architecture: `_bmad-output/planning-artifacts/` (ARCHITECTURE-SPINE AD-11/12/13, CAP-10/11) and the SPEC (`_bmad-output/specs/spec-wintick/`)
- Trackers: [/3p.md](../../3p.md) (codebase), [docs/3p.md](../../docs/3p.md) (docs)
