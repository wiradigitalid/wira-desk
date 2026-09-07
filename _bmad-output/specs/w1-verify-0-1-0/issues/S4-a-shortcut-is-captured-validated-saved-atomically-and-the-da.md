---
id: S4
epic: E2
wave: W1
component: settings
satisfies: [UC-4]
depends_on: []
status: done
closed_by: "0.1.0"
---

# S4 — A shortcut is captured, validated, saved atomically, and the daemon signalled

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-4` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `app::tests::capture_starts_idle`
- `app::tests::beginning_capture_targets_exactly_one_field`
- `app::tests::accepting_a_valid_capture_stores_the_canonical_form`
- `app::tests::accepting_without_listening_is_a_no_op`
- `app::tests::cancelling_capture_changes_nothing`
- `app::tests::editing_the_draft_marks_dirty_without_touching_saved`
- `app::tests::a_successful_save_promotes_the_draft_and_clears_dirty`
- `app::tests::a_rejected_save_reports_an_error_and_does_not_promote_the_draft`
- `persistence::tests::valid_shortcut_returns_canonical_form`
- `persistence::tests::bare_main_key_without_a_modifier_is_rejected`
- `persistence::tests::modifier_only_is_reported_as_no_main_key`
- `persistence::tests::two_main_keys_are_reported_distinctly`
- `persistence::tests::unsupported_token_is_reported_as_such`
- `persistence::tests::an_invalid_field_names_itself`
- `persistence::tests::rejection_leaves_the_previous_file_intact`
- `persistence::tests::saved_config_round_trips_without_loss`
- `persistence::tests::default_config_uses_frozen_shortcuts`
- `persistence::tests::reload_uses_the_frozen_message_identifier`
- `persistence::tests::reload_signal_is_harmless_when_no_daemon_is_running`

