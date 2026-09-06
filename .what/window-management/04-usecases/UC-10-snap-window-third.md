---
type: uc
id: UC-10
component: window-management
satisfies: [FR-27]
critical: false
created: '2026-09-06'
updated: '2026-09-06'
---

# UC-10 — Snap the active window to a third of the screen

## Trigger

User presses a configured thirds-snap keyboard shortcut for one of the three columns — left, middle, or
right (shipped defaults `Ctrl + Alt + 1`, `Ctrl + Alt + 2`, `Ctrl + Alt + 3`).

## Precondition

- Wira Desk daemon is running with active low-level keyboard hook.
- Foreground window is a standard resizable top-level application window on an active physical monitor.

## Main Flow

1. User presses the thirds-snap shortcut for one column while focused on a resizable window.
2. System intercepts keystroke on dedicated hook thread, validates shortcut chord, and enqueues the
   command to the lock-free ring buffer.
3. System posts command notification to worker thread and returns immediately without blocking input.
4. System worker thread retrieves the command and identifies active monitor bounds and DPI scale factor.
5. System divides the work area's width into three columns computed fresh on every press — any
   remainder pixel width goes to the middle column, so the three columns exactly tile the work area
   with neither a gap nor an overlap — and selects the column the chord named.
6. System executes atomic DPI-aware repositioning and resizing via non-blocking Win32 APIs.
7. User sees the active window aligned flush to the targeted column at exactly one third of the work
   area's width and full work-area height.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 1 | Foreground window is currently maximized | System restores window to normal state before applying the third-width dimensions. |
| Step 2 | Keystroke has unconfigured modifier combinations | System passes keystroke through via `CallNextHookEx` without executing any snapping action. |
| Step 4 | Foreground window belongs to Wira Desk itself — the Settings window or its onboarding modal | System resolves no target and arranges nothing. The chord stays consumed rather than passed back to Windows (`LBR-WM-6`, `DEC-006`). |
| Step 4 | Window spans a multi-monitor boundary | System determines the primary containing monitor via center-point calculation and applies the snap to that monitor's work area. |
| Step 5 | Work area is too narrow to divide into three non-zero columns | System plans nothing and reports a planning failure rather than emitting a zero-extent placement. Nothing moves. |
| Step 6 | Window enforces custom minimum size constraints larger than one third of the work area | System positions the window flush to the named column while respecting the application's enforced minimum boundaries. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 4 | Monitor handle invalid or disconnected during hot-unplug | System falls back to primary desktop work area coordinates | Window is safely positioned on primary display |
| Step 6 | Win32 `SetWindowPos` call refused due to target privilege or style lock | System logs Tier 2 diagnostic warning without crashing | Window remains at current position and size |

## Outcome

The active window is cleanly resized and positioned to exactly one third of the active monitor's
available work area — the column the chord named — with proper per-monitor DPI scaling and work area
boundary adherence. The three columns cover the work area exactly, with no gap and no overlap between
them, and a width not evenly divisible by three is divided the same way on every press.

## Business Rules

- `BR-1` (Explicit IPC configuration reload)
- `LBR-WM-1` (Exact shortcut matching only)
- `LBR-WM-3` (Non-blocking kernel API sterilization)
- `LBR-WM-4` (Lock-free drop-on-saturation policy)
- `LBR-WM-6` (Arrangement target eligibility)
- `LBR-WM-10` (Deterministic thirds division)
