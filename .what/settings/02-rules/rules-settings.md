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
| LBR-ST-14 | Every action whose chord a user may edit appears as exactly **one** row in the Shortcuts pane, and one declared sequence is the single source of the pane's draw order, its keyboard focus order, and the precedence order that resolves a chord collision. A second, independently maintained list of the same actions must not exist. Grouping the rows under headings must not reorder them relative to that sequence. | `settings`, `window-management` | FR-18, LBR-ST-5, BR-6, DEC-009 | active |

## Rationale — LBR-ST-14

The list of editable actions grew from six to nine in one pass, and it will grow again. The failure this rule prevents is not hypothetical in this codebase: the source already carries comments explaining that the draw order and the declared order must not become two lists, because they had been, and that a field added to the field-to-key table but not to its reverse lookup breaks the round trip silently rather than at compile time.

The rule also makes the collision precedence order (`BR-6`, `DEC-009`) inspectable. Precedence is arbitrary by nature; what makes it defensible is that a reader can verify it against one declared sequence in a few seconds, and that the pane they are looking at is drawn from that same sequence.

Grouping is explicitly permitted and explicitly constrained. Nine undifferentiated rows are hard to scan, so headings earn their place — but a heading that reorders rows relative to the declared sequence would put the visible order and the precedence order back into disagreement, which is the whole thing this rule exists to stop.

## Retired

*(No retired local rules)*

