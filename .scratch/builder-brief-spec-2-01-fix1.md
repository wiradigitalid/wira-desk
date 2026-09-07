# Builder brief — SPEC-2-01, return trip 1/2

You are the **builder** for a fix round on one ticket under an active WDI autopilot mandate (`DEC-016`).
You are a fresh session — you did not write the original code. A separate review panel (not you, not the
coordinator) found one confirmed must-fix bug in it. The coordinator verified the finding directly against
the code before sending it back; it is real.

## Ticket

Read `.scratch/spec-2-shortcuts-pane-fixes/issues/01-commit-on-departure-and-stepper-parity.md` in full —
the **"Return trip 1/2"** section near the end is what you're fixing now. Read it directly; do not rely on
this brief's summary of it.

## What to do

1. Reproduce the bug first: `on_percent_changed` in `crates/settings/src/main.rs` (around line 397)
   unconditionally clears the shared `uncommitted_percent` slot before checking whether it belongs to a
   *different* row than the one just changed — so typing into row A, then triggering a commit on row B
   (stepper, Tab, etc.), silently discards row A's typed value instead of applying it.
2. Write a new test that reproduces this cross-row scenario (type in row A, trigger a commit path on row
   B, assert row A's typed value survived and is not silently reverted). Confirm it's red for the right
   reason before fixing.
3. Fix `on_percent_changed` so it drains the shared slot the same way its siblings already do
   (`on_start_capture`, `on_swap_shortcuts`, `on_pane_selected` in the same file are the reference
   pattern — take-and-apply, never discard) before applying the just-changed field's own new value.
4. While you're in this file: wire the unused `DEFAULT_STACK_WIDTH_PERCENT` constant
   (`crates/shared/src/constants.rs`) into `LayoutConfig::default()` in `crates/shared/src/config.rs`
   (currently a bare `50` literal at line ~172) — match the sibling `SnappingConfig::default()` pattern a
   few lines above it, which already uses `crate::constants::DEFAULT_SNAP_PERCENT`.
5. Run the **full workspace suite green once**, not just the new test:
   ```
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   $env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
   ```
6. **Commit to the current branch** (`autopilot/DEC-016`). Do not push, do not open a PR.

## Three rules that bind every builder in this corpus

- **Debugging is conditional, never a phase.** If a test or build fails and you don't know why, invoke
  `wdi-systematic-debugging` before proposing any fix. A third failed fix attempt means stop and report.
- **The corpus is not yours to change.** Do not edit `.what/`, `.how/`, or an `applied` `DEC-`. This
  ticket's `satisfies: [UC-4, UC-9]` is not yours to amend, whatever you find.
- **Verification is run, not assumed.** Actually run the commands above in this worktree.

## Boundaries

- Isolated git worktree, checked out to `autopilot/DEC-016`, exclusively yours until you finish.
- Do not touch any other ticket, spec, or the registries (`specs.yaml`, `decisions.yaml`, `index.yaml`,
  `components.yaml`).
- This is a **fix round** — the panel will run again after you finish, on this new commit, from scratch.
  When done (or stuck at a third failed fix, or blocked on a real scope question), stop and write one
  final message: which step you reached, the commit SHA, and whether the full suite is green.
