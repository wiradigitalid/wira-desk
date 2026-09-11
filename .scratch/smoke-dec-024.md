# Smoke test — mandate `DEC-024` (SPEC-10 Mouse Navigation Follow-up & SPEC-11 App Distribution and Branding)

Date: 2026-09-11
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | FR-32 / UC-14 (Preset Label Standardization) | `MouseActionPreset::SnapThirdCenter` display label standardized to "Snap to middle third" | Display label matches Shortcuts pane exactly; verified by `snap_middle_third_display_label_matches_shortcuts_pane` and `mouse_preset_dropdown_displays_snap_to_middle_third` | PASS |
| 2 | FR-32 / UC-14 (Dropdown Tab Switch Dismissal) | Switching sidebar tabs while dropdown overlay is open immediately dismisses overlay | Overlay resets `dropdown_open = false` on tab transition; verified by `switching_panes_while_dropdown_open_automatically_dismisses_overlay` | PASS |
| 3 | FR-32 / UC-14 (Sidebar Outside Click Dismissal) | Clicking outside content on sidebar background dismisses dropdown overlay | Background TouchArea captures clicks outside buttons and closes overlay; verified by `clicking_sidebar_outside_content_dismisses_preset_overlay` | PASS |
| 4 | FR-30 / UC-13 (Daemon Config Reload Parity) | Daemon config reload validation accepts all 20 mouse action presets without rejection | `validate()` accepts all 20 presets on all 4 buttons; verified by `reload_with_all_20_mouse_action_presets_is_accepted` | PASS |
| 5 | FR-30 / UC-13 (Preset Dispatch Execution) | All mouse action presets map to concrete wire commands with active arrangement planning | Commands dispatch to planning/window movement cleanly; verified by `snap_commands_from_mouse_dispatch_to_planning` and `all_mouse_action_presets_have_corresponding_command_execution` | PASS |
| 6 | FR-30 / UC-13 (Process Lifecycle Helper) | `scripts/restart-daemon.ps1` terminates stale daemon instances and polls message window | Script terminates stale PIDs, launches elevated binary, and polls `WiraDeskDaemonHiddenWindow` | PASS |
| 7 | FR-30 / FR-31 / UC-13 (Tilt Hardware Repeat Lockout) | Sustained hold of physical tilt wheel swallows repeated hardware ticks across 2.0s hold | 20 consecutive ticks at 100ms intervals enqueue exactly 1 command and swallow 100% of repeats; verified by `tilt_hold_with_rapid_hardware_repeats_swallows_subsequent_ticks` | PASS |
| 8 | FR-30 / FR-31 / UC-13 (Tilt Quiet Period Boundary) | 400ms quiet period boundary: repeat at 399ms is swallowed, actuation at >=400ms fires | Boundary asserted; verified by `tilt_quiet_period_boundary_at_399ms_and_400ms` and `tilt_hold_alternating_directions_maintains_lockout_per_gesture` | PASS |
| 9 | Distribution (Release Binary Scanner) | `scripts/verify-release-binary.ps1` scans PE import tables and verifies runtime bundling | Script inspects PE headers; confirmed clean static daemon and bundled VC++ runtime fallback for settings; verified by unit tests in `crates/shared/src/binary.rs` | PASS |
| 10 | Distribution (About Pane Branding & Navigation) | About pane displays publisher attribution, GitHub link, GPL-3.0 license, and support navigation | Strict HTTPS allowlist in `is_allowed_browser_url`; verified by `about_pane_renders_publisher_and_links` and `open_in_browser_accepts_publisher_and_repo_domains` | PASS |
| 11 | Security (Transparent Network Disclosure) | `SECURITY.md`, `docs/threat-model.md`, and `README.md` accurately disclose updater network boundary | False "no network path" statements corrected to honest disclosure with zero telemetry; verified by `scripts/verify-public-export.ps1` | PASS |
| 12 | General (Workspace Hygiene & Quality) | Workspace clean on formatting, lints, and automated tests | 620+ tests pass (0 fail), `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `validate.py --generate` GREEN | PASS |

12 PASS, 0 FAIL, 0 NOT VERIFIABLE.
All open specifications (SPEC-10 and SPEC-11) confirmed complete and verified.
