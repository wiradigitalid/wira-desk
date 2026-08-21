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
---

# LC-arrangement-engine — Arrangement Engine

## Responsibility

`LC-arrangement-engine` plans DPI-aware window geometry for keyboard-driven snapping and overlapping stacks. It is invoked synchronously from `LC-worker-thread` after a snap or stack command is dequeued. Responsibilities:

1. Resolving the active monitor work area in physical pixels, accounting for per-monitor DPI (FR-14).
2. Computing half-screen placements (left, right, maximize) without activating the window during geometry application (`SWP_NOACTIVATE`).
3. Planning overlapping stack layouts for up to three half-width windows with visible leading edges on small monitors (FR-15).
4. Returning `PlacementPlan` structs consumed by `Win32WindowMover` (`SetWindowPos`).

The engine never installs hooks, never writes configuration, and never calls blocking enumeration beyond what the worker already collected.

## Depends on

- `crates/daemon/src/arrangement/mod.rs` — public planning API.
- `crates/daemon/src/arrangement/snap.rs` — half-screen and maximize math.
- `crates/daemon/src/arrangement/stack.rs` — three-window cascade geometry.
- `crates/daemon/src/arrangement/win32.rs` — `SetWindowPos` application.
- `crates/daemon/src/context/spatial.rs` — monitor work area and DPI.
- `shared::Command` — `SnapLeft`, `SnapRight`, `SnapMaximize`, `OverlappingStack` opcodes.

## Interface

### Inbound

| Method | Caller | Input |
| --- | --- | --- |
| `plan_snap_left(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_right(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_snap_maximize(hwnd)` | `LC-worker-thread` | Foreground window |
| `plan_stack(windows[])` | `LC-worker-thread` | Up to 3 eligible HWNDs |

### Outbound

| Result | Consumer |
| --- | --- |
| `PlacementPlan { x, y, cx, cy }` | `Win32WindowMover::apply` |

## Notes

- **DPI:** Plans use monitor DPI at plan time; moving a window across monitors after snap is out of scope for this command.
- **Evidence:** [PARTIAL] `crates/daemon/src/arrangement/`, `crates/daemon/src/context/spatial.rs`.
