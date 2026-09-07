---
type: uc
id: UC-12
component: window-management
satisfies: [FR-29]
critical: false
created: '2026-09-07'
---

# UC-12 — A disabled shortcut action's chord is not claimed at the hook

## Trigger

Daemon starts, or receives the configuration-reload IPC signal after a save in Settings (`BR-1`), and the
loaded configuration marks one or more shortcut actions disabled.

## Precondition

- Wira Desk daemon is running with active low-level keyboard hook.
- The configuration being loaded carries a per-action enabled/disabled flag alongside each action's chord
  (`FR-28`, `LBR-ST-17`).

## Main Flow

1. System loads configuration at startup or on reload.
2. For each of the sixteen editable actions, system reads its enabled/disabled flag before considering its
   chord for registration.
3. An action flagged disabled is excluded from the declared sequence's chord set entirely — its chord takes
   no part in hook matching and no part in `BR-6`/`DEC-009`'s collision-precedence resolution, the same as
   if the field were never configured.
4. Every enabled action's chord registers exactly as it does today.
5. User presses the physical key combination that was the disabled action's chord. System's low-level hook
   does not recognize it as any Wira Desk action and passes it through via `CallNextHookEx` unmodified.
6. The keystroke reaches the foreground application, or Windows itself, exactly as it would if Wira Desk
   were not running.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 3 | A disabled action's chord is identical to an enabled action's chord | The enabled action registers normally; the disabled action's absence from the chord set means there is nothing for it to contend with, so no collision diagnostic fires for this pair (`LBR-ST-17`). |
| Step 3 | An action is re-enabled in a later reload, using the same chord it held before being disabled | The chord registers normally on this reload, participating in collision resolution as any other field would. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 1 | Configuration fails to load or fails validation for a reason unrelated to enablement (e.g. `BR-6`'s duplicate-chord reject on the reload path) | System applies existing all-or-nothing reload behavior: the candidate configuration, enablement flags included, is refused in full and the last-known-good configuration keeps running | Enablement state does not change until a valid reload arrives |

## Outcome

Every action the user disabled behaves, from the keyboard's perspective, as if Wira Desk had never bound
that chord — the key combination is free for Windows or any other application to claim, and the user is not
left with a chord that is claimed and merely inert.

## Business Rules

- `BR-9`
- `BR-6` (One chord, one action — disabled actions are outside its scope entirely)
- `LBR-ST-17`
- `LBR-WM-1` (Exact shortcut matching only)
