# Work Handover Prompt — Story 1.5 (3-Tier Error Protocol & Tray State Machine)

> **How to use this file:** paste the whole document as the opening prompt for a fresh session. It is self-contained — it tells you the state, what is proven, what is not, and exactly what to do next. An **elevated, interactive Windows session** is required for everything in "Your tasks" below.

---

## Mission

Story 1.5 is **code-complete but held at status `in-progress`** (deliberately NOT bumped to `review`). All five implementation tasks (Task 1-5) are done and statically verified. Task 6 (verification) is only partially done: the parts that don't need elevation are green; the parts that do (running the daemon's own tests, and manually exercising the runtime behaviors) could not be performed by the implementing session, which was non-interactive and non-elevated.

Your job: decide how to simulate a dead keyboard hook (open question, see Task 0), run the elevated tests, manually verify the runtime behaviors, run a code review, resolve the watch-outs below, and close the story out.

---

## Current state (facts)

| Item | Value |
| --- | --- |
| Story file | `_bmad-output/implementation-artifacts/1-5-3-tier-error-protocol-and-tray-state-machine.md` |
| Story status | `in-progress` (`sprint-status.yaml`: `1-5-... : in-progress`) |
| Baseline commit | `a795175` (recorded in the story frontmatter) |
| Toolchain | Rust 1.96.1, MSVC; build via `./build.ps1 -Mode {dev|prod}` (loads vcvars64 automatically) |
| Constraint | The daemon crate embeds a `requireAdministrator` manifest (Story 1.2). Every binary it builds — **including the test harness** — needs elevation to run (`os error 740` otherwise). This is pre-existing (also hit in Stories 1.2/1.4), not a bug introduced here. |

---

## What was implemented

**New files**
- `crates/daemon/src/error.rs` — Tier 1 (Fatal): `fatal(msg: &str) -> !`, reuses `util::message_box`. Consolidates 3 previously-duplicated call sites in `main.rs` (elevation check, module-handle failure, hook-retry-exhausted). Note: `ExitProcess` is declared `-> !` in windows-sys 0.52, so `fatal()` needs no trailing `loop {}` — the compiler already knows it diverges.
- `crates/daemon/src/log.rs` — Tier 2 (Warning): `warn(hwnd, msg)` appends a timestamped line to `shared::log_path()` (open-write-close per line, no persistent handle) then `PostMessageW(WM_APP_LOG_WARNING)`. Timestamp via Win32 `GetLocalTime`/`SYSTEMTIME`, not a `chrono`/`time` dependency (NFR3). **`warn()` has no call-site yet in this story** — it's `#[allow(dead_code)]` on purpose (see Watch-out 3).
- `crates/daemon/src/health.rs` — AD-8 heartbeat: a **pure** `sleep(HOOK_HEARTBEAT_SECS)` loop that only does `PostMessageW(hwnd, WM_APP_HOOK_CHECK, 0, 0)`. It never touches `HHOOK`. This is the fix for the threading bug the Story Context Review caught: `WH_KEYBOARD_LL`'s callback only fires when pumped by the message loop of the thread that installed it, and `health.rs`'s thread has no message loop — so hook re-registration can only safely happen on the main/tray thread.

