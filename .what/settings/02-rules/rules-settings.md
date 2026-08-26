---
type: rules
scope: component
component: settings
status: reviewed
created: '2026-08-21'
updated: '2026-08-26'
---

# Business Rules — settings

Local component business rules binding the `settings` Product Component. Global cross-component rules (`BR-1` through `BR-5`) live in `.what/business-rules.md`.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| LBR-ST-1 | Shortcut fields accept listening-mode physical key combinations only, capturing modifiers and a single main key directly, and must not accept arbitrary typed text. | `settings` | FR-18, AD-11 | active |
| LBR-ST-2 | Configuration persistence must complete an atomic temporary-file write and replacement before an explicit `WM_APP_RELOAD_CONFIG` IPC reload signal is dispatched to the daemon. | `settings` | FR-7, BR-1, AD-5 | active |
| LBR-ST-3 | The "Skip Tutorial" action must remain accessible and clearly visible on every step of the first-run onboarding flow. | `settings` | FR-17 | active |
| LBR-ST-4 | Auto-start scheduled tasks must always be created with `/RU %USERNAME%` and `/RL HIGHEST`, and must never run under the `SYSTEM` account. | `settings` | FR-13, BR-4, AD-13 | active |
| LBR-ST-5 | All interactive settings controls must follow a deterministic Tab navigation order that starts with navigation tabs and terminates with action buttons. | `settings` | FR-20, AD-11a | active |
| LBR-ST-6 | Shortcut capture listening state and validation feedback must be announced via UI Automation accessible values, never communicated through visual styling alone. | `settings` | FR-21, AD-11a | active |
| LBR-ST-7 | Completing or skipping the first-run tutorial must write a valid configuration to disk so that onboarding does not re-trigger on subsequent launches. | `settings` | FR-17, BR-3 | active |
| LBR-ST-8 | A refused shortcut must be reported on the field carrying it; when the refusal is a collision with another action, the report must name both actions. | `settings` | FR-18, DEC-001 | active |
| LBR-ST-9 | The submit action must never be disabled to express a shortcut refusal; a draft holding a collision must be refused when it is submitted. | `settings` | FR-18, DEC-001 | active |
| LBR-ST-10 | A chord the Windows shell already owns must be refused and never claimed for a Wira Desk action, and the refusal must name the Windows function the user would have lost. Where the chord is one Wira Desk could technically have taken, the refusal must also offer a way through; where Windows keeps it regardless of any hook, it must not, because no alternative would be true. | `settings` | FR-18, DEC-003 | active |
| LBR-ST-11 | The daemon may withhold a chord from Windows on the settings process's behalf only while a shortcut field is recording. While the Shortcuts pane is merely visible, the daemon must report the chord and withhold only its own action, never the keystroke. | `settings`, `window-management` | FR-18, DEC-004, AD-1 | active |
| LBR-ST-12 | The key check must state only what was observed, correlating the daemon's report against what the settings window received, and must never predict whether a chord nobody has pressed is available. With no daemon report available it must say so and stop diagnosing. | `settings` | FR-18, DEC-002, DEC-005 | active |
| LBR-ST-13 | The settings window’s painted size is its Win32 size: a size change imposed from outside the process must be clamped at the window’s own message boundary before the toolkit sees it, while position changes pass through untouched. A size the window itself currently declares legal — the onboarding modal growing into the settings shell — must still be honoured. | `settings` | DEC-006 | active |

## Retired

*(No retired local rules)*

