---
type: model
component: window-management
layer: conceptual
created: 2026-08-21
updated: 2026-08-21
---

# Model — window-management

Conceptual domain model for the `window-management` component. Represents domain entities, relationships, state transitions, and invariants from the business and user perspective. Physical types, struct definitions, and Win32 FFI storage details belong in `.how/window-management/05-model/data-model.md`.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| `hook-command` | An intercepted, validated, and throttled user intent command dispatched from the keyboard hook to the background worker. | Command action code and dispatch timestamp |
| `window-focus-state` | The live snapshot of active window focus, monitor boundaries, virtual desktop context, and application identity on the current desktop. | Foreground window handle (`HWND`), monitor identity, and virtual desktop GUID |
| `tray-health-state` | The operational status of the background daemon, tracking hook attachment vitality, error severity level, and user notification state. | Error tier level (`Normal`, `Warning`, `Critical`) and hook vitality status |
| `arrangement-command` | A planned window repositioning and sizing action targeting specific desktop regions (a half-screen snap to any of the four halves, maximized state, an overlapping stack slot, or a move to the next monitor). | Target screen region geometry, the destination monitor, and monitor DPI scaling context |

## Relationships

- One `hook-command` **triggers** the evaluation of one live `window-focus-state`.
- One `window-focus-state` **determines** the candidate set of same-application windows on the active monitor.
- One `tray-health-state` **reflects** the runtime health of the hook thread and error reporting protocol.
- One `arrangement-command` **modifies** the geometry of the active window within the bounds of `window-focus-state`.
- One `arrangement-command` of the monitor-move kind **reads** the live monitor set to choose a destination, and is the only kind whose target work area is not the one `window-focus-state` reports.

## State Lifecycle

### tray-health-state

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| `Normal` | `Warning` | Non-fatal runtime operational failure (e.g. transient enumeration error) | Worker thread (Tier 2) |
| `Warning` | `Normal` | Successful recovery and clean operation on subsequent cycle | Worker thread |
| `Normal` / `Warning` | `Critical` | Hook death detected by 10-second heartbeat or OS unhook event | Tray controller (Tier 3) |
| `Critical` | `Normal` | Successful hook re-installation or daemon recovery | Tray controller / Worker thread |

### hook-command Lifecycle

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| `Pending` | `Dispatched` | Shortcut chord detected and passed throttle window (≥50 ms) | Hook thread |
| `Pending` | `Dropped` | Throttle window violation (<50 ms) or ring buffer full | Hook thread |
| `Dispatched` | `Executed` | Worker completes window focus transition or arrangement | Worker thread |

## Invariants

- **Live Traversal Invariant:** Window focus state and Z-order stacking must never be cached between shortcut keypresses; each cycle command must traverse live desktop state (AD-3).
- **Spatial Preservation Invariant:** Target candidate windows for cycling or snapping must reside on the exact same physical monitor and virtual desktop as the foreground window (FR-2, CAP-7). A monitor-move command is the one deliberate crossing of the monitor half of this boundary; it still must not cross the virtual desktop half, and moving a window never changes which desktop shows it (FR-23, AD-9).
- **Proportional Placement Invariant:** A window moved between monitors must be placed by the share of the destination work area it occupied on the source, never by copying its pixel width and height — otherwise an arrangement dissolves the moment the two monitors differ in size or display scaling (FR-23, DEC-007).
- **Live Monitor Set Invariant:** The set of attached monitors must be enumerated fresh on every monitor-move command and must never be cached between keypresses. An `HMONITOR` is a handle rather than an identity, and a cached list survives an unplug that the handle does not (AD-14).
- **One Chord, One Action Invariant:** No two actions may be reachable by the same chord. When configuration says otherwise, the chord belongs to the first action in the fixed precedence order and the later action is unbound rather than ambiguous (BR-6, DEC-009).
- **UX Honesty Invariant:** Unresponsive ("Not Responding") windows must receive focus when reached in the cycling sequence and must never be filtered out (FR-4).
- **Hook Callback Speed Invariant:** The low-level keyboard hook callback must complete within 10 ms without executing heap allocations or blocking synchronous APIs (NFR-2, NFR-3).
- **Single Instance Invariant:** Exactly one background daemon instance may run per user logon session (NFR-6).
