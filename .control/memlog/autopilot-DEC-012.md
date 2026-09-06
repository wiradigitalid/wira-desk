---
artifact: .control/decisions/DEC-012-autopilot-mandate-for-open-fr-ticket-spec-delivery.md
skill: wdi-autopilot
date: 2026-09-06
---

# Memlog — autopilot run DEC-012

## Resume

- Iteration: 0 (preflight only — no work done yet)
- Run branch: `autopilot/DEC-012`, worktree `D:\Developer\wiradigital.id\wira-desk-autopilot`, cut from
  `main` at `9216f4c`. PR: not opened yet.
- Stopped at: mandate just accepted; iteration 1 has not started.
- Blocked: —
- Parked: —
- Next: `wdi-report` intent `estimate` for `SPEC-1` (`FR-26`/`FR-27`, `window-management`), then continue
  `wdi-build` from `SPEC-1`'s next ticket (`SPEC-1-01`, `SPEC-1-02` — planned, tests not yet written).

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, `9216f4c` | Engines: isolated worktree | Cut `autopilot/DEC-012` as a sibling worktree at `D:\Developer\wiradigital.id\wira-desk-autopilot` rather than working in the main checkout | Running the mandate against the shared main checkout | Delete the worktree, re-cut if the path is wrong | `git worktree add` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile (headless, `CLAUDE_CONFIG_DIR=~/.claude-byok`), never the coordinator or the human owner | Guide default of `owner` (no scriptable app launch named in `codebase-stack-guide.md`) | One setting changes via a superseding `DEC-` | `DEC-012` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the owner's existing cron job `00aa92cd` (10m) rather than starting a second loop | A fresh `loop` skill invocation at the default 5m | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-012` |
