---
id: S1
epic: E1
wave: W1
component: window-management
satisfies: [UC-1]
depends_on: []
status: done
closed_by: "0.1.0"
---

# S1 — Cycling reaches every same-app window on this monitor, in order

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-1` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `cycling::tests::activates_next_window_of_same_application`
- `cycling::tests::same_executable_different_pid_is_same_application`
- `cycling::tests::multi_process_same_executable_groups_together`
- `cycling::tests::excluded_windows_are_skipped`
- `cycling::tests::invalid_target_continues_to_next_candidate`
- `cycling::selection::tests::wraps_to_beginning_from_last_window`
- `cycling::selection::tests::wraps_at_most_once`
- `cycling::selection::tests::active_absent_order_covers_every_eligible_window_once`
- `cycling::selection::tests::closing_window_mid_cycle_falls_through_to_next`
- `cycling::eligibility::tests::synthetic_hung_application_window_remains_eligible`
- `cycling::eligibility::tests::minimized_is_excluded`
- `cycling::eligibility::tests::ghost_outranks_every_other_exclusion`
- `cycling::eligibility::tests::same_executable_across_processes_stays_eligible`
- `context::tests::different_monitor_is_rejected`
- `context::tests::non_current_virtual_desktop_is_rejected`
- `context::tests::unknown_virtual_desktop_fails_closed`
- `context::vm_bypass::tests::configured_process_passes_through`
- `context::vm_bypass::tests::configured_class_passes_through_when_process_does_not_match`
- `context::vm_bypass::tests::fully_unresolved_fails_open`
- `hook::tests::exact_primary_match`
- `hook::tests::extra_modifier_is_non_match`
- `hook::tests::throttle_boundary`

