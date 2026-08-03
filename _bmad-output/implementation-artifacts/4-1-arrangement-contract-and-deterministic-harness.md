---
baseline_commit: dd91900
workflow_id: story-4-1-arrangement-contract
---

# Story 4.1: Arrangement Contract and Deterministic Harness

Status: review

## Story

As a desktop organizer,
I want snapping and stack commands to use one stable arrangement contract,
so that window-layout capabilities can be delivered independently without destabilizing keyboard interception or cycling.

## Acceptance Criteria

### AC-4.1-001 — Frozen command contract
`SnapLeft`/`SnapRight`/`SnapMaximize`/`OverlappingStack` retain their frozen `u8` values; unknown values decode as `Nop`; no payload added to the wake-only Worker notification.

### AC-4.1-002 — Frozen shortcut defaults
Half-left `Ctrl+Win+Left`, half-right `Ctrl+Win+Right`, maximize `Ctrl+Win+Enter`, stack `Ctrl+Win+Down`; stack disabled by default; default stack width 50 percent.

### AC-4.1-003 — No inter-monitor command
WinTick implements no inter-monitor arrangement command; movement between monitors stays delegated to native `Win+Shift+Arrow`.

### AC-4.1-004 — Validation outside the callback
An invalid or colliding mapping produces a Tier-2 warning; the last-known-good mapping or documented default stays active; the Hook Thread performs no parsing, allocation, logging, or blocking operation.

### AC-4.1-005 — Geometry invariants
Rectangles use signed physical-pixel coordinates and half-open edges; the contract carries monitor work area and DPI context; checked arithmetic prevents overflow, negative width, and negative height.

### AC-4.1-006 — Harness and lane ownership
The harness captures placement plans without invoking User32; cycling behavior unchanged; `arrangement/snap.rs` → 4.2, `arrangement/stack.rs` → 4.3, `arrangement/win32.rs` → 4.4; `arrangement/mod.rs`, `hook.rs`, `worker.rs`, final composition reserved for 4.5.

## Dev Notes

### Half-Open Edges Are Load-Bearing

`left <= x < right`. This is not a stylistic choice: it makes "left half plus
right half exactly tiles the work area" true by construction. With inclusive
edges, every tiling assertion would need an off-by-one correction that some
future caller would inevitably get wrong. `halves_tile_the_work_area_without_gap_or_overlap`
asserts the seam directly.

Signed coordinates are required because a secondary monitor legitimately sits
at negative coordinates; `negative_origin_work_area()` covers that.

### DPI Is Carried, Never Applied

`WorkArea::dpi` exists for traceability only. Coordinates arrive already in
physical pixels from a Per-Monitor-V2-aware process, so any planner that scaled
by DPI would scale twice. Both 4.2 and 4.3 assert identical plans across 96,
120, 144, and 192 DPI for identical geometry.

### `validate()` vs `is_valid()`

The first implementation used a single `is_valid() -> bool`, which collapsed
"extent overflowed `i32`" and "rectangle is empty or inverted" into one `false`.
A `Rect { left: i32::MIN, right: i32::MAX }` was therefore reported as
`EmptyOrInvertedWorkArea` when it is actually `UnrepresentableGeometry`.

**This was caught by executing the geometry**, not by review — see the
verification note below. `Rect::validate()` now returns the distinguishing
error and every planning path uses it; `is_valid()` remains as a convenience
predicate with a doc comment warning against using it on error paths.

### Scope

`AC-4.1-004` (Tier-2 warning on malformed mapping) is **contract-level only**
here: `PlanError` carries the deterministic failure, but wiring it to the
Tier-2 `log::warn` path belongs to Story 4.5 convergence, which owns
`worker.rs`. Flagged rather than silently assumed complete.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 17 unit tests in `arrangement/mod.rs`, all desktop-free. The module imports no
  `windows_sys`.
- Static gates PASS: `cargo clippy --workspace --all-targets` exit 0 in debug
  **and** release; test binary compiles.
- **Geometry logic independently executed.** Because `cargo test -p daemon`
  needs elevation, the pure geometry was extracted verbatim into a standalone
  `rustc` harness and run: **135 checks, 0 failures** after the `validate()`
  fix. This is real execution evidence for the math, though not for the
  in-crate test wiring.
- `cargo test -p daemon` still NOT executed.

### File List
- `crates/daemon/src/arrangement/mod.rs` (new)
- `crates/daemon/src/main.rs` (modified — `mod arrangement;`)
