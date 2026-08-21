# SCN-01 — Invalid shortcut rejected before save

**Parent UC:** UC-4  
**Actor:** Power User  
**Trigger:** User captures a chord without a required modifier key.

## Preconditions

- Settings is open on the Shortcuts tab.
- A shortcut field is focused and `LC-shortcut-capturer` is in `Listening` state.

## Steps

1. User clicks Listen on the cycling primary field.
2. User presses a single letter key (e.g. `A`) without Win, Ctrl, or Alt held.
3. `validate_shortcut` returns `NoModifier`.
4. UI shows inline error; the field remains in Listening or returns to Idle without updating the draft chord.
5. Save button stays disabled for the invalid field.
6. User dismisses error or presses Escape.

## Postcondition

- Prior `config.toml` on disk is unchanged.
- No `WM_APP_RELOAD_CONFIG` is sent; daemon keeps last valid shortcuts.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| Validation error | Inline red helper text | `debug!` validation reason |
