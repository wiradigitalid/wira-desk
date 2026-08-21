---
type: uc
id: UC-6
component: settings
satisfies: [FR-13]
critical: false
created: '2026-08-21'
---

# UC-6 — Turn auto-start on boot on or off

## Trigger

User toggles the Auto-Start setting in Settings or selects Auto-Start from the system tray context menu.

## Precondition

- Wira Desk is installed and running under the active Windows user account.
- User has permissions to configure scheduled tasks for their user session.

## Main Flow

1. User toggles the Auto-Start option in Settings or the tray context menu.
2. System creates an `ONLOGON` Windows Scheduled Task for the current user with highest privileges and absolute daemon path.
3. User saves settings or confirms the toggle state change.
4. System persists the updated `auto_start` boolean preference in `config.toml`.
5. User restarts Windows or signs out and logs back in.
6. System launches the background daemon automatically at logon elevated with highest privileges without a UAC prompt.
7. User toggles the Auto-Start option off at a later time.
8. System deletes the scheduled task from Windows Task Scheduler and updates the persisted configuration.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 1 | User toggles Auto-Start directly from the tray context menu | System creates or deletes the scheduled task immediately and persists the new setting to `config.toml`. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 2 | Task Scheduler creation fails due to Group Policy or permissions | System reverts the toggle to off, logs a Tier-2 entry, and updates tray warning state without a modal popup (SCN-02) | Toggle remains off and current task configuration is unchanged |
| Step 8 | Task deletion fails because task was already removed externally | System logs warning, updates persisted preference to disabled, and clears toggle state | Auto-start remains disabled without raising modal errors |

## Outcome

Windows Task Scheduler is configured to launch Wira Desk silently at user logon with highest privileges, or the scheduled task is removed when auto-start is disabled.

## Business Rules

- `BR-1`
- `BR-4`
- `LBR-ST-4`
