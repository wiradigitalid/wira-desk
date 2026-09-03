---
type: uc
id: UC-4
component: settings
satisfies: [FR-7, FR-18]
critical: false
created: '2026-08-21'
updated: '2026-08-26'
---

# UC-4 — Change a keyboard shortcut in Settings

## Trigger

User selects a shortcut field in Settings to customize its key combination.

## Precondition

- Settings application (`wiradesk-settings.exe`) is running and displaying the Shortcuts pane, which lists every editable action as exactly one row (LBR-ST-14).
- Background daemon is active or configuration file exists in `%APPDATA%\WiraDesk\config.toml`.

## Main Flow

1. User focuses a shortcut input field to change its key binding.
2. System transitions the field into listening mode and announces the capture state to assistive technologies.
3. User presses the new physical key combination containing at least one modifier.
4. System validates chord syntax, displays the canonical combination, and marks the draft configuration dirty.
5. User selects the Save action to commit changes.
6. System writes the validated configuration atomically to disk and posts `WM_APP_RELOAD_CONFIG` to the daemon.
7. System displays confirmation feedback and updates the saved baseline state.
8. User closes Settings and immediately uses the new shortcut for the action whose row they edited.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | User presses Escape while in listening mode | System cancels capture mode, restores the previous shortcut value, and leaves the draft unchanged. |
| Step 4 | User selects Revert instead of Save | System discards draft modifications, restores previously saved values, and clears dirty status. |
| Step 6 | Daemon is not running | System saves configuration atomically to disk without IPC reload; daemon will load updated settings on next startup. |
| Step 2 | The Shortcuts pane is visible and Settings holds the foreground window | The daemon withholds its own shortcut actions and reports each observed chord to Settings, so a chord pressed to test it neither switches windows nor is taken from Windows (LBR-ST-11). |
| Step 3 | Field is listening and Settings holds the foreground window | The daemon additionally withholds the chord from Windows for the duration of the recording, so the shell cannot act on it and steal the foreground before the capture completes (LBR-ST-11). |
| Step 3 | The captured chord is `Win + Ctrl + Left` or `Win + Ctrl + Right` | System refuses it and names what Windows uses it for — switching between virtual desktops. These two entered the reserved catalogue with `DEC-008`; before that they were accepted, and were the shipped snap defaults. |
| Step 3, when the just-captured chord displaces another field's existing chord | User offers Swap on the displaced field | System gives the displaced field back the chord the capture just took from it, restoring both fields to what they held before the capture collided them. Only the field the capture actually displaced can be swapped this way — offered on any other field it is a no-op — and the draft is not re-checked for a new collision after the swap; the pane re-evaluates on the next frame regardless. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 3 | User presses a bare key without modifier keys | System rejects the combination, retains listening mode, and displays an inline validation message (SCN-01) | Field remains in listening mode with previous valid shortcut intact in draft |
| Step 3 | User presses an unrecognized or unsupported key token | System rejects the input with an unsupported token notice and continues listening | Draft and disk configuration remain unchanged; user can re-strike a valid chord |
| Step 3 | User presses a chord the Windows shell already owns (`Win+1`, `Win+E`, `Win+D`, `Win+V`, `Win+Shift+S`, …) | System refuses the chord, names the Windows function it belongs to, and — where the chord is one Wira Desk could technically have taken — offers a way through; listening continues (SCN-01, LBR-ST-10) | Draft unchanged; the user knows which Windows function the chord serves and what to press instead |
| Step 3 | The daemon reports a chord the canonical grammar cannot name (a virtual-key code with no shared name, e.g. `Win+Semicolon`) | System refuses the chord explicitly rather than silently ignoring it, and continues listening | Draft unchanged; the field does not appear unresponsive |
| Step 4 | The captured chord is already held by another configurable action | System keeps the chord in the draft, marks both actions — each naming the other — and refuses the draft on submission (SCN-03) | Both offending actions are identified and the submit action stays available; nothing is written until the collision is resolved |
| Step 6 | Atomic file write fails due to filesystem permission error | System displays save failure error message and skips IPC reload | Draft remains in memory for correction; existing configuration on disk remains intact |

## Outcome

The new shortcut binding is persisted atomically to disk and immediately active in the running daemon without requiring a daemon restart or system reboot.

## Business Rules

- `BR-1`
- `BR-2`
- `LBR-ST-1`
- `LBR-ST-2`
- `LBR-ST-5`
- `LBR-ST-6`
- `LBR-ST-8`
- `LBR-ST-9`
- `LBR-ST-10`
- `LBR-ST-11`
- `LBR-ST-12`
