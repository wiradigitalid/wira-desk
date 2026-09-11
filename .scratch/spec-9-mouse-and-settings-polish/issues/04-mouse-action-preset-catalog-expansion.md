---
id: SPEC-9-04
component: settings
satisfies: [UC-14]
blocked_by: []
status: done
tests:
  - config::tests::expanded_mouse_presets_roundtrip_and_parse
  - hook::tests::all_expanded_presets_map_to_valid_commands
---

# 04: Mouse action preset catalog expansion to full window management suite

**What to build:** Expand `MouseActionPreset` in `crates/shared/src/config.rs` from 9 options to ~20 options, covering virtual desktops, Windows shell actions, window switching, all snapping variants (halves, thirds, custom %, maximize), overlapping stack, and moving to next monitor. Extend `map_preset_to_command` in `crates/daemon/src/hook.rs` to map all new presets to existing wire commands.

**Blocked by:** None

## Acceptance Criteria

- [ ] `shared::MouseActionPreset` enum expanded with canonical string slugs and display labels:
      - **Virtual Desktops:** `NextVirtualDesktop` (`"next_virtual_desktop"`), `PrevVirtualDesktop` (`"prev_virtual_desktop"`).
      - **Windows Shell:** `TaskView` (`"task_view"`), `ShowDesktop` (`"show_desktop"`).
      - **Window Switching:** `CycleForward` (`"cycle_forward"`).
      - **Snap — Halves:** `SnapLeft` (`"snap_left"`), `SnapRight` (`"snap_right"`), `SnapTop` (`"snap_top"`), `SnapBottom` (`"snap_bottom"`).
      - **Snap — Thirds:** `SnapThirdLeft` (`"snap_third_left"`), `SnapThirdCenter` (`"snap_third_center"`), `SnapThirdRight` (`"snap_third_right"`).
      - **Snap — Custom %:** `SnapPercentLeft` (`"snap_percent_left"`), `SnapPercentRight` (`"snap_percent_right"`), `SnapPercentTop` (`"snap_percent_top"`), `SnapPercentBottom` (`"snap_percent_bottom"`).
      - **Arrange & Move:** `Maximize` (`"maximize"`), `OverlappingStack` (`"overlapping_stack"`), `MoveNextMonitor` (`"move_next_monitor"`).
      - **Passthrough:** `Passthrough` (`"passthrough"`).
- [ ] Provide group categorization metadata in `shared` (e.g. `category(&self) -> &'static str`) for UI rendering:
      - "Virtual Desktops", "Windows Shell", "Switching", "Snap to Half", "Snap to Third", "Snap to Custom", "Arrange & Move", "Passthrough".
- [ ] Update `crates/daemon/src/hook.rs::map_preset_to_command`:
      - Maps `SnapTop` -> `Command::SnapTop` (opcode 6)
      - Maps `SnapBottom` -> `Command::SnapBottom` (opcode 7)
      - Maps `SnapThirdLeft` -> `Command::SnapThirdLeft` (opcode 13)
      - Maps `SnapThirdCenter` -> `Command::SnapThirdMiddle` (opcode 14)
      - Maps `SnapThirdRight` -> `Command::SnapThirdRight` (opcode 15)
      - Maps `SnapPercentLeft` -> `Command::SnapPercentLeft` (opcode 9)
      - Maps `SnapPercentRight` -> `Command::SnapPercentRight` (opcode 10)
      - Maps `SnapPercentTop` -> `Command::SnapPercentTop` (opcode 11)
      - Maps `SnapPercentBottom` -> `Command::SnapPercentBottom` (opcode 12)
      - Maps `OverlappingStack` -> `Command::OverlappingStack` (opcode 5)
      - Maps `MoveNextMonitor` -> `Command::MoveToNextMonitor` (opcode 8)
- [ ] Roundtrip serialization and pre-save validation tests verify all new presets parse cleanly and malformed slugs are rejected.
