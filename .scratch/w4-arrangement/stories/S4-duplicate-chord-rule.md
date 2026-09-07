---
id: "S4"
status: done
component: window-management
commit: 93bf8a5
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S4 - A duplicate chord unbinds the later action, or refuses the reload

## What landed

`unbind_duplicates` in `hook.rs` - pure, slice-taking, first-wins - called from `load_shortcuts`, which warns once per unreachable action naming both fields and the chord. `RejectReason::DuplicateShortcut` in `daemon/config.rs`, refusing the whole candidate inside the existing all-or-nothing contract. Both call sites name `DEC-009` in a comment, because the justification for two behaviours lives nowhere near the code.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 425 tests, +8. **The guard was seen failing**: breaking it turned exactly the five tests that assert it red (two on the reload path, three on the resolver) and restoring it turned them green, run with `--no-fail-fast` so the whole suite reported. One mistake worth recording: the first reading after restoring said still red, and that was wrong - `Copy-Item` preserved the backup's timestamp, so cargo served the broken build.

## Not done, and why

The unbound state has no representation in the Settings UI. `DEC-009` accepts that silence and names showing it as the route out; this wave does not build it. A three-way collision reports one loser per unreachable action rather than one for the group (`OQ-4`).
