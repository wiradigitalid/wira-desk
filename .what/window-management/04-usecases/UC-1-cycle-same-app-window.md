---
type: uc
id: UC-1
component: window-management
satisfies: [FR-1, FR-2, FR-3, FR-4, FR-5, FR-6]
critical: false
created: '2026-08-21'
---

# UC-1 — Cycle to the next window of the same app on this monitor

## Trigger

User presses the configured keyboard cycling shortcut (default `Win + \`` or custom modifier chord).

## Precondition

- Wira Desk daemon is running with active `WH_KEYBOARD_LL` hook.
- Foreground window is a standard visible window belonging to an application running on the active physical monitor and virtual desktop.
- User is working inside normal desktop applications (not a bypassed VM or remote desktop session).

## Main Flow

1. User presses the cycling shortcut while focused on an application window.
2. System intercepts the keystroke and validates exact modifier and key matching within 10 ms.
3. System discovers the foreground window handle, executable identity, monitor bounds, and virtual desktop ID.
4. System executes stateless live Z-order enumeration across top-level windows.
5. System filters and identifies candidate windows sharing identical process binary identity on the current monitor and virtual desktop.
6. System selects the next eligible window in Z-order succession.
7. System transfers keyboard focus and brings the selected window to the foreground without animations, switcher HUDs, or preview overlays.
8. User continues typing immediately in the newly focused window.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | Keystroke contains extra or unconfigured modifier keys | System passes keystroke through unmodified to OS via `CallNextHookEx` without cycling. |
| Step 3 | Foreground window is a virtual machine or remote desktop client | System evaluates bypass list, forwards keystroke directly to guest/remote session (see SCN-01), and takes no action. |
| Step 5 | Only one window exists for the active application on this monitor | System maintains focus on current window without UI disruption or audio beeps. |
| Step 5 | Next candidate window is marked "Not Responding" by Windows OS | System brings unresponsive window to foreground honestly without skipping or hiding state (upholding UX honesty). |
| Step 7 | Elevated Administrator target window requires focus transfer | Elevated daemon executes UIPI-immune focus activation, successfully raising the target window. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 4 | Window enumeration or focus transfer API fails | System logs diagnostic warning and releases focus lock | Current window retains focus; user input is not blocked |
| Step 7 | Target window destroyed or closed during enumeration race | System falls back gracefully to remaining top Z-order window | Focus settles on active window; daemon continues normal operation |

## Outcome

Keyboard focus and active window state transfer immediately to the next same-application window on the active monitor and virtual desktop with sub-millisecond perceived latency and zero visual flicker.

## Business Rules

- `BR-1` (Explicit IPC configuration reload)
- `LBR-WM-1` (Exact shortcut matching only)
- `LBR-WM-2` (Live candidate filtering and UX honesty)
- `LBR-WM-3` (Non-blocking kernel API sterilization)
- `LBR-WM-4` (Lock-free drop-on-saturation policy)
