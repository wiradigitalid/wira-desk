---
id: "S1"
status: done
component: window-management
commit: cd647c7
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S1 - Chord configuration travels as one struct

## What landed

`Chords` in `crates/daemon/src/hook.rs`, holding each chord as `Option<Shortcut>` where `None` means unbound. Replaces a six-tuple return from `load_shortcuts`, six positional parameters into `match_shortcut`, six `HookRuntime` fields, and six assignment statements in the config-snapshot arm. `Chords::in_declared_order()` is the single declared sequence and `match_shortcut` is a walk over it. `HookSnapshot` collapsed to one field. `load_shortcuts`' six near-identical blocks collapsed to one row per chord plus a shared `resolve_one`.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 391 tests, +2. Every pre-existing hook test kept its name and its assertions, which was the acceptance question for a pure refactor.

## Not done, and why

No new chord, config field, or command value, by design - those are stories 2 to 5. The story note's original justification (that `clippy::too_many_arguments` would fail the build at nine chords) was wrong and was corrected in `waves.yaml` and `stories.yaml`: `match_shortcut` already carried an explicit `allow`, so the refactor rests on the parallel-lists argument instead.
