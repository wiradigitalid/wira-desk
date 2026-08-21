---
type: uc
id: UC-3
component: window-management
satisfies: [FR-10, FR-11, FR-12]
critical: false
created: '2026-08-21'
---

# UC-3 — See the tray icon return after Windows Explorer restarts

## Trigger

Windows Explorer crashes, is terminated by the user/system, or restarts following a display configuration or shell update.

## Precondition

- Wira Desk daemon is executing in background with system tray integration active.
- Daemon has registered for the operating system `TaskbarCreated` broadcast message.

## Main Flow

1. Windows Explorer shell terminates and restarts, clearing existing notification area tray icons.
2. Operating system initializes new taskbar and broadcasts registered `TaskbarCreated` window message.
3. System daemon receives `TaskbarCreated` message on its dedicated Win32 message loop window.
4. System verifies current internal tray health state (`Normal`, `Warning`, or `Critical`).
5. System re-registers notification area icon via `Shell_NotifyIconW(NIM_ADD)` with appropriate icon asset, tooltip, and callback identifier.
6. System re-establishes tray context menu bindings (`Settings`, `View Logs`, `Run at Startup`, `Check for Updates`, `Exit`).
7. User sees Wira Desk icon reappear in taskbar notification tray ready for interaction without utility restart.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 4 | Tray health state has active warning latched (`warning_latched == true`) | System restores tray icon showing Amber/Red Dot warning overlay asset. |
| Step 4 | Hook heartbeat failure threshold reached during shell disruption (`fail_count >= 3`) | System restores tray icon showing Critical Red X overlay and updates menu state (see SCN-02). |
| Step 5 | Initial `NIM_ADD` call fails because taskbar is still initializing | System schedules retry sequence (up to 3 attempts with 500 ms backoff) until notification icon is accepted. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 5 | Taskbar registration persistently fails | System logs diagnostic warning and retains core keyboard hook and cycling functionality headless | Shortcut cycling continues functioning normally even without visual tray icon |

## Outcome

The Wira Desk system tray resident icon and context menu are fully restored following Windows Explorer shell restarts, maintaining accurate visual health badges and uninterrupted shortcut handling.

## Business Rules

- `BR-2` (Single-instance mutual exclusion)
- `LBR-WM-5` (One-shot Tier-3 notification latch)
