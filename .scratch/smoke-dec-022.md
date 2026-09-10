# Smoke test — mandate `DEC-022` (SPEC-8 Driverless Mouse Desktop Navigation)

Date: 2026-09-10
Head SHA: pending
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | FR-32 (Mouse Settings Pane) | Settings shell expands from 4 to 5 panes with dedicated Mouse navigation pane | Mouse pane declared at index 2 (General=0, Shortcuts=1, Mouse=2, VM=3, About=4); verified by `five_pane_focus_order_includes_mouse` and `pane_enum_no_longer_declares_layout` | PASS |
| 2 | FR-32 (Master Enable Toggle) | Master toggle enables/disables mouse auxiliary navigation in draft | Toggle reflects draft state, marks dirty when modified, and restores on revert; verified by `toggling_mouse_navigation_updates_draft` | PASS |
| 3 | FR-32 (Preset Action Selectors) | 4 physical controls (Thumb 1/2, Tilt Left/Right) render curated action preset selectors | Preset dropdown selectors load configured actions and parse slugs reliably; verified by `mouse_pane_loads_configured_presets` and `mouse_action_preset_slug_parsing` | PASS |
| 4 | FR-32 (Persistence & Validation) | Invalid preset strings in configuration are rejected prior to saving | Malformed presets rejected cleanly with `ShortcutError::InvalidMousePreset`; verified by `invalid_mouse_preset_string_is_rejected` and `mouse_preferences_save_and_reload_signal` | PASS |
| 5 | FR-30 (WH_MOUSE_LL Fast Path) | Mouse movement passes through with zero latency, locks, or allocations | `WM_MOUSEMOVE` bypasses hook logic directly; verified by `mouse_hook_passes_mousemove_without_interception` | PASS |
| 6 | FR-30 (Thumb Button Interception) | Mapped thumb buttons (XBUTTON1/XBUTTON2) down and up are swallowed, enqueuing discrete commands | Mapped clicks swallow down and up, enqueuing navigation commands to ring; verified by `mouse_hook_swallows_mapped_xbuttons_down_and_up` | PASS |
| 7 | FR-30 (Unmapped Passthrough) | Unmapped mouse buttons pass through to Windows transparently | Unmapped buttons return `PassToNext`; verified by `unmapped_mouse_buttons_pass_through` | PASS |
| 8 | FR-31 (Tilt Wheel Debounce) | Horizontal tilt wheel debounced at 150 ms (SCN-04) | Rapid burst ticks within 150 ms swallowed without enqueue; satisfying ticks enqueued; verified by `tilt_wheel_debounce_drops_rapid_burst_ticks_at_150ms` and `tilt_wheel_satisfying_debounce_enqueues_command` | PASS |
| 9 | FR-30/31 (VM/RDP Passthrough) | Foreground VM or RDP session receives mouse inputs unintercepted | Bypass policy causes mouse events to return `PassToNext`; verified by `vm_bypass_passes_mouse_events_through` | PASS |
| 10 | FR-30/31 (Worker Chord Synthesis) | Worker actor synthesizes virtual desktop chords via SendInput and suppresses Start menu | Navigation commands dispatch through `SendInput` with `suppress_start_menu()`; verified by `mouse_virtual_desktop_commands_dispatch_safely` | PASS |
| 11 | General (Ring Capacity & Metrics) | Ring buffer full increments dropped metric without blocking hook thread | Dropped metric incremented cleanly on full ring; verified by `ring_full_increments_dropped_metric_on_mouse_event` | PASS |
| 12 | General (Production artifacts) | Production release build produces valid binaries | `build.ps1 -Mode prod` succeeded (`wiradesk.exe`, `wiradesk-settings.exe`) | PASS |
| 13 | General (Codebase hygiene) | Workspace passes all lints, tests, and export hygiene | 591 tests pass (0 fail), clippy clean, fmt clean, export hygiene passes 10/10 checks | PASS |

13 PASS, 0 FAIL, 0 NOT VERIFIABLE.
SPEC-8 driverless mouse desktop navigation confirmed complete and verified.
