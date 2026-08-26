---
id: "S2"
status: done
component: window-management
commit: c44a0d5
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S2 - Snap the active window to the top or bottom half

## What landed

`split_y`, `plan_snap_top`, and `plan_snap_bottom` in `arrangement/snap.rs`, mirroring `split_x` exactly - one boundary computed once, both halves derived from it. Wire values 6 and 7. Two config fields defaulting to `ctrl+alt+up` and `ctrl+alt+down`. Hook and worker arms. `command_set_contains_no_inter_monitor_arrangement` replaced by `command_set_is_complete_over_its_whole_range`, which keeps the completeness property and drops only the withdrawn delegation.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 400 tests, +9. Includes the tiling property on both axes, deterministic odd-extent division asserted stable over repeats, negative origins, and the one-pixel degenerate case.

## Not done, and why

The four pre-existing snap defaults were left on `Ctrl+Win` in this story; moving the family is story 5, and mixing the two would have made both diffs unreadable.
