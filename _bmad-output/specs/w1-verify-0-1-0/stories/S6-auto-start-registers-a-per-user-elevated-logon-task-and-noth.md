---
id: S6
epic: E2
wave: W1
component: settings
satisfies: [UC-6]
depends_on: [S4]
status: done
closed_by: "0.1.0"
---

# S6 — Auto-start registers a per-user elevated logon task, and nothing else

## Why this story is `done` without a build step

Release `0.1.0` shipped the behaviour this story covers before the wave existed. The story is the trace, not the construction: it binds `UC-6` to the tests that already prove it, so the traceability row can close against evidence instead of against an assertion. No production code was written for it, and none should be — the build steps of the pipeline had nothing to do.

## Proving tests

Run with `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace`. Every name below was checked against `cargo test -- --list` when this file was written; a name here that no longer resolves is a finding, not a rename to absorb quietly.

- `autostart::tests::create_args_carries_logon_elevation_flags`
- `autostart::tests::create_args_wraps_exe_path_in_quotes`
- `autostart::tests::query_and_delete_target_the_pinned_task_name`

