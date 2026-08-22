---
type: scn
id: SCN-01
component: settings
attaches_to: UC-4
created: '2026-08-21'
updated: '2026-08-21'
---

# SCN-01 — Invalid shortcut rejected before save

## Where it branches

Leaves from **UC-4 (Change a keyboard shortcut in Settings)** at **Step 7** (chord validation), before anything is written to the draft.

## Condition

The captured chord carries no modifier (a bare key such as `A` or `Tab`), or more than one main key (`Ctrl + A + B`), or a token the canonical grammar does not recognise.

## Flow

1. User activates a shortcut field on the Shortcuts pane; `LC-shortcut-capturer` enters `Listening`.
2. User presses a single letter key with no `Win`, `Ctrl`, or `Alt` held.
3. `validate_shortcut` returns `Err(ShortcutError::NoModifier)`; a multi-key chord would return `MultipleMainKeys`.
4. The draft chord is left at its previous valid value — a rejected capture never becomes the draft.
5. Capture state **stays in `Listening`**, so the user can press a valid chord without reactivating the field. Only `Escape` returns it to `Idle`.
6. The inline error renders next to the field, and the reason is exposed to assistive technology as an accessible value rather than by colour alone (LBR-ST-6).
7. User presses a valid chord, or presses `Escape` to abandon the change.

## Outcome

Nothing is written. The on-disk `config.toml` is untouched, no `WM_APP_RELOAD_CONFIG` is sent, and the daemon keeps the shortcuts it already has.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| Chord has no modifier | Inline error naming the field and what it needs | `debug!` validation reason |
| Chord has several main keys | Inline error naming the field and what it needs | `debug!` validation reason |

## Why it is not in the UC

Keeps the rejection grammar and the listening-state rules out of the success path, which is a straight line from activate to save.

## Notes

Save is not gated on validity: the button is always live (`crates/settings/src/main.rs`), and an invalid draft is refused at save time by `validate_config`, surfacing as `SaveFeedback::Error`. A design that disabled Save would have to explain which field disabled it, which is why rejection is reported per field instead.
