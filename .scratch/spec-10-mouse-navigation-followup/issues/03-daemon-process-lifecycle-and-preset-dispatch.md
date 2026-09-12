---
id: SPEC-10-03
component: window-management
satisfies: [UC-13]
blocked_by: []
status: done
tests:
  - config::tests::reload_with_all_20_mouse_action_presets_is_accepted
  - worker::tests::snap_commands_from_mouse_dispatch_to_planning
  - commands::tests::all_mouse_action_presets_have_corresponding_command_execution
---

# 03: Daemon process lifecycle, reload validation, and preset dispatch verification

**What to build:** Validate that the daemon configuration reload mechanism (`handle_reload_message` in `crates/daemon/src/config.rs`) accepts all 20 `MouseActionPreset` slugs without error, preventing `RejectReason::InvalidShortcut` errors in `wiradesk.log`. Provide clear process lifecycle guidance and tests to ensure developers and automation verify binary currency between Settings and Daemon. Confirm that every mouse snap preset correctly executes geometric arrangement when the active target is an external window.

**Blocked by:** none

## Acceptance Criteria

- [ ] In `crates/daemon/src/config.rs`:
      - `validate()` parses and accepts configuration containing any of the 20 `MouseActionPreset` variants.
      - Tested with pure unit tests: candidate text containing all 20 presets in `thumb_back`, `thumb_forward`, `tilt_left`, and `tilt_right` returns `Ok((cfg, hook, worker))`.
- [ ] In `crates/daemon/src/worker.rs`:
      - Every `Command` mapped from mouse presets (`SnapTop`, `SnapBottom`, `SnapThirdLeft`, `SnapThirdMiddle`, `SnapThirdRight`, `SnapPercentLeft`, `SnapPercentRight`, `SnapPercentTop`, `SnapPercentBottom`, `OverlappingStack`, `MoveToNextMonitor`) produces an arrangement plan and dispatches to `apply_or_report`.
      - Tested with synthetic worker execution verifying non-null dispatch when target window is external (`is_own_window == false`).
- [ ] Operational / Executable Daemon Lifecycle Helper:
      - Add an automated PowerShell lifecycle script `scripts/restart-daemon.ps1` that:
        1. Forcefully terminates any stale running `wiradesk.exe` instances (`Stop-Process -Name wiradesk -Force -ErrorAction SilentlyContinue`).
        2. Validates timestamp and launches the latest compiled binary (`target\debug\wiradesk.exe` or `target\release\wiradesk.exe`) with elevated privileges (`Start-Process -Verb RunAs`).
        3. Polls until the daemon hidden message window (`WiraDeskDaemonHiddenWindow`) is alive and ready to receive `WM_APP_RELOAD_CONFIG`.
      - Incorporate this restart step into the smoke test checklist and delivery verification so manual and automated tests never test against a stale daemon binary.
