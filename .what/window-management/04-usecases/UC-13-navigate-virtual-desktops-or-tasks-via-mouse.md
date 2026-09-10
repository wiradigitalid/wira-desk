---
type: uc
id: UC-13
component: window-management
satisfies: [FR-30, FR-31]
critical: false
created: '2026-09-10'
---

# UC-13 — Navigate virtual desktops or tasks via mouse thumb buttons and tilt wheel

## Trigger

User clicks an auxiliary mouse button (Thumb Button 1 or 2) or tilts the scroll wheel horizontally (Left or Right) on a multi-button productivity mouse while mouse navigation is enabled in configuration.

## Precondition

- Wira Desk daemon is running elevated with active low-level keyboard and mouse hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`).
- Mouse navigation is enabled (`mouse.enabled = true`) in `config.toml`, and the physical mouse input is mapped to an action preset (e.g. Next Virtual Desktop, Previous Virtual Desktop, Task View, or Show Desktop).

## Main Flow

1. User clicks a mapped thumb button or tilts the mouse wheel horizontally.
2. System's low-level mouse hook (`WH_MOUSE_LL`) intercepts the input event (`WM_XBUTTONDOWN` or `WM_MOUSEHWHEEL`).
3. System verifies mouse navigation is enabled and the input matches a configured action preset.
4. For tilt-wheel events (`WM_MOUSEHWHEEL`), system checks the debounce timer; if the elapsed time since the previous tilt tick is within the debounce window (150–200 ms), the event is dropped.
5. System swallows the event (`return 1`), preventing the host operating system or active application from executing default browser navigation or horizontal scrolling.
6. Hook thread enqueues the mapped command byte to the lock-free ring buffer and signals the worker thread via `WM_APP_COMMAND_READY`.
7. Worker thread drains the command and executes the target action (e.g. synthesizing `Ctrl + Win + Right` for Next Virtual Desktop, or executing window cycling/snapping).
8. Windows desktop or window focus updates immediately without cursor hitching or visual overlays.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 3 | The physical mouse input is set to `passthrough` or `default` | System calls `CallNextHookEx` immediately; the event reaches the active application as a standard Windows mouse event (e.g. browser back/forward). |
| Step 4 | User rapidly flicks the tilt wheel generating multiple hardware ticks | First tick passes and resets debounce timer; subsequent ticks within 150–200 ms are swallowed without queuing duplicate commands (`SCN-04`). |
| Step 7 | Configured action is an internal Wira Desk action (e.g. Cycle Same-App Window) | Worker thread initiates live Z-order traversal and shifts focus to the next same-application window (`UC-1`). |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 2 | Mouse event is cursor movement (`WM_MOUSEMOVE`) | System immediately calls `CallNextHookEx` without locks, memory allocations, or logging | Continuous, fluid cursor movement with zero micro-stutter |
| Step 6 | Ring buffer is completely full | Hook thread increments dropped-metric counter and returns `1` | Command is dropped; hook thread remains non-blocking |

## Outcome

The user navigates virtual desktops, triggers Task View, or controls window layout seamlessly from their mouse hand with zero perceived input lag and without installing third-party vendor companion software.

## Business Rules

- `BR-10` (Mouse Navigation Action Mapping and Passthrough)
- `LBR-WM-3` (Non-blocking window enumeration)
- `LBR-WM-4` (Command channel capacity)
- `LBR-WM-11` (Mouse motion passthrough and tilt debounce)
