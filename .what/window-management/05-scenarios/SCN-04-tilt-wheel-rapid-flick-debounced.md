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

Without software-level debounce filtering, a single intentional tilt flick to switch virtual desktops would trigger multiple rapid switches, skipping 2 to 4 desktops and landing on the wrong workspace. Furthermore, holding the tilt wheel down on some hardware generates an uninterrupted stream of `WM_MOUSEHWHEEL` ticks; without a gesture lockout, this causes rapid toggle flickering (e.g. toggling Task View or Show Desktop every 150 ms). `LBR-WM-11` and `AD-15` define the two-phase filtering policy: a 150 ms debounce for rapid flick bounce, plus a 400 ms quiet-period gesture lockout for sustained holds, ensuring exactly one action executes per physical actuation.

## Flow — rapid tilt-wheel flick & hold

| Step | What happens |
| --- | --- |
| 1 | User physically tilts the mouse scroll wheel to the left to trigger Show Desktop, or holds it tilted. |
| 2 | Hardware delivers the first `WM_MOUSEHWHEEL` message with negative delta (`WHEEL_DELTA`). |
| 3 | System's low-level mouse hook (`WH_MOUSE_LL`) intercepts the message, verifies that no gesture lockout is active, updates `last_tilt_ms = now`, arms the gesture lockout, swallows the message (`return 1`), and enqueues the command to the ring buffer. |
| 4 | Worker thread wakes up, synthesizes `Win + D`, and Windows smoothly shows the desktop. |
| 5 | Within 35 ms of step 2, the physical switch delivers second and third `WM_MOUSEHWHEEL` burst ticks. |
| 6 | System's low-level mouse hook intercepts each burst tick; because gesture lockout is armed (or elapsed time is within bounce window), each tick is swallowed (`return 1`) without queuing duplicate commands. |
| 7 | If the user sustains/holds the tilt wheel, continuous `WM_MOUSEHWHEEL` repeat messages continue arriving; gesture lockout swallows every repeating tick so long as ticks arrive without a 400 ms quiet gap. |
| 8 | User releases the tilt wheel. Once 400 ms elapses with zero `WM_MOUSEHWHEEL` ticks, the gesture lockout disarms, leaving the system ready for the next intentional actuation. |

## What the user sees, and what they do not

The user experiences exactly one instant, clean virtual desktop transition for their single finger flick. They do not see the desktop skip multiple workspaces, and the active application does not perform horizontal scrolling.

## Business Rules

- `BR-10` (Mouse Navigation Action Mapping and Passthrough)
- `LBR-WM-11` (Mouse motion passthrough and tilt debounce)
- `AD-15` (WH_MOUSE_LL & Motion Passthrough)
