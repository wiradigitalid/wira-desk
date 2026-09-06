---
artifact: .control/decisions/DEC-012-autopilot-mandate-for-open-fr-ticket-spec-delivery.md
skill: wdi-autopilot
date: 2026-09-06
---

# Memlog — autopilot run DEC-012

## Resume

- Iteration: 1
- Run branch: `autopilot/DEC-012`, worktree `D:\Developer\wiradigital.id\wira-desk-autopilot`, cut from
  `main` at `9216f4c`. PR: not opened yet (opens at first spec close, per mandate).
- Stopped at: capacity — claude-byok dispatched in the background for `SPEC-1-01`; cannot be waited on
  synchronously. Coordinator does not touch this worktree's build/test while it runs.
- Blocked: —
- Parked: —
- In flight: claude-byok background session `4d098608` building ticket `SPEC-1-01` (Custom-percentage edge
  snap, Steps 1+2 of `wdi-build`'s pipeline: TDD + implement + full suite green + commit to
  `autopilot/DEC-012`, no push). Check with `claude logs 4d098608` / `claude agents --json` before doing
  anything else in this worktree.
- Next: once `4d098608` reports done — (a) coordinator reviews the diff as a separate agent from the
  builder (Step 3: Standards + Spec axes, `code-review`/`bmad-code-review`), judging from claude-byok's own
  logged command output, not its chat summary; (b) on a clean review, ticket `SPEC-1-01` is closed (already
  committed to the run branch by claude-byok — no separate merge needed in this single-worktree design);
  (c) dispatch claude-byok for `SPEC-1-02` next, sequentially (not concurrently, by design); (d) once both
  tickets are done, close `SPEC-1`, push the run branch, open the one draft PR, watch CI.
  If `4d098608` is still running, just re-check next firing — do not dispatch a second claude-byok session
  into this worktree concurrently.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, `9216f4c` | Engines: isolated worktree | Cut `autopilot/DEC-012` as a sibling worktree at `D:\Developer\wiradigital.id\wira-desk-autopilot` rather than working in the main checkout | Running the mandate against the shared main checkout | Delete the worktree, re-cut if the path is wrong | `git worktree add` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile (headless, `CLAUDE_CONFIG_DIR=~/.claude-byok`), never the coordinator or the human owner | Guide default of `owner` (no scriptable app launch named in `codebase-stack-guide.md`) | One setting changes via a superseding `DEC-` | `DEC-012` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the owner's existing cron job `00aa92cd` (10m) rather than starting a second loop | A fresh `loop` skill invocation at the default 5m | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-012` |
| Iter 1 | `wdi-build` Step 2 dispatch | Dispatched claude-byok as a detached background CLI session (`claude --bg`, `--dangerously-skip-permissions`, `CLAUDE_CONFIG_DIR=~/.claude-byok`) rather than a synchronous call, since Step 1+2 for a real ticket can run far longer than one tool-call window | A synchronous/blocking invocation | None — `claude logs`/`claude agents` recover the session at any later point | This ledger |
| Iter 1 | Work order across `SPEC-1`'s two tickets | `SPEC-1-01` first, `SPEC-1-02` only after it finishes — sequential despite both having `depends_on: []` | Building both in parallel (allowed by the guide since there's no blocking edge) | None — just slower than it had to be | This ledger |
