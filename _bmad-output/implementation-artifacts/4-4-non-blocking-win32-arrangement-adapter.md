---
baseline_commit: 2474934
workflow_id: story-4-4-win32-arrangement-adapter
---

# Story 4.4: Non-Blocking Win32 Arrangement Adapter

Status: review

## Story

As a WinTick user,
I want layout plans applied to the correct monitor without freezing or stealing focus,
so that arrangement remains reliable even when another application is slow or closes unexpectedly.

## Acceptance Criteria

### AC-4.4-001 — Fresh platform context
Resolves foreground window, nearest monitor, current work area, and window DPI per command; nothing cached between commands.

### AC-4.4-002 — Coordinate fidelity
Non-default DPI and negative coordinates survive conversion without logical-pixel double scaling.

### AC-4.4-003 — Non-activating placement
Each target revalidated immediately before placement; `SetWindowPos` uses non-activating, Z-order-preserving, asynchronous flags; no overlay or transition surface.

### AC-4.4-004 — Partial failure
A target that closes after planning is skipped; remaining placements continue; no popup, crash, or blocking cross-process call.

### AC-4.4-005 — Inspection
`SendMessage`, `GetWindowText`, internal geometry caching, and Epic 3 virtual-desktop integration are absent.

## Dev Notes

### `rcWork`, Not `rcMonitor`

`GetMonitorInfoW` returns both. The adapter uses `rcWork`, where the taskbar and
any reserved appbar are already excluded — which is exactly what the arrangement
contract's `WorkArea` means. Using `rcMonitor` would place windows under the
taskbar.

### `MONITOR_DEFAULTTONULL` Again

Same reasoning as the Epic 3 spatial adapter: a window on no monitor must fail
rather than be arranged onto an arbitrary one.

### RECT Converts by Field Copy

Win32 `RECT` is already half-open on right and bottom, so `rect_from_win32()` is
a field copy plus validation — **no** coordinate adjustment. That absence is
what guarantees physical pixels survive intact; any "+1" here would be a silent
double-scaling bug at non-default DPI.

### Flag Set Is Pinned

`SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_ASYNCWINDOWPOS`.

`SWP_ASYNCWINDOWPOS` is the load-bearing one: it posts the request instead of
delivering it synchronously, so a hung target cannot block the Worker. Dropping
`SWP_NOACTIVATE` would steal focus. A test asserts each flag is present, because
losing one would be invisible in review but obvious to a user.

### No Cache, Structurally

There is no `static`, no `OnceLock`, and no memoized monitor or DPI anywhere in
the file. `resolve_context()` re-queries everything per command.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 13 unit tests. The `FakeMover` tests verify call arguments, ordering, and
  partial failure without touching User32 (AC-4.4-005). The real-adapter tests
  use only invalid handles, which resolve identically in any session.
- Static gates PASS in debug and release.
- `Win32_UI_HiDpi` added to the daemon's `windows-sys` features for
  `GetDpiForWindow`.
- **`SetWindowPos` has never been called against a real window.** Flag
  behaviour, work-area selection, and DPI fidelity are argued from
  documentation, not observed. The elevated helper-window runtime test named in
  AC-4.4-005 is **not** implemented.

### File List
- `crates/daemon/src/arrangement/win32.rs` (new)
- `crates/daemon/src/arrangement/mod.rs` (modified — submodule decl)
- `crates/daemon/Cargo.toml` (modified — `Win32_UI_HiDpi`)

### Review Findings

