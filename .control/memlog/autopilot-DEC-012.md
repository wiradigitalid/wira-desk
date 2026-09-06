---
artifact: .control/decisions/DEC-012-autopilot-mandate-for-open-fr-ticket-spec-delivery.md
skill: wdi-autopilot
date: 2026-09-06
---

# Memlog — autopilot run DEC-012

## Resume

- Iteration: 5
- Run branch: `autopilot/DEC-012`, worktree `D:\Developer\wiradigital.id\wira-desk-autopilot`, HEAD
  `ede6656`. PR: not opened yet (opens at first spec close, per mandate).
- Stopped at: capacity — `bl2cjjxeb` has already committed `SPEC-1-01`'s work (`ede6656`, "All 499 tests
  pass" claimed in the message) but `TaskOutput` confirms the job itself is still `running` — do not treat
  the commit as the finish line.
- Blocked: —
- Parked: —
- In flight: harness background job `bl2cjjxeb`, still running past its first commit. Do not run
  cargo build/test in this worktree, do not start the Step 3 review, and do not dispatch `SPEC-1-02` until
  `bl2cjjxeb` is confirmed no longer `running` (via `TaskOutput ... block:false` or a completion
  notification) — a commit mid-job is not the same as the job being done.
- Next: once `bl2cjjxeb` is confirmed finished — independently verify (this is now safe: nothing else will
  be building in this worktree once the job has actually exited) rather than trusting the commit message's
  "499 tests pass" claim, then run Step 3 (Standards + Spec axes review, as a separate agent from the
  builder). On a clean review, `SPEC-1-01` is closed (already on the run branch, no merge needed), then
  dispatch claude-byok for `SPEC-1-02` next, sequentially. Once both tickets are done, close `SPEC-1`, push
  the run branch, open the one draft PR, watch CI.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, `9216f4c` | Engines: isolated worktree | Cut `autopilot/DEC-012` as a sibling worktree at `D:\Developer\wiradigital.id\wira-desk-autopilot` rather than working in the main checkout | Running the mandate against the shared main checkout | Delete the worktree, re-cut if the path is wrong | `git worktree add` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile (headless, `CLAUDE_CONFIG_DIR=~/.claude-byok`), never the coordinator or the human owner | Guide default of `owner` (no scriptable app launch named in `codebase-stack-guide.md`) | One setting changes via a superseding `DEC-` | `DEC-012` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the owner's existing cron job `00aa92cd` (10m) rather than starting a second loop | A fresh `loop` skill invocation at the default 5m | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-012` |
| Iter 1 | `wdi-build` Step 2 dispatch | Dispatched claude-byok as a detached background CLI session (`claude --bg`, `--dangerously-skip-permissions`, `CLAUDE_CONFIG_DIR=~/.claude-byok`) rather than a synchronous call, since Step 1+2 for a real ticket can run far longer than one tool-call window | A synchronous/blocking invocation | None — `claude logs`/`claude agents` recover the session at any later point | This ledger |
| Iter 1 | Work order across `SPEC-1`'s two tickets | `SPEC-1-01` first, `SPEC-1-02` only after it finishes — sequential despite both having `depends_on: []` | Building both in parallel (allowed by the guide since there's no blocking edge) | None — just slower than it had to be | This ledger |
| Iter 2 | Capability failure, `wdi-build`'s own rule | `claude --bg`'s detached process was killed by this sandbox's job-object cleanup — session `4d098608` never persisted, no commits, nothing to recover. Not recorded against the ticket (capability failure, not a ticket failure) | Treating it as a ticket-level Blocked | — | This row |
| Iter 2 | Capability failure, retry mechanism | Re-dispatched via the harness's own Bash/PowerShell `run_in_background` tracking instead of `claude --bg`'s OS-level daemon — first Bash attempt also failed (`CLAUDE_CONFIG_DIR` alone isn't enough auth; got "Not logged in") until the actual `claude-byok` PowerShell profile function (`~/Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1`) was found and used instead of hand-rolling the env vars | Guessing at `ANTHROPIC_API_KEY`/`ANTHROPIC_BASE_URL` values directly, or asking the owner before checking for an existing launcher | If `claude-byok`'s own env wiring ever changes, re-read the profile function rather than trusting this row | This row, `b8nzz6xny` |
| Iter 3 | Capability failure, PowerShell redirection | `b8nzz6xny` still failed — `*>` (all-streams redirect) on a native command makes PowerShell 5.1 wrap any stderr line (here, a harmless model-metadata warning) in a terminating `NativeCommandError`, killing the job before `claude-byok` ran at all. Re-dispatched with no stream redirection (harness already captures stdout/stderr for a background job) as `bl2cjjxeb`, confirmed actually running before this iteration ended | Redirecting only stderr elsewhere, or writing to a log file at all | If it fails a third time for a new reason, escalate per `wdi-systematic-debugging` rather than trying a fourth variant | This row, `bl2cjjxeb` |
| Iter 5 | Judging "done" for a background builder | `bl2cjjxeb` committed `ede6656` while `TaskOutput` still reported it `running` — treated the job's actual exit, not its first commit, as the completion signal; deferred review and cargo commands until it exits | Starting Step 3 review the moment a commit appears | A ticket "closed" while claude-byok still mid-run to it, or a build race if I'd run cargo concurrently | This row |
