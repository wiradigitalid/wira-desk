---
type: scn
id: SCN-03
component: settings
attaches_to: UC-4
created: '2026-08-25'
updated: '2026-08-25'
---

# SCN-03 — Two actions left holding the same chord

## Where it branches

Leaves from **UC-4 (Change a keyboard shortcut in Settings)** at **Step 4**, after the captured chord
has become the draft value and before the draft is submitted.

## Condition

The chord just accepted for one action is already held by another of the configurable actions.
Nothing about the chord itself is wrong: it carries a modifier, it carries one main key, and the
grammar recognises every token.

## Flow

1. User activates a shortcut field and presses a chord that is legal in isolation.
2. The chord becomes the draft value for that action.
3. Settings marks that action and the action already holding the chord, each naming the other, so
   the pair can be identified without searching the pane. The submit action stays available
   throughout.
4. User either resolves the collision — by giving one of the two a different chord, or by
   exchanging the two actions' chords — or submits the draft as it stands.
5. If resolved, the main flow is rejoined at UC-4 Step 5. The remaining steps describe what happens
   instead if the user submits without resolving it.
6. Settings refuses the draft, names both actions in the refusal, and writes nothing.
7. Settings signals no reload; the configuration already on disk keeps the pair it had. User
   resolves the collision and submits again, rejoining the main flow at UC-4 Step 5.

## Outcome

The user resolves the collision and the draft commits by the main flow, rejoining at UC-4 Step 5 —
or Settings refuses it at submission, leaving the draft intact with both offending actions named;
the user remains free to resolve it and try again.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| Collision standing in the draft | Both actions marked, each naming the other | — |
| Collision submitted unresolved | Refusal naming both actions; nothing written and no reload signalled | The refusal reason and the two actions |

## Why it is not in the UC

It is the only rejection that survives capture and reaches submission, so it carries a second
decision point — resolve or submit anyway — that the use case's straight line from activate to save
has no room for.

## Notes

Separate from SCN-01 because the two refuse at different moments. SCN-01's classes never enter the
draft and hold the field in listening; a collision is legal in isolation, enters the draft, and is
refused only on submission. Folding them into one scenario would have made SCN-01's title —
*rejected before save* — untrue of half its own content.

The submit action is never disabled to express this collision. `DEC-001` governs, and states the
reason: a disabled submit has to explain which field disabled it — something a user facing nine
shortcut fields cannot work out.
