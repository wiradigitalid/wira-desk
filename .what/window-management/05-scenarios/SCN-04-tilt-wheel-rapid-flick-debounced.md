---
type: scenario
id: SCN-04
component: window-management
relates_to: [UC-13]
created: '2026-09-10'
---

# SCN-04 — A rapid tilt-wheel flick fires multiple hardware delta ticks

## Why this scenario exists

Physical tilt-wheel mechanisms on productivity mice (such as Logitech M-series or MX Master) use mechanical or optical micro-switches that frequently emit multiple rapid `WM_MOUSEHWHEEL` delta events (e.g. 2 to 5 consecutive ticks with delta ±120) within tens of milliseconds from a single physical flick of the user's finger.

Without software-level debounce filtering, a single intentional tilt flick to switch virtual desktops would trigger multiple rapid switches, skipping 2 to 4 desktops and landing on the wrong workspace. `LBR-WM-11` and `AD-15` define the debounce threshold (150–200 ms) to ensure exactly one action executes per physical flick.

## Flow — rapid tilt-wheel flick

| Step | What happens |
| --- | --- |
| 1 | User physically tilts the mouse scroll wheel to the left to trigger Next Virtual Desktop. |
| 2 | Hardware delivers the first `WM_MOUSEHWHEEL` message with negative delta (`WHEEL_DELTA`). |
| 3 | System's low-level mouse hook (`WH_MOUSE_LL`) intercepts the message, verifies that elapsed time since `last_tilt_ms` exceeds the debounce window (e.g. 150 ms), updates `last_tilt_ms = now`, swallows the message (`return 1`), and enqueues the command to the ring buffer. |
| 4 | Worker thread wakes up, synthesizes `Ctrl + Win + Right`, and Windows transitions smoothly to the next virtual desktop. |
| 5 | Within 35 ms of step 2, the physical switch delivers second and third `WM_MOUSEHWHEEL` burst ticks. |
| 6 | System's low-level mouse hook intercepts each burst tick, compares elapsed time (`now - last_tilt_ms < 150 ms`), drops the event from ring enqueueing, and swallows the message (`return 1`). |
| 7 | User releases the tilt wheel. No further messages arrive until the next deliberate physical tilt. |

## What the user sees, and what they do not

The user experiences exactly one instant, clean virtual desktop transition for their single finger flick. They do not see the desktop skip multiple workspaces, and the active application does not perform horizontal scrolling.

## Business Rules

- `BR-10` (Mouse Navigation Action Mapping and Passthrough)
- `LBR-WM-11` (Mouse motion passthrough and tilt debounce)
- `AD-15` (WH_MOUSE_LL & Motion Passthrough)
