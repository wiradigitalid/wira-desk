---
id: S3
epic: E1
wave: W1
component: window-management
satisfies: [UC-3]
depends_on: [S1]
status: done
closed_by: "0.1.0"
---

# S3 — Tray health escalates to Tier 3 once and recovers without a restart

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-3` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `tray::tests::next_hook_check_state_escalates_after_threshold`
- `tray::tests::next_hook_check_state_keeps_escalating_while_still_failing`
- `tray::tests::next_hook_check_state_resets_on_success`
- `tray::tests::state_after_recovery_restores_latched_warning`
- `menu::tests::settings_exe_matches_cargo_bin_name`
- `log::tests::format_timestamp_pads_single_digit_fields`

