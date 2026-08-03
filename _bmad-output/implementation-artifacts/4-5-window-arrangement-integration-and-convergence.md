---
baseline_commit: 2474934
workflow_id: story-4-5-arrangement-convergence
---

# Story 4.5: Window-Arrangement Integration and Convergence

Status: in-progress

## Story

As a WinTick user,
I want arrangement shortcuts to produce correct snapping and stacking through the existing background service,
so that the complete feature works instantly without weakening cycling or hook reliability.

## Acceptance Criteria

### AC-4.5-001 — Upstream readiness — **partially met**
Stories 4.2, 4.3, 4.4 are code-complete against 4.1 but none has an *accepted review*; no lane has been executed.

### AC-4.5-002 — Snap pipeline — **code complete, unverified**
The primitive command travels the existing SPSC ring; the Worker resolves fresh platform context, invokes the snap planner, and applies the plan; callback, allocation, throttle, and swallowing contracts intact.

### AC-4.5-003 — Stack pipeline — **code complete, unverified**
Live same-application candidates via the Epic 2 contract, restricted to the active target monitor without requiring Epic 3; at most three placements.

### AC-4.5-004 — Disabled stack — **code complete**
No window moved; daemon stays responsive with no overlay, animation, popup, or error-state escalation.

### AC-4.5-005 — Fresh geometry and Tier-2 failure — **code complete**

### AC-4.5-006 — Convergence suite — **NOT DONE**

## Dev Notes

### Dispatch Is All That Changed in the Hot Path

`drain_commands()` gained three arms. The Hook side is untouched: arrangement
commands already had frozen `u8` values and already travelled the existing ring,
so nothing about callback timing, allocation, throttle, or swallowing moved.

### Stack Filters by Monitor Without Epic 3

AC-4.5-003 requires "the active target monitor **without requiring Epic 3**". So
`execute_stack()` compares `HMONITOR` directly via `Win32Monitors` rather than
going through the Epic 3 `SpatialContext`/`evaluate_spatial` path. Same
underlying call, deliberately not the same contract — stacking must work whether
or not Epic 3 has converged.

That is a real duplication, and it is intentional. Collapsing it into the Epic 3
contract would make Epic 4 depend on Epic 3, which the AC forbids.

### Failure Is Quiet

`report_arrangement_failure()` emits a debug-log line and a trace entry. It does
**not** show a popup and does not touch tray state, so an arrangement failure
cannot downgrade an existing Critical indication (AC-4.5-005).

A no-op plan — disabled stack, zero candidates — is logged as `noop=1` and moves
nothing, which is the success path, not an error.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- Static gates PASS in debug and release; test binary compiles.
- **Status `in-progress`.** AC-4.5-006 (convergence suite) is not done, and
  AC-4.5-001 is not satisfiable yet: no upstream lane has an accepted review
  because no test has been executed anywhere in Epic 4.
- The end-to-end path — press shortcut, ring, Worker, `SetWindowPos` — has never
  been exercised.
- Config is re-read per stack command via `Config::load_or_default`. That is
  correct for "fresh geometry" but is file I/O on the Worker thread; if it shows
  up in profiling, cache it behind the existing `WM_APP_RELOAD_CONFIG` signal
  rather than removing the freshness guarantee.

### File List
- `crates/daemon/src/worker.rs` (modified — snap/stack dispatch, Tier-2 failure path)
