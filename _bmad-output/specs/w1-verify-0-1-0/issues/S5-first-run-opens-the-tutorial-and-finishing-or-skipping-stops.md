---
id: S5
epic: E2
wave: W1
component: settings
satisfies: [UC-5]
depends_on: [S4]
status: done
closed_by: "0.1.0"
---

# S5 — First run opens the tutorial, and finishing or skipping stops it returning

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-5` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `app::tests::onboarding_is_absent_unless_requested`
- `app::tests::onboarding_starts_at_welcome_and_advances_to_done`
- `app::tests::every_onboarding_step_has_heading_and_body`
- `app::tests::onboarding_teaches_the_spatial_philosophy`
- `app::tests::skip_reaches_the_same_terminal_state_as_completing`
- `persistence::tests::missing_config_selects_onboarding`
- `persistence::tests::existing_config_selects_settings`
- `persistence::tests::explicit_flag_forces_onboarding`
- `persistence::tests::onboarding_flag_is_the_frozen_spelling`
- `persistence::tests::completing_onboarding_writes_a_valid_config_so_it_does_not_repeat`

