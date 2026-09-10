---
type: uc
id: UC-14
component: settings
satisfies: [FR-32]
critical: false
created: '2026-09-10'
---

# UC-14 — Configure mouse navigation actions and presets in Settings

## Trigger

User opens the Settings window and selects the "Mouse" tab in the sidebar.

## Precondition

- Wira Desk daemon is running elevated.
- Settings process (`wiradesk-settings.exe`) has launched and loaded `%APPDATA%\WiraDesk\config.toml`.

## Main Flow

1. User navigates to the Mouse tab in the Settings sidebar.
2. System displays the Mouse configuration pane featuring:
   - A master toggle: "Enable Mouse Navigation" (On/Off).
   - Four input configuration rows: Thumb Button 1 (Back), Thumb Button 2 (Forward), Tilt Wheel Left, Tilt Wheel Right.
   - Each row displays a dropdown selector containing curated action presets (e.g. Next Virtual Desktop, Previous Virtual Desktop, Task View, Show Desktop, Cycle Same-App Window Forward/Backward, Snap Window Left/Right, or Default/Passthrough).
3. User toggles the master switch or changes the dropdown selection for one or more inputs.
4. System marks the settings state as dirty and enables the Save button.
5. User clicks Save (or presses Ctrl + S).
6. System serializes the configuration, writes `config.toml.tmp` atomically, and renames it over `%APPDATA%\WiraDesk\config.toml` (`BR-2`, `LBR-ST-2`).
7. System dispatches `WM_APP_RELOAD_CONFIG` to the daemon hidden message window (`WiraDeskDaemonHiddenWindow`).
8. System displays confirmation feedback, clears the dirty indicator, and the daemon reloads and activates the new mouse bindings immediately.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 2 | Configuration file did not yet contain a `[mouse]` section (legacy config) | System populates the UI with default values (`enabled = true`, default recommended presets), without modifying the file until the user saves. |
| Step 3 | User sets an input to "Default / Passthrough" | That physical mouse button or tilt direction is marked unmapped; upon save, the daemon will pass that event through to Windows/active applications via `CallNextHookEx`. |
| Step 4 | User switches sidebar tabs or closes Settings without saving | System retains unsaved working draft until user confirms discard or applies changes. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 6 | File write error or permissions refusal on `%APPDATA%\WiraDesk\` | System retains the working draft in memory and displays an inline error alert | Working copy remains intact in GUI; user can re-try saving |
| Step 7 | Daemon hidden window cannot be resolved | System reports a reload warning | Saved configuration is on disk; daemon will load it upon its next restart |

## Outcome

The user easily configures mouse navigation actions from an intuitive dropdown UI without needing to record complex keyboard macros or install bloated vendor utilities.

## Business Rules

- `BR-1` (Explicit IPC Configuration Reload)
- `BR-2` (Canonical Shared Configuration Schema)
- `BR-10` (Mouse Navigation Action Mapping and Passthrough)
- `LBR-ST-2` (Atomic configuration persistence)
- `LBR-ST-5` (Deterministic Tab navigation order)
- `LBR-ST-18` (Curated mouse action presets)
