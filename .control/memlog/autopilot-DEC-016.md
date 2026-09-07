---
artifact: .control/decisions/DEC-016-autopilot-mandate-for-spec-2-spec-3-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-016

## Resume

- Iteration: 0 — mandate just accepted, loop not yet iterated.
- Run branch: `autopilot/DEC-016`, HEAD `0d3eb1f` (cut from `main` after the preflight backfill). No PR
  opened yet — opens as a draft at the first spec close, per the skill's own rule.
- Stopped at: — (preflight just finished).
- Blocked: —
- Parked: —
- Next: `SPEC-2` is open with two tickets, `SPEC-2-01` `ready-for-agent` and `SPEC-2-02` already `done`
  (preflight backfill). Start `wdi-build` on `SPEC-2-01`.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Engines: isolated worktree | Removed the stale, already-merged `DEC-012` worktree/branch (`../wira-desk-autopilot`, `autopilot/DEC-012`, PR #16 merged) and cut a fresh sibling worktree on `autopilot/DEC-016` from `main` at `0d3eb1f` | Reusing the old worktree/branch, which would have mixed an unrelated finished PR's history with this mandate | Re-cut if the path or branch is wrong | `git worktree remove`, `git branch -d`, `git worktree add` |
| Preflight | Position: red validator | `validate.py` reported `uc-scheduled` RED (`UC-8`/`FR-25`, "Check for updates", shipped without a ticket). Backfilled `SPEC-2-02` against the live, already-passing code/tests (`cargo test --workspace update:: updatecheck::`, 17 passing + 2 pre-existing ignored live tests), same precedent as `SPEC-1`'s `W1-S4..S6`. Committed and pushed directly to `main` (`0d3eb1f`) as preflight remediation, before this mandate's own worktree/branch existed — not run-branch work | Starting the loop over a red corpus, or inventing new behavior to "finish" `UC-8` | If wrong, a superseding `DEC-` reopens `SPEC-2-02` and reverts the backfill | `0d3eb1f`, `specs.yaml`, `.scratch/spec-2-shortcuts-pane-fixes/issues/02-check-for-updates-from-the-about-pane.md` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile, never the coordinator or the human owner — same as `DEC-012`, per the owner's own instruction this turn | Guide default of `owner` | One setting changes via a superseding `DEC-` | `DEC-016` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the cron job created this turn (`3ef639b3`, every 10m) rather than starting a second loop | A fresh `loop` skill invocation | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it, confined to the one autopilot worktree | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-016` |
