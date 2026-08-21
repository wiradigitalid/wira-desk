---
type: uc
id: UC-2
component: window-management
satisfies: [FR-14]
critical: false
created: '2026-08-21'
---

# UC-2 — Snap the active window to the left or right half of the screen

## Trigger

User presses a configured window snapping keyboard shortcut (e.g. `Ctrl + Win + Left` or `Ctrl + Win + Right`).

## Precondition

- Wira Desk daemon is running with active low-level keyboard hook.
- Foreground window is a standard resizable top-level application window on an active physical monitor.

## Main Flow

1. User presses the snap shortcut while focused on a resizable window.
2. System intercepts keystroke on dedicated hook thread, validates shortcut chord, and enqueues snap command to lock-free ring buffer.
3. System posts command notification to worker thread and returns immediately without blocking input.
4. System worker thread retrieves snap command and identifies active monitor bounds and DPI scale factor.
5. System computes target half-screen coordinates subtracting taskbar and docking work area margins.
6. System executes atomic DPI-aware repositioning and resizing via non-blocking Win32 APIs.
7. User sees active window aligned flush to the targeted screen half at correct pixel dimensions.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 1 | Foreground window is currently maximized | System restores window to normal state before applying half-screen snapping dimensions. |
| Step 2 | Keystroke has unconfigured modifier combinations | System passes keystroke through via `CallNextHookEx` without executing snapping action. |
| Step 4 | Window spans multi-monitor boundary | System determines primary containing monitor via center-point calculation and applies snap to that monitor's work area. |
| Step 6 | Window enforces custom minimum size constraints larger than half-screen | System positions window flush to screen edge while respecting application's enforced minimum boundaries. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 4 | Monitor handle invalid or disconnected during hot-unplug | System falls back to primary desktop work area coordinates | Window is safely positioned on primary display |
| Step 6 | Win32 `SetWindowPos` call refused due to target privilege or style lock | System logs Tier 2 diagnostic warning without crashing | Window remains at current position and size |

## Outcome

The active window is cleanly resized and positioned to exactly half of the active monitor's available working area with proper per-monitor DPI scaling and work area boundary adherence.

## Business Rules

- `BR-1` (Settings persistence and IPC reload)
- `BR-3` (UIPI bypass and elevated execution)
- `LBR-WM-1` (Exact shortcut matching only)
- `LBR-WM-3` (Non-blocking kernel API sterilization)
- `LBR-WM-4` (Lock-free drop-on-saturation policy)
