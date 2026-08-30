---
type: lc
id: LC-arrangement-engine
name: Arrangement Engine
lc_type: service
container: daemon
component: window-management
owner: Wira Desk Core
area: layout-planning
created: 2026-08-21
updated: 2026-08-26
---

# LC-arrangement-engine — Arrangement Engine

## Responsibility

`LC-arrangement-engine` plans DPI-aware window geometry for keyboard-driven snapping, overlapping stacks, and deliberate movement between monitors. It is invoked synchronously from `LC-worker-thread` after an arrangement command is dequeued. Responsibilities:

1. Refusing a target that belongs to Wira Desk itself, before any geometry is resolved (LBR-WM-6, DEC-006).
2. Resolving the active monitor work area in physical pixels, accounting for per-monitor DPI (FR-14).
3. Computing half-screen placements for all four halves — left, right, top, bottom — and maximize, without activating the window during geometry application (`SWP_NOACTIVATE`). Both halves of an axis derive from one boundary computed once, so they tile the work area exactly (LBR-WM-8, FR-14, FR-22).
4. Enumerating the live monitor set and planning a move to the next monitor by mapping the window's share of its source work area onto the destination work area (LBR-WM-7, FR-23, AD-14).
5. Planning overlapping stack layouts for up to three half-width windows with visible leading edges on small monitors (FR-15).
6. Returning `PlacementPlan` structs consumed by `Win32WindowMover` (`SetWindowPos`).

The engine never installs hooks, never writes configuration, and never calls blocking enumeration beyond what the worker already collected.

## Depends on

- `crates/daemon/src/arrangement/mod.rs` — public planning API.
- `crates/daemon/src/arrangement/snap.rs` — half-screen and maximize math, both axes.
- `crates/daemon/src/arrangement/monitor.rs` — next-monitor selection and proportional remapping. `[MISSING]` — not yet written; planned by this pass. It is deliberately a separate module because every function in `snap.rs` takes exactly one `WorkArea`, and this operation needs two plus a monitor list (DEC-007).
- `crates/daemon/src/arrangement/stack.rs` — three-window cascade geometry.
- `crates/daemon/src/arrangement/win32.rs` — `SetWindowPos` application.
- `crates/daemon/src/context/spatial.rs` — monitor work area, DPI, and the live monitor set. `[PARTIAL]` — today it resolves only `MonitorFromWindow`; enumeration via `EnumDisplayMonitors` is `[MISSING]`.
- `shared::Command` — `SnapLeft`, `SnapRight`, `SnapMaximize`, `OverlappingStack` opcodes, plus `SnapTop`, `SnapBottom`, `MoveToNextMonitor` at wire values 6, 7, and 8. `[MISSING]` — the three new variants are planned, not present (AD-2).

## Interface

### Inbound

| Method | Caller | Input |
| --- | --- | --- |
| `plan_snap_left(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_right(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_top(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_bottom(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_maximize(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_move_next_monitor(hwnd)` | `LC-worker-thread` | Foreground window, plus the live monitor set |
| `plan_stack(windows[])` | `LC-worker-thread` | Up to 3 eligible HWNDs |

### Outbound

| Result | Consumer |
| --- | --- |
| `PlacementPlan { x, y, cx, cy }` | `Win32WindowMover::apply` |

## Notes

- **Target eligibility:** Ownership is decided from the target process’s image basename and from the daemon’s own process id. Identity that cannot be read degrades to “not ours”, so a process the daemon cannot open does not silently lose snapping; the complementary guard in the `settings` container (LBR-ST-13) covers the residual case.
- **DPI:** Plans use monitor DPI at plan time. Coordinates arrive already in physical pixels from a Per-Monitor-V2-aware process, so no planner scales by DPI — doing so would scale twice.
- **Cross-monitor DPI, known limitation:** frame-inset compensation is measured on the source monitor before the move, so on a move between monitors of different scaling the visible frame lands a few pixels off. `DEC-007` accepts this rather than specifying a two-pass placement, and states why. That small inset — rather than Windows relocating the window outright — depends on `Win32WindowMover::apply` clamping against the *destination* monitor (the planned rect's monitor), not the monitor the window is still on when `apply` runs (`DEC-010`).
- **Monitor set:** never cached. Enumerated fresh per invocation, because an `HMONITOR` is a handle rather than an identity and a cached list survives an unplug the handle does not (AD-14).
- **Single monitor:** `plan_move_next_monitor` returns an empty plan, which is a successful no-op rather than a failure — the same convention the stack planner already uses when it is disabled.
- **Evidence:** [PARTIAL] `crates/daemon/src/arrangement/`, `crates/daemon/src/context/spatial.rs`.
