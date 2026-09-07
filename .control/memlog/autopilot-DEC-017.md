---
artifact: .control/decisions/DEC-017-autopilot-mandate-for-spec-4-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-017

## Resume

- Iteration 1 — mandate accepted; worktree and run branch cut from `main` at `cce1d75`; the two
  document blockers preflight named are closed. Boundary commit: the commit carrying this line.
- Run branch: `autopilot/DEC-017`, in the sibling autopilot worktree. No PR open yet (opens as a draft
  at the first spec close).
- Stopped at: — (iteration in progress)
- Blocked: —
- Parked: —
- Next: `wdi-build` from Step 3 on `SPEC-4-01`, dispatching the coding to `claude-byok`.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Confirmation gate | Read the owner's second, verbatim re-issue of the same `/wdi-autopilot` instruction — sent with the printed preflight page in front of them and no row contested — as the confirmation, rather than printing the page a third time | Asking again, which the skill's own "a preflight that asks fourteen questions has failed" rule refuses | The owner cancels cron `04d63c6f` and supersedes `DEC-017`; nothing has reached `main` at that point | `DEC-017`, this row |
| Preflight | Settings: build location | ONE build location, at the owner's instruction: the run worktree pins `CARGO_TARGET_DIR` to the main checkout's already-warm `target/`, so no second target tree exists and cargo's own lock serialises every build | A per-worktree target directory (`DEC-016`'s shape), which compiles every dependency again and splits builds across two trees | Workspace-crate fingerprints thrash once if builds alternate between the two source dirs; deps stay cached either way | `decisions.yaml` `build_target_dir` |
| Preflight | Settings: parallelism | No parallel builders this run — `SPEC-4`'s four tickets are a strict `depends_on` chain, so the single-worktree rule costs nothing and `wdi-build` § Parallel tickets never applies | Reserving a second worktree that could not legally be used | None | this row |
| Preflight | Settings: `smoke_test` | `agent`, delegated to the `claude-byok` profile alone — never the coordinator, never the human owner — per the owner's instruction this turn and `DEC-012`/`DEC-016` precedent | Guide default of `owner` | One setting changes via a superseding `DEC-` | `DEC-017` row |
| Preflight | Settings: `loop` | Ride the owner's own cron job `04d63c6f` (10m, created this turn) rather than starting a second loop | A fresh `loop` skill invocation | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Engines: mandate authoring route | Wrote `DEC-017` directly against `DEC-016`'s validated file shape and proved it with `validate.py`'s own `mandate-accept` check, rather than invoking `wdi-decision` for the mandate itself | Invoking `wdi-decision`, which costs context the run's Capacity stop is measured in | If the shape is wrong the validator says so in the same turn, before any work lands | `DEC-017`, `validate.py --check` |
| Preflight | Position: document behind code | `codebase-stack-guide.md` describes `crates/settings` as `eframe`/`egui` 0.36 while its manifest depends on `slint` 1.17; corrected BEFORE any coding is dispatched, because it is the guide a dispatched builder loads | Dispatching first and correcting the guide at Finish | A builder reaches for egui and the ticket is thrown away | iteration 1 — see the row below |
| Preflight | Position: review-trace | `review-trace` is advisory-stale on `.how/settings/SDD-settings.md` (changed `079ea56`, reviewed `f7760db`); scheduled `wdi-review` on it before `SPEC-4` closes rather than treating an advisory skip as green | Closing `SPEC-4` on a stale trace | A gate closes over an unreviewed SDD | pending — before spec close |
| Iter 1 | Position: document behind code | Rewrote `codebase-stack-guide.md`'s toolkit rows from the manifest: `slint` 1.17 with its four non-default features, `winit` 0.30, `serde_json`; dropped `eframe`/`egui`/`accesskit`/`ttf-parser` (the last is absent from the whole workspace); added a section on `.slint` markup, the two-places pane declaration, and the accessible-value-vs-keystroke test gap that let `DEF-5` ship. `ratified_by` moved `67f2645` to `c803a1d` | Leaving the guide as the code's record and letting the builder discover the toolkit from `Cargo.toml` | If a claim is wrong the next builder repeats it; every figure here was read from a manifest or a source file, not carried over | `.constitution/project/codebase-stack-guide.md` |
| Iter 1 | Publication hygiene gate | `main` is RED on `verify-public-export.ps1`: `.scratch/live-test-dec-016.md:10` hardcodes an absolute machine path, added at `079ea56` — AFTER PR #17's CI last passed at `9a7c0c6`, so nothing had run the gate since. Fixed the source to name the worktree relatively, the way `DEC-016`'s own mandate file already did | Widening the gate's pattern, which is the exact failure that file exists to prevent | None — gate re-run green, 10 checks passed | `.scratch/live-test-dec-016.md` |
