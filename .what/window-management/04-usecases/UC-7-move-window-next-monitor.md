---
type: uc
id: UC-7
component: window-management
satisfies: [FR-23]
critical: false
created: '2026-08-26'
---

# UC-7 — Move the active window to the next monitor

## Trigger

User presses the configured monitor-move shortcut (shipped default `Ctrl + Alt + Shift + Enter`).

## Precondition

- Wira Desk daemon is running with active low-level keyboard hook.
- Foreground window is a standard resizable top-level application window on an active physical monitor.
- More than one monitor is attached. With one, see Alternate Flows.

## Main Flow

1. User presses the monitor-move shortcut while focused on a window.
2. System intercepts keystroke on the hook thread, validates the chord, and enqueues the move command to the lock-free ring buffer.
3. System posts command notification to the worker thread and returns immediately without blocking input.
4. System worker thread retrieves the command, resolves the foreground window's monitor, and enumerates the live monitor set.
5. System selects the monitor following the current one in the enumeration order, wrapping from the last back to the first.
6. System expresses the window's current rectangle as a share of its source work area, then maps that share onto the destination work area.
7. System executes atomic repositioning and resizing via non-blocking Win32 APIs, without changing which virtual desktop shows the window.
8. User sees the window on the next monitor holding the same share of the screen it held before.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | Keystroke has unconfigured modifier combinations | System passes the keystroke through via `CallNextHookEx` without moving anything. |
| Step 4 | Foreground window belongs to Wira Desk itself | System resolves no target and moves nothing. The chord stays consumed rather than passed back to Windows, and the refusal is a Tier-2 diagnostic with no popup. `LBR-WM-6`. |
| Step 4 | Exactly one monitor is attached | System plans nothing. Nothing moves, no message is shown, and no warning is logged — this is a successful no-op, not a failure. |
| Step 6 | Foreground window is currently maximized | System restores it to its normal state before placing it, the same way a half-screen snap does, then places it to fill the destination work area. Repositioning a window Windows still considers maximized is unreliable: the maximized state is bound to the monitor it was maximized on, and the window springs back. The user sees a window filling the new monitor — the outcome they expected — reached by restore-then-place rather than by moving a maximized window. |
| Step 6 | Source and destination monitors differ in display scaling | System maps by share regardless. Windows re-scales the window's own content on the DPI change; the system does not compensate for that a second time. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 4 | Monitor enumeration fails, or the foreground window resolves to no monitor | System plans nothing and records a Tier-2 diagnostic | Window stays exactly where it was |
| Step 5 | Destination monitor disappears between enumeration and placement — a hot-unplug inside the same command | System attempts the placement and the Win32 call is refused or clamped by Windows; the refusal is recorded as a Tier-2 diagnostic | Window is left wherever Windows placed it, on an attached monitor; the next press moves it again |
| Step 6 | Destination work area is empty or unrepresentable | System reports a planning failure and emits no placement | Window stays exactly where it was |
| Step 7 | Win32 `SetWindowPos` refused due to target privilege or style lock | System logs a Tier-2 diagnostic warning without crashing | Window remains at its current position and size |
| Step 7 | Source and destination monitors differ in display scaling, so the frame allowance measured before the move no longer describes the window | System places the window anyway; no warning, because nothing failed | Window is on the right monitor at the right share, with its visible edge a few pixels off the intended rectangle. Accepted rather than corrected — `DEC-007` states why a second placement pass was declined. Staying a small edge inset rather than a full relocation depends on the border clamp resolving against the destination monitor, not the source (`DEC-010`) |

## Outcome

The active window sits on the next attached monitor, occupying the same share of that monitor's work area as it occupied on the monitor it left, on the same virtual desktop as before. No other window on either monitor has moved. Pressing the shortcut once per attached monitor returns the window to where it started.

## Business Rules

- `BR-3` (UIPI bypass and elevated execution)
- `BR-6` (One chord, one action)
- `LBR-WM-1` (Exact shortcut matching only)
- `LBR-WM-3` (Non-blocking kernel API sterilization)
- `LBR-WM-4` (Lock-free drop-on-saturation policy)
- `LBR-WM-6` (Arrangement target eligibility)
- `LBR-WM-7` (Monitor-move semantics)
