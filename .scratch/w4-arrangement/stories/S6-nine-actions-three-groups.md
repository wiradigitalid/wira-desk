---
id: "S6"
status: done
component: settings
commit: ec863df
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S6 - Nine actions in three labelled groups, drawn from one sequence

## What landed

`ShortcutField` grown to nine with `group()`, `description()`, and `from_index()`. The Shortcuts pane rebuilt around three `ShortcutRowData` models built from `ShortcutField::ALL`, replacing 27 property declarations, 27 bindings, and 27 setters. `Pane::Layout` renamed from "Layout & Snapping", which had been describing a pane holding no snapping control. `validate_config`'s field table grown to nine, in the same order.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 438 tests, +7. `cargo build -p settings` clean. Guards include: no chord field in any pane but Shortcuts, and concatenating the pane groups reproduces the declared sequence exactly - which holds only while each group's members stay contiguous.

## Not done, and why

The pane was not driven in a real UI - no `testing-ui` run happened, because the wave ran without dispatched workers. The three-card layout, the group headings, and the focus order are verified by unit test and a clean Slint compile, not by a screenshot or an accessibility tree.
