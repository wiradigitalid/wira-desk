---
id: SPEC-10-04
component: window-management
satisfies: [UC-13]
blocked_by: []
status: done
tests:
  - hook::tests::tilt_hold_with_rapid_hardware_repeats_swallows_subsequent_ticks
  - hook::tests::tilt_hold_alternating_directions_maintains_lockout_per_gesture
  - hook::tests::tilt_quiet_period_boundary_at_399ms_and_400ms
---

# 04: Synthetic multi-tick hardware tilt lockout verification test suite

**What to build:** Provide high-density synthetic test coverage for the two-phase tilt state machine in `crates/daemon/src/hook.rs`, simulating realistic hardware mouse drivers that generate rapid repeat ticks while tilted and held (at 30ms, 50ms, 100ms, and 200ms intervals). Prove that sustained hold dispatches exactly one action and swallows 100% of subsequent ticks until the physical quiet period (400 ms) has elapsed.

**Blocked by:** none

## Acceptance Criteria

- [ ] In `crates/daemon/src/hook.rs`:
      - Synthetic test simulates a user tilting and holding for 2.0 seconds with 20 consecutive `WM_MOUSEHWHEEL` events arriving at 100ms intervals: exactly 1 command is enqueued, and 19 events return `MouseHandleResult::Swallow`.
      - Boundary test validates that an event arriving at `399 ms` after the last hardware event is swallowed, while an event arriving at `>= 400 ms` disarms the lockout and triggers a new actuation.
      - Alternating tilt directions during a hold do not circumvent the lockout.
- [ ] Automated suite passes without regression.
