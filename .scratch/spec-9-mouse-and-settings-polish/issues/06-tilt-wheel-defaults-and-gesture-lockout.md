---
id: SPEC-9-06
component: daemon
satisfies: [FR-30, FR-31]
blocked_by: []
status: ready-for-agent
tests:
  - hook::tests::tilt_hold_without_release_fires_exactly_once
  - hook::tests::tilt_second_actuation_after_quiet_period_fires_again
  - config::tests::default_tilt_directions_are_inverted
---

# 06: Tilt wheel default inversion and sustained hold gesture lockout

**What to build:** Invert the default tilt wheel mapping in `crates/shared/src/config.rs` (`tilt_left = "show_desktop"` and `tilt_right = "task_view"`). Upgrade the tilt-wheel filter in `crates/daemon/src/hook.rs` from simple inter-arrival debounce to a two-phase state machine: Phase A (150 ms bounce filter) plus Phase B (gesture lockout). Sustained holding of the tilt wheel will execute the assigned action exactly once; all repeating `WM_MOUSEHWHEEL` messages while held are swallowed until a 400 ms quiet gap occurs.

**Blocked by:** None

## Acceptance Criteria

- [ ] In `crates/shared/src/config.rs::MouseConfig::default()`:
      - `tilt_left` defaults to `"show_desktop"`.
      - `tilt_right` defaults to `"task_view"`.
- [ ] In `crates/daemon/src/hook.rs`:
      - `HookRuntime` tracks:
        - `last_tilt_ms: u64` (timestamp of last accepted command).
        - `last_tilt_event_ms: u64` (timestamp of most recent `WM_MOUSEHWHEEL` tick, accepted or swallowed).
        - `tilt_gesture_armed: bool` (armed when a tilt command is accepted; disarmed after quiet period).
      - Constant defined: `TILT_QUIET_MS = 400`.
      - When `WM_MOUSEHWHEEL` arrives:
        - Update `rt.last_tilt_event_ms = now`.
        - If `rt.tilt_gesture_armed`:
          - If `now.saturating_sub(rt.last_tilt_event_ms_previous) >= TILT_QUIET_MS`: disarm `rt.tilt_gesture_armed = false`.
          - If still within quiet window: swallow message (`return 1`) without enqueuing duplicate commands.
        - If not armed:
          - Apply Phase A bounce filter (`now.saturating_sub(rt.last_tilt_ms) >= 150 ms`).
          - Enqueue command to ring buffer, update `rt.last_tilt_ms = now`, and set `rt.tilt_gesture_armed = true`.
- [ ] Unit tests in `hook.rs`:
      - `tilt_hold_without_release_fires_exactly_once`: simulates a rapid stream of 20 `WM_MOUSEHWHEEL` ticks spaced 50–100 ms apart (holding the wheel); asserts exactly 1 command is enqueued and 19 are swallowed.
      - `tilt_second_actuation_after_quiet_period_fires_again`: simulates hold ticks, pauses 450 ms, then delivers a new tick; asserts a second command is enqueued.
      - `default_tilt_directions_are_inverted`: verifies `MouseConfig::default()` has `tilt_left == "show_desktop"` and `tilt_right == "task_view"`.
- [ ] `.what/window-management/05-scenarios/SCN-04-tilt-wheel-rapid-flick-debounced.md` updated and validated against the new state machine.
