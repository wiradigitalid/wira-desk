---
type: uc
id: UC-4
component: settings
satisfies: [FR-7, FR-18]
critical: false
created: '2026-08-21'
---

# UC-4 — Change a keyboard shortcut in Settings

## Trigger

User selects a shortcut field in Settings to customize its key combination.

## Precondition

- Settings application (`wiradesk-settings.exe`) is running and displaying the Shortcuts pane.
- Background daemon is active or configuration file exists in `%APPDATA%\WiraDesk\config.toml`.

## Main Flow

1. User focuses a shortcut input field to change its key binding.
2. System transitions the field into listening mode and announces the capture state to assistive technologies.
3. User presses the new physical key combination containing at least one modifier.
4. System validates chord syntax, displays the canonical combination, and marks the draft configuration dirty.
5. User selects the Save action to commit changes.
6. System writes the validated configuration atomically to disk and posts `WM_APP_RELOAD_CONFIG` to the daemon.
7. System displays confirmation feedback and updates the saved baseline state.
8. User closes Settings and immediately uses the new shortcut for window cycling.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | User presses Escape while in listening mode | System cancels capture mode, restores the previous shortcut value, and leaves the draft unchanged. |
| Step 4 | User selects Revert instead of Save | System discards draft modifications, restores previously saved values, and clears dirty status. |
| Step 6 | Daemon is not running | System saves configuration atomically to disk without IPC reload; daemon will load updated settings on next startup. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 3 | User presses a bare key without modifier keys | System rejects the combination, retains listening mode, and displays an inline validation message (SCN-01) | Field remains in listening mode with previous valid shortcut intact in draft |
| Step 3 | User presses an unrecognized or unsupported key token | System rejects the input with an unsupported token notice and continues listening | Draft and disk configuration remain unchanged; user can re-strike a valid chord |
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
