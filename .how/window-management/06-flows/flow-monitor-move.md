# Flow — Move the active window to the next monitor

**Component:** window-management
**Realizes:** UC-7, FR-23, LBR-WM-7, AD-2, AD-9, AD-14, DEC-007

## Sequence

```mermaid
sequenceDiagram
    participant User
    participant Hook as LC-hook-thread
    participant Ring as ring.rs
    participant Worker as LC-worker-thread
    participant Spatial as context/spatial.rs
    participant Plan as LC-arrangement-engine
    participant Win as Win32 (SetWindowPos)

    User->>Hook: Ctrl + Alt + Shift + Enter
    Hook->>Hook: throttle (>=50 ms) and bypass check
    Hook->>Ring: push(Command::MoveToNextMonitor = 8)
    Hook-->>User: return 1 (keystroke swallowed)
    Hook->>Worker: WM_APP_COMMAND_READY

    Worker->>Ring: pop() -> 8
    Worker->>Spatial: resolve_context() (foreground HWND, its monitor)
    alt Foreground window belongs to Wira Desk
        Spatial-->>Worker: None
        Worker->>Worker: Tier-2 diagnostic, nothing moves (LBR-WM-6)
    else Target is eligible
        Worker->>Spatial: enumerate live monitor set (EnumDisplayMonitors)
        alt Exactly one monitor
            Spatial-->>Worker: one entry
            Worker->>Worker: empty plan, successful no-op, nothing logged
        else Two or more monitors
            Spatial-->>Worker: ordered monitor set with work area and DPI
            opt Window is maximized
                Worker->>Win: restore to normal state first
            end
            Worker->>Plan: source work area, destination work area, window rect
            Plan->>Plan: express rect as a share of source, map onto destination
            Plan-->>Worker: PlacementPlan
            Worker->>Win: SetWindowPos (SWP_NOACTIVATE | SWP_NOZORDER)
            Win-->>Worker: ok, or refused
            alt Refused
                Worker->>Worker: Tier-2 diagnostic; window left as-is
            end
        end
        Worker->>Worker: suppress_start_menu()
    end
```

## Why the enumeration sits on the worker, not the hook

The hook callback is budgeted under 10 ms and must not allocate (`NFR-2`, `NFR-3`, `AD-2`). `EnumDisplayMonitors` is a callback-driven sweep whose cost grows with the number of attached displays, and its result is only needed once a command has already been accepted. The hook's whole job here is to recognise the chord and push one byte; every question about *where* the window goes is the worker's.

This mirrors the reasoning `DEC-006` used to put the target-eligibility gate on the worker rather than beside the VM bypass: the hook answers *whose chord is this*, the worker answers *what is a legal target and where does it go*.

## Where the destination comes from

| Step | What is read | What is not |
| --- | --- | --- |
| Resolve source | `MonitorFromWindow` on the foreground window, `MONITOR_DEFAULTTONULL` | A cached monitor from a previous command |
| Enumerate | `EnumDisplayMonitors`, then `GetMonitorInfoW` and `GetDpiForMonitor` per entry | A stored list, a display-change subscription, anything in a `static` |
| Select | The entry after the source in enumeration order, wrapping | Coordinate order, which is undefined for stacked or L-shaped arrangements |
| Map | The window rect as a share of the source **work area**, applied to the destination **work area** | The rect's absolute width and height; full monitor bounds |

## Why a maximized window is restored first

The maximized state is bound to the monitor the window was maximized on. `SetWindowPos` on a window Windows still considers maximized is unreliable — the window springs back rather than landing where it was told. So the sequence is restore, then place to fill the destination work area, which produces the outcome the user expected by a route that actually works. This is the same instinct `UC-2` already applies before a half-screen snap; `UC-7` inherits it rather than inventing a second answer.

## Virtual desktop

`SetWindowPos` does not change virtual desktop membership, so the window stays on the desktop it was on with no extra work. `AD-9` is not weakened by this command — it is the *monitor* half of the spatial boundary that is crossed deliberately, never the desktop half. Verified by reading what `SetWindowPos` affects rather than assumed.

## Known limitation

Frame-inset compensation (`frame_insets`, `compensate_for_frame_insets` in `arrangement/win32.rs`) measures the gap between `GetWindowRect` and the extended frame bounds *before* the move, at the source monitor's scaling. On a move between monitors of different scaling the visible frame lands a few pixels off the planned rectangle. `DEC-007` accepts this and states why a two-pass placement was declined: the second pass would have to wait on Windows' own asynchronous DPI-change reflow, which nothing in this codebase currently waits on.

That "a few pixels off" outcome depends on the border clamp in `Win32WindowMover::apply` resolving its monitor from the *planned* (destination) rect rather than from the window, which is still on the source monitor when `apply` runs. Resolving from the window would clamp the compensated rect against the wrong monitor's bounds; a compensated rect that touches a different-DPI monitor is what Windows itself uses as the cue to relocate and rescale the window — turning the small edge inset into the window landing somewhere nobody planned. `DEC-010` records the correction and why it is measured, not argued.

## Evidence

`[MISSING]` — no part of this flow exists in code yet. `arrangement/monitor.rs` and the `EnumDisplayMonitors` path in `context/spatial.rs` are both planned by this pass. The hook, ring buffer, worker dispatch, and `SetWindowPos` application steps are `[PARTIAL]`: those mechanisms exist and are exercised by the four current arrangement commands (`crates/daemon/src/hook.rs`, `ring.rs`, `worker.rs`, `arrangement/win32.rs`), and only the new command's arms are absent.
