---
type: uc
id: UC-11
component: settings
satisfies: [FR-28]
critical: false
created: '2026-09-07'
---

# UC-11 — Turn a shortcut action on or off

## Trigger

User toggles the on/off control on any editable shortcut action's row in the Shortcuts pane and clicks
Save.

## Precondition

- Settings is open and displaying the Shortcuts pane, which lists every editable action as exactly one row
  (`LBR-ST-14`).
- The action's stored chord, if any, is unchanged by this use case — turning an action off never clears or
  invalidates the chord recorded for it.

## Main Flow

1. User clicks the on/off control on an action's row. The row marks itself disabled immediately in the
   draft, the same way any other field edit marks the draft dirty.
2. User clicks Save. System validates the draft, persists the enabled/disabled flag alongside the action's
   existing chord, and dispatches the same atomic-write-then-IPC-reload-signal sequence every other save
   already uses (`BR-1`).
3. System re-renders the Shortcuts pane from the saved configuration. The disabled row reads visibly as
   user-turned-off — distinct wording or presentation from a row `DEC-009` left unbound over a chord
   collision (`BR-9`), so the user is never left guessing which of the two applies to a row that shows no
   chord taking effect.
4. Daemon receives the reload signal and excludes the disabled action's chord from hook registration (`UC-12`,
   `FR-29`) on its next config load.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 1 | User turns a previously disabled action back on | The row returns to its normal enabled state, showing the same chord it held before being disabled — nothing about the stored chord changed while the action was off (`LBR-ST-17`). |
| Step 2 | The action being disabled currently shares its chord with another action, one of them already unbound by `DEC-009`'s collision resolution | Disabling either action removes it from collision consideration entirely; a disabled action is never a candidate for `find_conflict`, so it can neither win nor lose a chord it no longer contends for (`LBR-ST-17`). |
| Precondition | Configuration was written before this feature existed, or is missing the flag for any other reason | Every action reads as enabled — the same behavior that install already had. Absence of the flag must never be interpreted as the user having turned an action off (`LBR-ST-17`). |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 2 | Save is otherwise rejected for a reason unrelated to this action (e.g. a different field fails validation) | System refuses the whole save and names the offending field, per the existing all-or-nothing save contract | This action's on/off toggle remains as edited in the draft, unsaved, exactly like any other pending field edit |

## Outcome

The user has exactly the set of shortcut actions active that they want; every action they turned off
returns its chord to Windows and other applications, and every row unambiguously shows whether it is off
because the user chose that or because a chord collision left it unbound.

## Business Rules

- `BR-9`
- `LBR-ST-14`
- `LBR-ST-17`
