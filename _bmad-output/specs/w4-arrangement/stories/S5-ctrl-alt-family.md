---
id: "S5"
status: done
component: window-management
commit: d1941ba
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S5 - Shipped chords move to Ctrl+Alt, and Win+Ctrl+Arrow is refused

## What landed

Every shipped arrangement default moved to the `Ctrl+Alt` family; `stack_shortcut` to `ctrl+alt+shift+down`. `Win+Ctrl+Left` and `Win+Ctrl+Right` added to `reservation()` as `ShellOwned`, owner text naming virtual-desktop switching. Four frozen tests updated rather than deleted. Two new guards: no shipped default may be a reserved chord, and every shipped default must be distinct.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 431 tests, +6. Two tests failed for the right reason (they held `win: true, ctrl: true` by hand while the default had become `ctrl+alt`) and were fixed at the root: modifier state and the shipped chord set are now both derived rather than restated, so a test cannot drift from the default it exercises.

## Not done, and why

No migration for existing `config.toml` files, deliberately (`DEC-008`). `ctrl+win+enter` still absent from the catalogue: its exclusion used to rest on being our own default, a premise this story removes, so it could now be catalogued - but that is a third entry nobody has decided on, and the source comment says so rather than still claiming the old reason.
