---
baseline_commit: dd91900
workflow_id: story-4-2-dpi-aware-snap-planning
---

# Story 4.2: DPI-Aware Snap Planning

Status: review

## Story

As a desktop organizer,
I want the active window snapped precisely to the left half, right half, or usable full screen,
so that I can arrange my workspace instantly on monitors with different sizes and DPI settings.

## Acceptance Criteria

### AC-4.2-001 — Snap Left
Returns the left half of the work area; neither covers reserved work area nor leaves the monitor work area.

### AC-4.2-002 — Snap Right
Returns the complementary right half; an odd pixel width is divided deterministically without gap or overlap.

### AC-4.2-003 — Snap Maximize
Returns the complete usable work-area rectangle; never full monitor bounds that would cover a taskbar or reserved appbar.

### AC-4.2-004 — DPI neutrality
At 96/120/144/192 DPI and with negative monitor coordinates, supplied physical-pixel coordinates are preserved and DPI is not applied a second time.

### AC-4.2-005 — Deterministic failure
Empty, inverted, or unrepresentable geometry yields a deterministic planning failure with no partial placement.

## Dev Notes

### One Boundary, Two Halves

Both halves derive from a single `split_x()` value, so `left.right == right.left`
holds by construction rather than by two independent calculations agreeing.
That is what makes "no gap and no overlap" a structural property instead of a
coincidence to be re-verified whenever either function changes.

An odd width gives the floor to the left half and the remainder to the right:
1367 px → 683 + 684. Deterministic, and asserted to be stable across repeated
calls.

### Maximize Uses the Work Area, Never Monitor Bounds

`maximize_never_exceeds_the_work_area` asserts `bottom == 1040` on a
1920×1080 fixture with a 40 px taskbar, and explicitly that it did not reach
1080. Returning monitor bounds here would cover the taskbar — the failure mode
the AC names.

### A Deliberate Degenerate Case

A one-pixel-wide work area splits into an empty left half. Rather than emit a
zero-width placement, `Rect::new` rejects it and the planner returns
`EmptyOrInvertedWorkArea`; the right half remains the whole sliver and stays
valid. Documented and tested rather than left to chance.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 13 unit tests, all desktop-free; no `windows_sys` import.
- Static gates PASS in debug and release.
- **Geometry independently executed** via a standalone `rustc` harness
  (135 checks shared with Stories 4.1/4.3, 0 failures), covering seam tiling,
  odd-width split, negative origin, DPI invariance, and all three failure
  modes.
- `cargo test -p daemon` NOT executed — needs elevation.

### File List
- `crates/daemon/src/arrangement/snap.rs` (new)
