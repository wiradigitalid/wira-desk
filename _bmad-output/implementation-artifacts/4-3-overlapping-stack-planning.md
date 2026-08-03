---
baseline_commit: dd91900
workflow_id: story-4-3-overlapping-stack-planning
---

# Story 4.3: Overlapping-Stack Planning

Status: review

## Story

As a user with a small monitor,
I want up to three related windows arranged in a clickable overlapping stack,
so that I can reach each window directly without using Alt+Tab.

## Acceptance Criteria

### AC-4.3-001 — Disabled is a successful no-op
A disabled stack returns success and produces no placement.

### AC-4.3-002 — Order and cap
Accepted live order is preserved; no more than the first three candidates are selected; zero candidates is a successful no-op.

### AC-4.3-003 — Default anchoring
At the default width each window uses 50 percent of the work-area width; one window centred; two anchored left and right; three anchored left, centre, and right; every window retains a visible clickable horizontal edge.

### AC-4.3-004 — Width policy
Anchors are distributed deterministically across the available travel range; all rectangles remain inside the monitor work area; zero or greater-than-100 percent produces a deterministic policy error.

### AC-4.3-005 — Fixture coverage
Odd dimensions, negative origins, and 96/120/144/192 DPI produce deterministic results for one-, two-, three-, and more-than-three candidate fixtures.

## Dev Notes

### Anchors Distribute Across *Travel*, Not Width

`travel = work_width - window_width`. Anchors are spread across that leftover
range: one window at `travel/2`, two at `0` and `travel`, three at `0`,
`travel/2`, `travel`. Distributing across travel rather than full width is what
keeps every rectangle inside the work area **by construction** — containment is
not a separate clamp that could be forgotten.

`all_rectangles_stay_inside_the_work_area` still asserts it across three work
areas × five width percentages × three counts, as a guard.

### Disabled Short-Circuits Before Validation

A disabled stack returns the no-op before the width check, so
`enable_overlapping_stack = false` with `stack_width_percent = 0` succeeds
rather than erroring. A user who has stack turned off should never see a policy
error from a setting that is not in use.

### Two Documented Degenerate Cases

**100 percent** is valid per the AC (only 0 and >100 are errors) but leaves zero
travel, so all three windows coincide and no clickable edge remains.
`full_width_degenerates_to_a_single_position` pins this rather than pretending
it cannot happen.

**Sub-pixel widths**: 1 percent of a 50 px work area rounds to zero pixels.
That returns `InvalidWidthPercent` rather than a zero-width placement.

### Intermediate Arithmetic Is Widened

`work_width * percent` is computed in `i64` before narrowing, so a large work
area at 100 percent cannot overflow `i32` mid-calculation. Anchor offsets use
the same widening.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 17 unit tests, all desktop-free; no `windows_sys` import.
- Static gates PASS in debug and release.
- **Planning logic independently executed** via a standalone `rustc` harness
  (135 checks shared with Stories 4.1/4.2, 0 failures), covering anchors,
  cap, no-ops, clickable edges, containment across 45 combinations, policy
  errors, the 100-percent degenerate case, negative origin, and determinism.
- `cargo test -p daemon` NOT executed — needs elevation.

### File List
- `crates/daemon/src/arrangement/stack.rs` (new)