- [ ] [Review][Decision] Maximized foreground windows are never restored before `SetWindowPos`, and the standard fix reintroduces the exact blocking hazard this story eliminates — `SetWindowPos` never clears `WS_MAXIMIZE`. A maximized-and-focused window — the single most common real-world snap-shortcut scenario — will not resize correctly unless first restored via `ShowWindow(SW_RESTORE)` or `SetWindowPlacement`, but both of those are synchronous, cross-thread messaging calls that can block on a hung target, reintroducing the exact hazard `SWP_ASYNCWINDOWPOS`/AC-4.4-003 was built to remove. No code anywhere in the daemon currently handles `IsZoomed`/`WS_MAXIMIZE`/`SW_RESTORE`/`SetWindowPlacement` (repo-wide grep confirms zero references). This needs a human/architect call on whether to accept a bounded blocking risk for the restore step, find an async-safe partial workaround, or explicitly scope "already-maximized window" support out of Story 4.4 with a documented decision. [`crates/daemon/src/arrangement/win32.rs:96-123`]
- [ ] [Review][Decision] Foreground window is never filtered for non-arrangeable shell-surface classes before being treated as a target — `resolve_context`/`resolve_context_for` accept whatever `GetForegroundWindow()` returns unconditionally. Windows can legitimately report `Progman`/`WorkerW`/`Shell_TrayWnd` (desktop or taskbar) as the foreground window (e.g. right after Win+D or a click on empty desktop); firing a snap/stack hotkey in that state would call `SetWindowPos` on the desktop or taskbar itself. `crates/daemon/src/cycling/mod.rs` and `eligibility.rs` already define `CLASS_PROGMAN`/`CLASS_WORKERW`/`CLASS_SHELL_TRAY` and an eligibility policy for exactly this concern in the Epic 2 discovery adapter, but `arrangement/win32.rs` has no equivalent guard. Whether this filter belongs inside the Story 4.4 adapter itself or in the Story 4.5 composition layer (which owns `worker.rs`/`hook.rs` per this file's own module-ownership contract) is an architectural ownership question that should be decided by a human before it's patched, so it isn't duplicated or contradicted by Story 4.5's design. [`crates/daemon/src/arrangement/win32.rs:39-56`]
- [x] [Review][Patch] `resolve_context`/`resolve_context_for` do not exclude DWM-cloaked windows before treating `GetForegroundWindow()`'s result as an arrangement target — a cloaked window (suspended UWP surface, or a leftover from a virtual-desktop switch) can transiently be reported as foreground while invisible to the user; the adapter would silently build a valid context for it and "eat" the hotkey with no visible effect. The sibling `crates/daemon/src/cycling/source.rs::is_cloaked` already implements this exact check via `DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, ...)`, and this diff's own `Cargo.toml` already adds the `Win32_Graphics_Dwm` feature needed for it (otherwise unused by this file). Recommended fix: add a small, self-contained cloak check inside `resolve_context_for` (duplicate the ~8-line check locally rather than reaching into the `cycling` module, consistent with this file's story-isolation design). [`crates/daemon/src/arrangement/win32.rs:48-56`]
- [x] [Review][Patch] Every failure path in this file is completely silent — no `GetLastError()` capture, no `debug_log`/`append_debug_trace` call anywhere in `resolve_context_for` or `Win32WindowMover::apply`, unlike the 9 other daemon modules (including the closest sibling, `cycling/activation.rs`, which traces `FOCUS_ATTEMPT`/`FOCUS_RESULT` for the same "flaky Win32 call" shape) that use this codebase's established tracing convention. A concrete, plausible manifestation: `SetWindowPos` fails via UIPI whenever the target belongs to a higher-integrity (elevated) process and the daemon is unelevated (e.g. Task Manager, an installer) — arrangement would silently and permanently fail with zero diagnostic trail for a user reporting "snap doesn't work." Recommended fix: add `debug_log`/`append_debug_trace` calls at the `None`/`false`-return branches, following the existing convention. [`crates/daemon/src/arrangement/win32.rs:39-122`]
- [x] [Review][Patch] `placement_flags_are_non_activating_and_z_order_preserving` is tautological and never verifies the flags actually passed to `SetWindowPos` inside `Win32WindowMover::apply` — it recomputes the same literal flag expression locally and asserts it against itself, so it would keep passing even if a flag were dropped from the real code path. The story's own Dev Notes call out that this is specifically the property "a test asserts... because losing one would be invisible in review but obvious to a user" — as written, it doesn't provide that protection. Recommended fix: hoist the flag expression into a shared `const`/private fn used by both `apply()` and the test (or otherwise capture the real flags via a test seam). [`crates/daemon/src/arrangement/win32.rs:314-323`, flags used at `:119`]
- [x] [Review][Patch] `real_mover_rejects_degenerate_geometry` does not exercise the degenerate-geometry rejection branch it's named for — it builds a `Placement` with `window: WindowId(0)`, which is already rejected by the earlier `hwnd == 0 || IsWindow(hwnd) == FALSE` guard before the function ever reaches `checked_width()`/`checked_height()`. It's an accidental duplicate of `real_mover_rejects_invalid_targets`; the actual `w <= 0`/`h <= 0` rejection branch inside `Win32WindowMover::apply` has zero coverage from the real mover (this is a direct symptom of the already-disclosed "no live-window harness" gap under AC-4.4-005, since exercising it for real requires a live HWND). Recommended fix: remove the duplicate or rename/retarget it to be accurate about what it covers, and track the real gap under the existing AC-4.4-005 disclosure rather than leaving a misleadingly-named passing test. [`crates/daemon/src/arrangement/win32.rs:299-312`]
- [x] [Review][Defer] `Rect`'s fields are all public and `Placement { .. }` can be constructed by struct literal, bypassing `Rect::new()`'s validation entirely (as the test file itself does) — pre-existing from the frozen Story 4.1 contract in `arrangement/mod.rs`, not touched by this diff. [`crates/daemon/src/arrangement/mod.rs:29-34,126-129`] — deferred, pre-existing
- [x] [Review][Defer] TOCTOU gaps around raw `HWND` identity: a window can be destroyed (or its handle recycled by the OS for an unrelated window) between the `IsWindow` check and the subsequent `MonitorFromWindow`/`GetDpiForWindow`/`SetWindowPos` calls. This is inherent to trusting a raw `HWND` without a generation/identity token — a pattern used throughout the whole codebase (e.g. `cycling::WindowId` has the same property) — not something introduced or fixable within this diff's scope. [`crates/daemon/src/arrangement/win32.rs:48-75,96-121`] — deferred, pre-existing
- [x] [Review][Defer] Minimized foreground window edge case: Windows parks a minimized window at `(-32000, -32000)`, so `MonitorFromWindow(..., MONITOR_DEFAULTTONULL)` returns `NULL` and `resolve_context_for` degrades to a silent `None`/no-op — consistent with the adapter's own stated fail-safe design (no partial arrangement), and low real-world reachability since minimizing a window ordinarily removes its foreground/focus status in the first place. [`crates/daemon/src/arrangement/win32.rs:39-58`] — deferred, low reachability
- [x] [Review][Defer] `apply_plan` does not defend against a plan containing the same `WindowId` twice (double placement applied to one window, double-counted in `applied`); no current planner is expected to produce one (each comes from a distinct Z-order/candidate enumeration), so this is optional future hardening rather than an active bug. [`crates/daemon/src/arrangement/win32.rs:129-143`] — deferred, low reachability

**Dismissed as noise (not written above):** (1) claim that `resolve_context`/`Win32WindowMover` are unreferenced/dead-code — factually incorrect, `worker.rs` already imports and calls `resolve_context()`/`apply_plan`/`Win32WindowMover` for `SnapLeft`/`SnapRight`/`SnapMaximize`/`OverlappingStack`; (2) `WorkArea`/`PlatformContext` not preserving the `HMONITOR`/a persistent `MonitorId` — explicitly by-design per this story's own AC-4.4-005 "no Epic 3 virtual-desktop integration" disclaimer; (3) `GetDpiForWindow`'s 0→96 DPI fallback being indistinguishable from a genuine 96 DPI monitor — cosmetic only, since DPI is documented as carried-but-never-applied (AC-4.4-002); (4) `SWP_ASYNCWINDOWPOS` "only posts, never confirms" flagged as a gap — this is the story's explicit, heavily-documented intentional design choice, not a defect; (5) the three "real Win32" tests calling live User32 functions directly being fragile on a headless/Server-Core CI runner — overstated, since they only pass sentinel-invalid handles (`0`, `-1`) whose `IsWindow` behavior is guaranteed by Win32 contract regardless of desktop-session state; (6) `apply_plan` not cross-checking placements against `PlacementPlan::fits_within`/`STACK_MAX_WINDOWS` — that enforcement belongs to the planner layer (Stories 4.2/4.3), redundant at this adapter layer; (7) no direct test of the zero-argument `resolve_context()` entry point — subsumed by the already-disclosed AC-4.4-005 live-window-harness gap, not an independent issue.
