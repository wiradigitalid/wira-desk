---
type: scn
id: SCN-02
component: window-management
attaches_to: UC-3
created: '2026-08-21'
---

# SCN-02 — Hook death detection and recovery attempt

## Where it branches

Leaves from **UC-3 (See the tray icon return after Windows Explorer restarts)** at **Alternate Step 4b** (Heartbeat failure handling) and runs alongside ongoing background health monitoring.

## Condition

External system events, antivirus/EDR intervention, or OS `LowLevelHooksTimeout` unhooking silently invalidates the active `WH_KEYBOARD_LL` hook handle, causing consecutive heartbeat refresh attempts to fail (`fail_count >= HOOK_CHECK_FAIL_THRESHOLD = 3`).

## Flow

1. Daemon health monitoring thread dispatches periodic `WM_APP_HOOK_CHECK` message every 10 seconds to the Hook Thread.
2. Hook thread attempts to re-register `WH_KEYBOARD_LL` hook via `SetWindowsHookExW` and detect responsiveness.
3. Hook registration fails repeatedly across 3 consecutive heartbeat cycles (30-second window).
4. Hook thread transitions state from `Degraded` to `Dead` and posts `WM_APP_HOOK_DEAD` to Worker thread.
5. Worker thread switches tray health state to `Critical`, modifies notification area icon to display **Red X overlay**, and updates context menu status.
6. System checks one-shot notification latch (`hook_dead_toast_sent`); finding it `false`, dispatches exactly one Windows Toast Notification informing the user of degraded shortcut interception, then sets latch to `true` (AD-7, LBR-WM-5).
7. System continues executing background heartbeat checks without crashing or terminating the tray process.
8. On a subsequent heartbeat check, security block clears or OS resource recovers, allowing `SetWindowsHookExW` to succeed.
9. Hook thread unhooks stale handle, resets `fail_count = 0`, transitions state to `Active`, and posts `WM_APP_HOOK_REFRESH_OK` to Worker.
10. Worker thread resets `hook_dead_toast_sent = false` and restores tray icon to `Normal` (or `Warning` if `warning_latched == true`).
11. User can resume global keyboard shortcut cycling and snapping immediately.

## Outcome

System gracefully signals critical hook impairment to the user without spamming notifications, keeps daemon UI alive, and self-heals immediately once operating system hook registration is restored.

## Why it is not in the UC

Covers the complete 3-tier error protocol escalation, one-shot notification latching, and automatic background self-healing recovery loop separate from the explorer shell restart use case.
