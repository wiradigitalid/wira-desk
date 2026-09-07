---
artifact: .control/decisions/DEC-017-autopilot-mandate-for-spec-4-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-017

## Resume

- Iteration 1 — `SPEC-4-01` Step 1 closed (seven tests red at the right assertions, `3d3a9bf`);
  **Step 2 dispatched and IN FLIGHT**. Boundary commit: `c8a9aad` on `ticket/SPEC-4-01`.
- Run branch: `autopilot/DEC-017` at `c58e703`; ticket work rides `ticket/SPEC-4-01`, cut from it, and
  merges back only green, so the run branch never carries red. No PR open yet.
- Stopped at: — (iteration in progress, waiting on a dispatched step)
- Blocked: —
- Parked: —
- **In flight — do NOT re-dispatch:** `claude-byok` builds `SPEC-4-01` as background job `ble3851qv`,
  brief at `.scratch/builder-brief-spec-4-01.md`. Confirmed working from the worktree itself (five
  files modified inside ticket scope), not from its own report. The coordinator holds the build lock
  shut while it is out: run **no** `cargo` command until this job exits.
- Next: when `ble3851qv` exits — judge Step 2 from the artifact (re-run fmt/clippy/full suite and read
  the diff, never the builder's summary), then Step 3's two-axis panel by agents that are not the
  builder, then merge green into the run branch and delete `ticket/SPEC-4-01`.

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
| Iter 1 | `wdi-build` Phase 3 scope | Found a second declared order the ticket did not name: `daemon`'s `Chords::in_declared_order` and `load_shortcuts_from_config`'s row table order the collision unbinding that actually runs, while `ShortcutField::ALL` orders only the pane. `LBR-ST-14` forbids exactly that, naming both components. Amended `SPEC-4-01` to carry both crates plus a shared source of the order, and opened `DEC-018` for it | Shipping the ticket as written — a settings-only reorder, which leaves `DEC-014`'s recorded precedence consequence undelivered and makes the pane's `DEC-009` warning name a winner the daemon does not pick | If the unification is wrong, `DEC-018` is superseded and the constant collapses back into two lists; the reorder itself still stands | `DEC-018`, ticket amendment 1, `specs.yaml` |
| Iter 1 | `DEC-014` internal inconsistency | `DEC-014`'s Why called Maximize's old slot "position 5" (it is index 6) and its new one "second-to-last" (its own group enumeration puts it third-to-last). Took the Decision section's group list as normative — Maximize at index 13, ahead of `MoveNextMonitor` and `Stack` — and corrected the Why prose to match, permitted because `DEC-014` is `accepted`, not `applied` | Following the Why's "second-to-last", which would put `MoveNextMonitor` ahead of Maximize and contradict the same decision's own group membership | If the owner meant the Why literally, one entry swaps and the tests move with it | `DEC-014`, ticket amendment 1 |
| Iter 1 | `wdi-build` Step 1 authorship | Wrote the seven failing tests myself and added `SHORTCUT_DECLARED_ORDER` with them, because the constant IS the acceptance criterion expressed as data and without it the suite would fail to compile rather than fail an assertion. Every behaviour change stays `claude-byok`'s | Handing Step 1 to `claude-byok` too, against the owner's split (unit tests are the coordinator's) | If the constant's order is wrong, six tests say so immediately | `crates/shared/src/constants.rs`, `crates/settings/src/app.rs`, `crates/daemon/src/hook.rs` |
| Iter 1 | Run-branch hygiene | Ticket work rides `ticket/SPEC-4-01` cut from the run branch and merges back only once green, so `wdi-build`'s "nothing lands on the run branch red" holds literally while TDD's red phase is still committed and handed to the builder. Same worktree, one build location — the owner's constraint — since the four tickets are a serial chain and never build concurrently | Committing red tests straight onto the run branch, or opening a second worktree per ticket | None; the branch is deleted after the merge and only the run branch is ever pushed | `ticket/SPEC-4-01` |
