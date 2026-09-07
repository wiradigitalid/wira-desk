---
artifact: .control/decisions/DEC-016-autopilot-mandate-for-spec-2-spec-3-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-016

## Resume

- Iteration: 6 — `SPEC-2-01` builder finished; Steps 1+2 verified, panel dispatched.
- Run branch: `autopilot/DEC-016`, HEAD `fa3dddc` (builder's own commit, landed directly — not yet a ledger
  commit boundary). No PR opened yet — opens as a draft at the first spec close.
- Stopped at: **Capacity.** Job `bafaw9fiy` exited (code 0) after ~80 min. Did NOT trust its self-report:
  independently re-ran `cargo fmt --check`, `clippy -D warnings`, and the full workspace suite myself in
  this worktree (safe now that the builder process has exited) — clean, 519 passed/0 failed, all 5 named
  ticket tests plus 2 extra present and green; `verify-public-export.ps1` 10/10. Diff touches only
  `crates/settings`, `crates/shared`, `3p.md` — no corpus file. Dispatched Step 3 as two genuinely separate,
  fresh-context review agents (Standards axis `a38e9e2fc328ecbfe`, Spec axis `a08eb5bd423c4a6c3`), neither
  of which is `bafaw9fiy` or this coordinator. Both still running.
- Blocked: —
- Parked: —
- Next: read both review agents' verdicts. Clean on both axes → close `SPEC-2-01` (ticket-closing checklist,
  commit already on the run branch, no PR-per-ticket under a mandate), then move to the `SPEC-3-01` frontier.
  A must-fix → amend the ticket, reset to `ready-for-agent`, redispatch `claude-byok` (return trip 1/2).

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Iter 1 | `wdi-build` Step 1+2 dispatch | Dispatched `claude-byok` as a detached background CLI session (PowerShell `run_in_background`, dot-sourcing the owner's own `claude-byok` profile function, `-p` with a written builder brief) rather than a synchronous call | A synchronous/blocking invocation, which would stall this iteration for the whole build | None — `TaskOutput`/the harness recovers the session at any later point | `bafaw9fiy`, `.scratch/builder-brief-spec-2-01.md` |
| Preflight | Engines: isolated worktree | Removed the stale, already-merged `DEC-012` worktree/branch (`../wira-desk-autopilot`, `autopilot/DEC-012`, PR #16 merged) and cut a fresh sibling worktree on `autopilot/DEC-016` from `main` at `0d3eb1f` | Reusing the old worktree/branch, which would have mixed an unrelated finished PR's history with this mandate | Re-cut if the path or branch is wrong | `git worktree remove`, `git branch -d`, `git worktree add` |
| Preflight | Position: red validator | `validate.py` reported `uc-scheduled` RED (`UC-8`/`FR-25`, "Check for updates", shipped without a ticket). Backfilled `SPEC-2-02` against the live, already-passing code/tests (`cargo test --workspace update:: updatecheck::`, 17 passing + 2 pre-existing ignored live tests), same precedent as `SPEC-1`'s `W1-S4..S6`. Committed and pushed directly to `main` (`0d3eb1f`) as preflight remediation, before this mandate's own worktree/branch existed — not run-branch work | Starting the loop over a red corpus, or inventing new behavior to "finish" `UC-8` | If wrong, a superseding `DEC-` reopens `SPEC-2-02` and reverts the backfill | `0d3eb1f`, `specs.yaml`, `.scratch/spec-2-shortcuts-pane-fixes/issues/02-check-for-updates-from-the-about-pane.md` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile, never the coordinator or the human owner — same as `DEC-012`, per the owner's own instruction this turn | Guide default of `owner` | One setting changes via a superseding `DEC-` | `DEC-016` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the cron job created this turn (`3ef639b3`, every 10m) rather than starting a second loop | A fresh `loop` skill invocation | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it, confined to the one autopilot worktree | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-016` |
| Iter 6 | `wdi-build` Steps 1+2 judging | Judged from the artifact, not `bafaw9fiy`'s self-report: re-ran fmt/clippy/full suite/export-gate myself once the builder process had exited (no longer a race), read `git show fa3dddc` directly for corpus-boundary violations | Trusting the job's own completion summary | A false-green ticket if the report and the artifact ever disagree | `fa3dddc`, this ledger |
| Iter 6 | `wdi-build` Step 3 dispatch | Two fresh-context `general-purpose` agents, one per axis (Standards `a38e9e2fc328ecbfe`, Spec `a08eb5bd423c4a6c3`), neither carrying this coordinator's own analysis nor `bafaw9fiy`'s report — genuine separation, not a self-review | A single combined review, or reusing this session's own read of the diff | Re-dispatch if either agent's findings turn out unverifiable from the diff | This ledger |