**Modified files**
- `crates/daemon/src/tray.rs` — added to `TrayData`: `hook_check_fail_count: u32`, `hook_dead_toast_sent: bool`. Added `show_toast()` (reuses `notify_data()` as a base, ORs in `NIF_INFO`, sets `szInfo`/`szInfoTitle`/`dwInfoFlags` — note `dwInfoFlags` is a **direct field** on `NOTIFYICONDATAW`, not part of the `Anonymous` union, contrary to what the story's Dev Notes originally sketched). Added `refresh_hook()` + pure-logic `next_hook_check_state()` (3 unit tests) for AD-8. Added the `WM_APP_HOOK_CHECK` arm; extended the existing `WM_APP_HOOK_DEAD` arm to fire the toast (guarded). `run_message_loop` now calls `crate::health::spawn(hwnd)` right after the hidden window is created.
- `crates/daemon/src/main.rs` — `mod error; mod health; mod log;` added; local `show_message_box` removed (3 call sites now call `error::fatal(...)`); `MB_OK`/`MB_ICONERROR` import removed (now unused here); `dummy_hook_proc` changed to `pub(crate)` so `tray::refresh_hook` can reinstall the hook with the same callback.
- `crates/shared/src/constants.rs` — added `WM_APP_HOOK_CHECK = WM_APP + 5` and `HOOK_CHECK_FAIL_THRESHOLD = 3`.
- `crates/daemon/Cargo.toml` — added feature `Win32_System_SystemInformation` (needed for `GetLocalTime` in `log.rs`; this is beyond what the story's Dev Notes originally estimated for Cargo.toml changes).

**Key design decision (the subtle one — this is where a second opinion is most valuable):**
Win32 has **no native API to check whether an `HHOOK` is still alive** without side effects. `refresh_hook()` (called on every `WM_APP_HOOK_CHECK` tick, i.e. every 10s) resolves this by **unconditionally** calling `UnhookWindowsHookEx` on the current handle (a safe no-op if it's already dead) and then immediately `SetWindowsHookExW`-ing a fresh one — a synchronous refresh rather than passive detection. The no-hook window is only the duration of two syscalls, not the full 10-second heartbeat interval. `hook_check_fail_count` only increments when the **reinstall itself** fails (e.g. an AV/GPO actively blocking it at runtime); after `HOOK_CHECK_FAIL_THRESHOLD` (3) consecutive failures, it escalates to `WM_APP_HOOK_DEAD` → Tier 3. A synthetic-probe / passive-flag alternative was considered and rejected as materially more complex (cross-module atomics, `SendInput`, multi-tick state machine) for marginal benefit. See Watch-outs 1 and 2 below — this is a real engineering tradeoff, not an obviously-correct choice, and deserves a review pass.

---

## Verification status

**Done (static, in the implementing session):**
- `cargo check --workspace` — clean, 0 warnings.
- `cargo clippy --workspace` — clean, 0 warnings.
- `cargo build --release -p daemon` — succeeded; `wintick.exe` **219,648 bytes (~214.5 KB)**, below the 500 KB NFR3 budget and below the prior 215 KB baseline.
- `cargo test -p shared` — 10/10 pass (this crate is a pure lib, not subject to the elevation manifest).

**NOT done (your job — needs elevation, or a decision first):**
- `cargo test -p daemon` — blocked by `os error 740`. New tests to expect: 3 in `tray.rs` (`next_hook_check_state_*`) + 1 in `log.rs` (`format_timestamp_pads_single_digit_fields`), plus the pre-existing `autostart.rs` tests.
- All of Task 6's runtime scenarios: `warn()` → red dot; simulated hook death → red cross + exactly 1 toast (repeat → confirm no second toast); forced startup failure → exactly 1 MessageBox then clean exit.

---

## Your tasks (in order)

### 0. Decide how to simulate a dead hook (open question — was mid-discussion when this handover was requested)
Task 6 says "unhook paksa di debug build," which was never actually built. Two options, pick one:
- **Recommended:** add a small `#[cfg(debug_assertions)]`-gated test hook to the code — e.g. a hidden tray menu item or hotkey, visible/compiled only in debug builds, that directly calls `UnhookWindowsHookEx` on the current hook handle. Compiles out entirely in release, no external tooling needed, and matches the story's own "debug build" phrasing.
- **Alternative:** attach WinDbg/Visual Studio to the running elevated process and manually locate/invoke `UnhookWindowsHookEx` on the `hook_handle` field inside the heap-allocated `TrayData` (reachable via `GetWindowLongPtrW(hwnd, GWLP_USERDATA)`). Harder — no exported symbol makes this field directly addressable without correlating the PDB layout by hand — but doesn't require a code change.

Whichever you pick, note in the story's Completion Notes which one was used.

### 1. Elevated daemon unit tests
Open an **elevated** terminal (Run as Administrator) at the repo root and run:
```powershell
cargo test -p daemon
```
All tests should pass, including the 4 new ones listed above. If it still reports `os error 740`, the shell isn't actually elevated.

### 2. Runtime verification (elevated)
```powershell
./build.ps1 -Mode prod
Start-Process ".\target\release\wintick.exe" -Verb RunAs
```
Then confirm each Task-6 runtime scenario:
- [ ] Trigger a Tier 2 warning (there's no wired call-site for `log::warn()` yet in this story — you may need to call it from a temporary test path, or defer this specific check) → tray icon shows the small red dot.
- [ ] Simulate hook death (via whatever you chose in Task 0) → wait ~10s for the next heartbeat → tray icon shows the red cross **and** exactly one toast appears. Trigger death again → confirm **no second toast** fires (guard is working) — but see Watch-out 4 about whether this guard should ever reset.
- [ ] Force a startup failure (e.g. run the exe without elevation) → exactly 1 `MessageBox` then a clean exit.

### 3. Code review
Run `code-review` (the `bmad-code-review` skill), ideally with a different LLM than the implementer. Focus especially on the watch-outs below. Triage findings into the story's Review Follow-ups and address via `bmad-dev-story` (review-continuation mode).

### 4. Close out
Once tests pass and all Task-6 boxes are genuinely checked:
- Flip the remaining Task 6 checkboxes (`[~]`/`[ ]`) to `[x]`, set Status → `review` (or `done` if the review is also finished in the same pass).
- Update `1-5-3-tier-error-protocol-and-tray-state-machine: review` (or `done`) in `sprint-status.yaml`.
- Update `/3p.md` (codebase) and `docs/3p.md` (docs) per the constitution's 3P routing.

---

## Watch-outs (highest-value review targets)

1. **`refresh_hook()`'s unconditional unhook-then-reinstall, every 10 seconds, forever.** This touches a security-relevant global keyboard hook (UIPI bypass) even when nothing is wrong. The exposure window is believed negligible (two back-to-back syscalls), but this hasn't been empirically confirmed under real keyboard load. Worth a second opinion, and worth actually testing "type continuously through several heartbeat ticks, confirm zero dropped keystrokes."
2. **The Tier-3 escalation path (3 consecutive `SetWindowsHookExW` failures) is not realistically testable via forced-unhook alone.** Forcing a single unhook only exercises the *recovery* path (refresh succeeds immediately, nothing escalates). Genuinely testing escalation would need `SetWindowsHookExW` itself to fail 3 heartbeats in a row, which in practice only happens via real AV/GPO blocking — there's no clean way to mock a raw FFI call here. Decide whether this residual test gap is acceptable or worth a dedicated debug-only failure-injection seam.
3. **The Critical tray state (and the toast guard) never reset once reached.** After the first Tier-3 event in a process's lifetime, `TrayState` stays `Critical` forever — even after `refresh_hook()` later succeeds and the hook is genuinely healthy again — because nothing transitions the state back to `Normal`/`Warning` on a successful refresh. Similarly, `hook_dead_toast_sent` is a one-way latch: it never resets, so if the hook fails again much later (a *second*, independent Tier-3 episode), **no toast fires the second time**. AC3 literally says "tepat 1x Toast Notification" (exactly once), which supports the current sticky-forever behavior — but it's genuinely ambiguous whether "once" means "once per process lifetime" or "once per episode," and the sticky-Critical-icon-after-recovery behavior in particular may not match user expectations ("the hook came back, why is the icon still showing an error?"). This was not decided with the user — flag it explicitly in review and get an explicit answer either way before closing the story.
4. **`log::warn()` has no call-site in this story** (`#[allow(dead_code)]`, documented in `log.rs` and the story's Completion Notes as intentional — Tier 2 triggers are expected to come from a future story). Confirm a reviewer wouldn't reasonably read this as "half-finished" and check it's clearly justified.
5. **`crates/daemon/Cargo.toml` gained a feature (`Win32_System_SystemInformation`)** beyond what the story's Dev Notes originally scoped (which only anticipated `NIF_INFO`/`NIIF_ERROR`, already covered by the existing `Win32_UI_Shell` feature). Low risk, but sanity-check the binary-size number above already reflects it (it does — 214.5 KB, confirmed after the change).

---

## Referenced documents
- Story: [1-5-3-tier-error-protocol-and-tray-state-machine.md](1-5-3-tier-error-protocol-and-tray-state-machine.md)
- Trackers: [/3p.md](../../3p.md) (codebase), [docs/3p.md](../../docs/3p.md) (docs)
- Architecture: `_bmad-output/planning-artifacts/architecture/architecture-WinTick-2026-07-06/ARCHITECTURE-SPINE.md` (AD-1, AD-7, AD-8)
