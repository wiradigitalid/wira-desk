---
type: uc
id: UC-5
component: settings
satisfies: [FR-17]
critical: false
created: '2026-08-21'
---

# UC-5 — Complete or skip the first-run tutorial

## Trigger

User launches Wira Desk for the first time without an existing configuration file, or launches Settings with the `--onboarding` flag.

## Precondition

- Wira Desk is launched on Windows desktop.
- No existing `%APPDATA%\WiraDesk\config.toml` file exists, or the executable is invoked with `--onboarding`.

## Main Flow

1. User starts Wira Desk or triggers onboarding.
2. System detects first-run intent and presents the interactive onboarding window at the Welcome step.
3. User reads the spatial cycling explanation and advances to the interactive practice step.
4. System presents the practice exercise explaining same-application cycling on the current monitor.
5. User presses the demonstrated cycling shortcut to complete the practice step.
6. System displays the completion step confirming setup is complete.
7. User selects the finish action to conclude onboarding.
8. System writes the baseline configuration to disk and transitions to normal background tray operation.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | User selects Skip Tutorial on the welcome step | System skips remaining tutorial steps, writes initial default configuration to disk, and transitions to tray operation. |
| Step 4 | User selects Skip Tutorial during practice step | System bypasses remainder of tutorial, writes valid default configuration to disk, and transitions to tray operation. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 8 | Configuration persistence fails due to disk write error | System displays an error message and keeps onboarding open | User is alerted to filesystem issue; onboarding will re-prompt on next launch until saved |

## Outcome

User understands same-application spatial cycling concepts, a valid configuration file is persisted to disk, and first-run onboarding will not appear on subsequent launches.

## Business Rules

- `BR-2`
- `BR-3`
- `LBR-ST-3`
- `LBR-ST-7`
