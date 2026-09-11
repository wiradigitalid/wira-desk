# Smoke test — mandate `DEC-023` (SPEC-9 Mouse Navigation Polish, Preset Catalog Expansion & Settings UX Consistency)

Date: 2026-09-11
Head SHA: pending
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | FR-30 (Debug Trace Rotation) | `append_debug_trace` rotates at 1 MB cap with `.old` generation, bounded to ~2 MB | Capped at 1 MB, single `.old` backup, compiled out in prod; verified by `shared_log_rotation_caps_file_at_1mb` and `debug_trace_rotates_at_cap_when_debug_assertions_active` | PASS |
| 2 | FR-30 (AppData Files Inventory) | Full lifecycle of `%APPDATA%\WiraDesk` documented in `docs/README.md` and `CONTRIBUTING.md` | Inventory section added covering `config.toml`, `wiradesk.log`, and `wiradesk-debug-trace.log` | PASS |
| 3 | FR-32 (SettingToggleRow Centering) | Reusable `SettingToggleRow` centers `ToggleSwitch` mathematically against multi-line text | Clean vertical centering across General, Mouse, and About panes, removing `y: 4px` and `y: 3px` hacks; verified by `toggle_switches_are_vertically_centered_in_cards` | PASS |
| 4 | FR-32 (CardDivider Full-Bleed) | Multi-row cards standardize to 0px container padding with full-bleed `CardDivider` | Edge-to-edge dividers across General, Mouse, and About panes matching `shortcuts_pane` reference; verified by `card_dividers_render_full_bleed_across_panes` | PASS |
| 5 | FR-30/FR-32 (Preset Catalog Expansion) | `MouseActionPreset` expands from 9 to 20 window management actions across 8 categories | All 20 presets parse, serialize, validate, and map to wire `Command` opcodes; verified by `expanded_mouse_presets_roundtrip_and_parse` and `all_expanded_presets_map_to_valid_commands` | PASS |
| 6 | FR-32 (Preset Dropdown Overlay) | Clicking preset trigger opens categorized window-level dropdown at `z: 1000` | Floating overlay displays categories with hover highlighting; verified by `mouse_preset_dropdown_opens_overlay_with_groups` | PASS |
| 7 | FR-32 (Dropdown Selection & Dirty) | Selecting preset updates draft in-memory and marks model dirty to enable Save | In-memory draft updated and marked dirty upon selection; verified by `selecting_preset_from_dropdown_updates_draft` | PASS |
| 8 | FR-32 (Dropdown Dismissal) | Outside click on backdrop or Escape key dismisses overlay without mutating draft | Backdrop TouchArea and Escape handler dismiss overlay with draft untouched; verified by `mouse_preset_dropdown_dismisses_on_escape_without_mutating_draft` | PASS |
| 9 | FR-30/FR-31 (Tilt Wheel Inversion) | Default tilt wheel directions inverted (left=show_desktop, right=task_view) | `MouseConfig::default()` inverted; verified by `default_tilt_directions_are_inverted` | PASS |
| 10 | FR-30/FR-31 (Tilt Gesture Lockout) | Sustained hold of tilt wheel executes command exactly once without repeated spamming | Two-phase filter (150ms bounce + 400ms quiet-period lockout) swallows repeating ticks during hold; verified by `tilt_hold_without_release_fires_exactly_once` and `tilt_second_actuation_after_quiet_period_fires_again` | PASS |
| 11 | General (Codebase hygiene) | Workspace passes all formatting, clippy `-D warnings`, and test suite | 604 tests pass (0 fail), clippy clean, fmt clean, `validate.py --generate` GREEN with 0 findings | PASS |

11 PASS, 0 FAIL, 0 NOT VERIFIABLE.
SPEC-9 mouse navigation polish, preset catalog expansion, and settings UX consistency confirmed complete and verified.
