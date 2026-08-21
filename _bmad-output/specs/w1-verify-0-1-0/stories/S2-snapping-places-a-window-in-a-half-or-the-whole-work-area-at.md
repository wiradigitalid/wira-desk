---
id: S2
epic: E1
wave: W1
component: window-management
satisfies: [UC-2]
depends_on: [S1]
status: done
closed_by: "0.1.0"
---

# S2 — Snapping places a window in a half or the whole work area, at any DPI

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-2` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `arrangement::snap::tests::snap_left_returns_left_half`
- `arrangement::snap::tests::snap_right_returns_complementary_half`
- `arrangement::snap::tests::halves_tile_the_work_area_without_gap_or_overlap`
- `arrangement::snap::tests::halves_stay_inside_the_work_area`
- `arrangement::snap::tests::maximize_never_exceeds_the_work_area`
- `arrangement::snap::tests::maximize_returns_the_whole_work_area`
- `arrangement::snap::tests::identical_geometry_yields_identical_plans_at_every_dpi`
- `arrangement::snap::tests::odd_width_is_split_deterministically`
- `arrangement::snap::tests::halves_work_at_negative_origin`
- `arrangement::snap::tests::inverted_work_area_fails_without_placement`
- `arrangement::snap::tests::empty_work_area_fails_without_placement`
- `arrangement::snap::tests::one_pixel_wide_work_area_still_splits_safely`
- `arrangement::tests::arrangement_shortcut_defaults_are_frozen`
- `arrangement::tests::command_set_contains_no_inter_monitor_arrangement`
- `arrangement::win32::tests::an_invalid_target_is_skipped_and_the_rest_continue`
- `arrangement::win32::tests::negative_monitor_coordinates_survive_conversion`

