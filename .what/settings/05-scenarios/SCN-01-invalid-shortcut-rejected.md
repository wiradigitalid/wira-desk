---
type: scn
id: SCN-01
component: settings
attaches_to: UC-4
created: '2026-08-21'
updated: '2026-08-26'
---

# SCN-01 — Invalid shortcut rejected before save

## Where it branches

Leaves from **UC-4 (Change a keyboard shortcut in Settings)** at **Step 7** (chord validation), before anything is written to the draft.

## Condition

The captured chord carries no modifier (a bare key such as `A` or `Tab`), or more than one main key (`Ctrl + A + B`), or a token the canonical grammar does not recognise, or it is a chord the Windows shell already owns (`DEC-003`), or it is a virtual-key code the shared vocabulary has no canonical name for.

## Flow

1. User activates a shortcut field on the Shortcuts pane; `LC-shortcut-capturer` enters `Listening`.
2. User presses a single letter key with no `Win`, `Ctrl`, or `Alt` held.
3. `validate_shortcut` returns `Err(ShortcutError::NoModifier)`; a multi-key chord would return `MultipleMainKeys`.
4. The draft chord is left at its previous valid value — a rejected capture never becomes the draft.
5. Capture state **stays in `Listening`**, so the user can press a valid chord without reactivating the field. Only `Escape` returns it to `Idle`.
6. The inline error renders next to the field, and the reason is exposed to assistive technology as an accessible value rather than by colour alone (LBR-ST-6).
7. User presses a valid chord, or presses `Escape` to abandon the change.

A reserved-chord refusal follows the same path with one difference: while the field is listening the
daemon withholds the chord from Windows (LBR-ST-11), so the shell cannot act on it and take the
foreground before step 6 renders. Without that, `Win+E` opens Explorer, Settings loses focus, and the
refusal never reaches the screen — which is what this scenario used to do.

## Outcome

Nothing is written. The on-disk `config.toml` is untouched, no `WM_APP_RELOAD_CONFIG` is sent, and the daemon keeps the shortcuts it already has.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| Chord has no modifier | Inline error naming the field and what it needs | `debug!` validation reason |
| Chord has several main keys | Inline error naming the field and what it needs | `debug!` validation reason |
| Chord is one the Windows shell owns and Wira Desk could have taken | Inline refusal naming the Windows function, plus a chord that is free | `debug!` catalogue entry matched |
| Chord is one Windows keeps regardless of any hook (`Ctrl+Alt+Del`, `Win+L`) | Inline refusal naming the Windows function, with no alternative offered — none would be true | `debug!` catalogue entry matched |
| Reported virtual-key code has no canonical name | Inline refusal saying the key cannot be used in a shortcut | `debug!` unnameable vk |

## Why it is not in the UC

Keeps the rejection grammar and the listening-state rules out of the success path, which is a straight line from activate to save.

## Notes

Save is not gated on validity: the button is always live (`crates/settings/src/main.rs`), and an invalid draft is refused at save time by `validate_config`, surfacing as `SaveFeedback::Error`. A design that disabled Save would have to explain which field disabled it, which is why rejection is reported per field instead.

A chord the shell owns is refused here, never overridden: an override would make one keypress do two
different things depending on whether a background process happens to be alive, and neither state is
visible at the moment it is pressed (`DEC-003`). A chord an *external* application holds is a
different matter again — nothing is refused, because nothing is predicted (`DEC-002`), and what the
user gets instead is an observation after the fact (`DEC-005`).

A legal chord already held by another action is not this scenario's condition — it enters the draft rather than being held back, and is refused only at submission. That case is **SCN-03**.
