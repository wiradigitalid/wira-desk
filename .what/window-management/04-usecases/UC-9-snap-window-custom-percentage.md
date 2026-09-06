---
type: uc
id: UC-9
component: window-management
satisfies: [FR-26]
critical: false
created: '2026-09-06'
updated: '2026-09-06'
---

# UC-9 — Snap the active window to a screen edge at a custom percentage

## Trigger

User presses a configured custom-percentage snap keyboard shortcut for one of the four edges — left,
right, top, or bottom (shipped defaults `Ctrl + Alt + Shift + Left`, `Ctrl + Alt + Shift + Right`,
`Ctrl + Alt + Shift + Up`, `Ctrl + Alt + Shift + Down`).

## Precondition

- Wira Desk daemon is running with active low-level keyboard hook.
- Foreground window is a standard resizable top-level application window on an active physical monitor.
- The pressed edge has a percentage configured in Settings (default 50%, matching the fixed half-snap
  until the user changes it).

## Main Flow

1. User presses the custom-percentage snap shortcut for one edge while focused on a resizable window.
2. System intercepts keystroke on dedicated hook thread, validates shortcut chord, and enqueues the
   command to the lock-free ring buffer.
3. System posts command notification to worker thread and returns immediately without blocking input.
4. System worker thread retrieves the command and identifies active monitor bounds, DPI scale factor,
   and the percentage configured for the named edge.
5. System computes target coordinates by taking the configured percentage of the work area's width
   (left/right edges) or height (top/bottom edges), measured from the named edge inward.
6. System executes atomic DPI-aware repositioning and resizing via non-blocking Win32 APIs.
7. User sees active window aligned flush to the targeted edge at exactly the configured percentage of
   the work area.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 1 | Foreground window is currently maximized | System restores window to normal state before applying the configured-percentage dimensions. |
| Step 2 | Keystroke has unconfigured modifier combinations | System passes keystroke through via `CallNextHookEx` without executing any snapping action. |
| Step 4 | Foreground window belongs to Wira Desk itself — the Settings window or its onboarding modal | System resolves no target and arranges nothing. The chord stays consumed rather than passed back to Windows (`LBR-WM-6`, `DEC-006`). |
| Step 4 | Window spans a multi-monitor boundary | System determines the primary containing monitor via center-point calculation and applies the snap to that monitor's work area. |
| Step 5 | Work area is too small to produce a non-zero extent at the configured percentage | System plans nothing and reports a planning failure rather than emitting a zero-extent placement. Nothing moves. |
| Step 6 | Window enforces custom minimum size constraints larger than the configured percentage | System positions the window flush to the named edge while respecting the application's enforced minimum boundaries. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 4 | Monitor handle invalid or disconnected during hot-unplug | System falls back to primary desktop work area coordinates | Window is safely positioned on primary display |
| Step 6 | Win32 `SetWindowPos` call refused due to target privilege or style lock | System logs Tier 2 diagnostic warning without crashing | Window remains at current position and size |

## Outcome

The active window is cleanly resized and positioned against the named edge at exactly the percentage of
the active monitor's available work area configured for that edge — proper per-monitor DPI scaling and
work area boundary adherence, independent of what the fixed half-snap (`UC-2`) does with the same edge.

## Business Rules

- `BR-1` (Explicit IPC configuration reload)
- `LBR-WM-1` (Exact shortcut matching only)
- `LBR-WM-3` (Non-blocking kernel API sterilization)
- `LBR-WM-4` (Lock-free drop-on-saturation policy)
- `LBR-WM-6` (Arrangement target eligibility)
- `LBR-WM-9` (Deterministic percentage division)
