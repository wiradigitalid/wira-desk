---
artifact: .control/decisions/DEC-012-autopilot-mandate-for-open-fr-ticket-spec-delivery.md
skill: wdi-autopilot
date: 2026-09-06
---

# Memlog — autopilot run DEC-012

## Resume

- Iteration: 7
- Run branch: `autopilot/DEC-012`, worktree at `../wira-desk-autopilot` relative to the main checkout (`git
  worktree list` finds its actual path), HEAD `b49204b`. PR: not opened yet (opens at first spec close, per
  mandate).
- Stopped at: capacity — `SPEC-1-01`'s code is committed and independently verified green (fmt/clippy/501
  tests, 0 failed), Step 3 review dispatched as two parallel agents (`abb560804106a7f88` Standards,
  `a868fce8df0c9fc15` Spec). Cannot be waited on synchronously.
- Blocked: —
- Parked: —
- **Near-miss, resolved but worth restating**: job `b8nzz6xny` (iteration 2's first `claude-byok`
  dispatch) was wrongly presumed dead after its log showed a PowerShell `NativeCommandError` from `*>`
  redirecting a harmless stderr warning. It was NOT dead — `TaskOutput` later showed it `running`, and it
  kept running concurrently with its replacement (`bl2cjjxeb`) for several iterations, finishing only at
  iteration 7 with one small commit (`b49204b`, a 3p.md test-count correction). This was briefly two
  claude-byok processes in the same worktree at once — exactly the race this mandate exists to prevent.
  No actual damage: worktree stayed clean, history stayed linear, `b49204b` only touched a doc line,
  independent test run (501 passed) still holds. Root cause and fix are in Decisions below — the practice
  going forward is `TaskOutput(block:false)` to check a job's real status, never inferring death from a
  redirected stderr line.
- Both review agents reported and were independently verified against the diff/gate, not trusted as
  written. Two real findings: (1) Spec must-fix — Slint's percentage input silently clamps out-of-range
  values instead of refusing them, contradicting the ticket's own checklist line; ticket amended with the
  finding, status reset to `ready-for-agent` (return trip 1/2). (2) Standards must-fix on files *I* wrote
  (not claude-byok's) — `verify-public-export.ps1` flagged 4 literal local-path hits in `DEC-012`/ledger/
  `decisions.yaml`; fixed by rewording to `../wira-desk-autopilot` / "see decisions.yaml" instead of the
  absolute path. A 5th/6th finding (maintainer-handle in `.control/decisions/*`) matches a pre-existing,
  already-failing pattern (`DEC-011`, on `main` before this mandate) — filed as `OQ-37`, not decided
  unilaterally. Three Standards *smells* (duplicated range-check logic, duplicated per-edge planner shape,
  triple command dispatch) are follow-up-only, not must-fix — left unfixed, not returned to Step 2 for.
- Blocked: —
- Parked: —
- `b4i44ijoi` completed (confirmed via `TaskOutput`, not the log), commit `1d15f5c` — `Math.clamp` removed
  from `shortcut_row.slint`, an out-of-range value now reaches backend validation. Independently verified:
  fmt/clippy/full suite green (chain exit 0), `verify-public-export.ps1` unchanged at 2 findings (only the
  already-filed `OQ-37`, nothing new). One incidental note: claude-byok's own commit also edited one word in
  this ledger's Decisions table (replacing a literal path I'd left in a finding's own description with
  "developer workstation path") — a real, correct fix, but technically not the coordinator's-only-write rule
  being followed to the letter; recorded rather than silently accepted.
- Blocked: —
- Parked: —
- In flight: independent re-review of just commit `1d15f5c` (agent `a6cedeca717fb4baa`) — genuinely separate
  from claude-byok, since its own internal review is self-review by construction.
- Next: once the re-review reports — on clean, close `SPEC-1-01` (ticket status already `ready-for-review`
  from claude-byok; coordinator's own review is what actually closes it) and dispatch claude-byok for
  `SPEC-1-02`, sequentially, confirming via `TaskOutput` throughout. If the re-review finds a real must-fix,
  that would be return trip 2/2 (the cap) — a third failed fix escalates rather than retrying. Once both
  tickets are done, close `SPEC-1`, push the run branch, open the one draft PR, watch CI.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, `9216f4c` | Engines: isolated worktree | Cut `autopilot/DEC-012` as a sibling worktree (`../wira-desk-autopilot`, see `decisions.yaml`) rather than working in the main checkout | Running the mandate against the shared main checkout | Delete the worktree, re-cut if the path is wrong | `git worktree add` |
| Preflight | Settings: `smoke_test` | `agent`, delegated specifically to the `claude-byok` CLI profile (headless, `CLAUDE_CONFIG_DIR=~/.claude-byok`), never the coordinator or the human owner | Guide default of `owner` (no scriptable app launch named in `codebase-stack-guide.md`) | One setting changes via a superseding `DEC-` | `DEC-012` row, `decisions.yaml` |
| Preflight | Settings: `loop` | Ride the owner's existing cron job `00aa92cd` (10m) rather than starting a second loop | A fresh `loop` skill invocation at the default 5m | Cancel one, keep the other | `decisions.yaml` mandate row |
| Preflight | Runtime: build/test ownership | Coordinator never runs `cargo build`/`cargo test`/`./build.ps1` or launches the app in the shared worktree; claude-byok's own ticket-closing checklist covers it | Coordinator running the full suite itself between claude-byok invocations | A concurrent build race in the shared worktree | This ledger, `DEC-012` |
| Iter 1 | `wdi-build` Step 2 dispatch | Dispatched claude-byok as a detached background CLI session (`claude --bg`, `--dangerously-skip-permissions`, `CLAUDE_CONFIG_DIR=~/.claude-byok`) rather than a synchronous call, since Step 1+2 for a real ticket can run far longer than one tool-call window | A synchronous/blocking invocation | None — `claude logs`/`claude agents` recover the session at any later point | This ledger |
| Iter 1 | Work order across `SPEC-1`'s two tickets | `SPEC-1-01` first, `SPEC-1-02` only after it finishes — sequential despite both having `depends_on: []` | Building both in parallel (allowed by the guide since there's no blocking edge) | None — just slower than it had to be | This ledger |
| Iter 2 | Capability failure, `wdi-build`'s own rule | `claude --bg`'s detached process was killed by this sandbox's job-object cleanup — session `4d098608` never persisted, no commits, nothing to recover. Not recorded against the ticket (capability failure, not a ticket failure) | Treating it as a ticket-level Blocked | — | This row |
| Iter 2 | Capability failure, retry mechanism | Re-dispatched via the harness's own Bash/PowerShell `run_in_background` tracking instead of `claude --bg`'s OS-level daemon — first Bash attempt also failed (`CLAUDE_CONFIG_DIR` alone isn't enough auth; got "Not logged in") until the actual `claude-byok` PowerShell profile function (`~/Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1`) was found and used instead of hand-rolling the env vars | Guessing at `ANTHROPIC_API_KEY`/`ANTHROPIC_BASE_URL` values directly, or asking the owner before checking for an existing launcher | If `claude-byok`'s own env wiring ever changes, re-read the profile function rather than trusting this row | This row, `b8nzz6xny` |
| Iter 3 | Capability failure, PowerShell redirection | `b8nzz6xny` still failed — `*>` (all-streams redirect) on a native command makes PowerShell 5.1 wrap any stderr line (here, a harmless model-metadata warning) in a terminating `NativeCommandError`, killing the job before `claude-byok` ran at all. Re-dispatched with no stream redirection (harness already captures stdout/stderr for a background job) as `bl2cjjxeb`, confirmed actually running before this iteration ended | Redirecting only stderr elsewhere, or writing to a log file at all | If it fails a third time for a new reason, escalate per `wdi-systematic-debugging` rather than trying a fourth variant | This row, `bl2cjjxeb` |
| Iter 5 | Judging "done" for a background builder | `bl2cjjxeb` committed `ede6656` while `TaskOutput` still reported it `running` — treated the job's actual exit, not its first commit, as the completion signal; deferred review and cargo commands until it exits | Starting Step 3 review the moment a commit appears | A ticket "closed" while claude-byok still mid-run to it, or a build race if I'd run cargo concurrently | This row |
| Iter 7 | Root cause: `b8nzz6xny` misjudged dead | A PowerShell `NativeCommandError` written to a redirected log by `*>` does not mean the underlying job died — `$ErrorActionPreference` was never `Stop`, so the background job kept running past that line. Confirmed via `TaskOutput`, which showed it still `running` for iterations 3-6 and only just completed | Assuming the visible error text meant termination, which is what let a second builder get dispatched into the same worktree while the first was still technically alive | Two concurrent claude-byok processes in one worktree, the exact race `DEC-012` exists to prevent — this time harmless (one small doc commit), not guaranteed next time | This row, `b49204b` |
| Iter 7 | Fix: job-liveness check going forward | Every future "is this job still running" question is answered by `TaskOutput(task_id, block:false)`, never by reading a log's content and inferring state from it | Continuing to eyeball log files for signs of life or death | Repeating the same near-miss | This row, all iterations from here |
| Iter 7 | Step 3 dispatch | Two parallel review agents, Standards axis and Spec axis, per `code-review`'s own process — genuinely separate from claude-byok (the builder) | A self-review, or trusting claude-byok's own internal `/code-review` pass (self-review by construction, doesn't satisfy Step 3 per `wdi-build`) | Re-dispatch if either agent's findings turn out unverifiable from the diff | This row, `abb560804106a7f88`, `a868fce8df0c9fc15` |
| Iter 7 | Spec must-fix, verified | Confirmed by reading `shortcut_row.slint` directly: `Math.clamp` on both `TextInput` handlers silently clamps 1-99, contradicting the ticket's explicit "not silently clamped" line. Ticket amended (return trip 1/2), status reset to `ready-for-agent` | Accepting the reviewer's claim without reading the diff myself | A ticket closed with a real acceptance-criterion violation shipped | `01-custom-percentage-edge-snap.md` |
| Iter 7 | Standards must-fix, my own files, fixed | Confirmed by running `verify-public-export.ps1` myself: 4 literal developer workstation path hits in `DEC-012`, the ledger, and `decisions.yaml` — all mine, all fixed in place (still `accepted`, not `applied`, so editable) by rewording to a relative/registry-pointer form | Leaving the literal paths in, or widening the gate's pattern to suppress the finding (the exact failure mode the gate's own comment warns against) | Re-check with the gate again if any future ledger edit reintroduces a literal path | `DEC-012` file, ledger, `decisions.yaml` |
| Iter 7 | Standards must-fix, corpus-wide, not decided | The remaining 2 gate findings (maintainer handle in `.control/decisions/*`, no `Allowed` entry there) match `DEC-011`'s pre-existing violation on `main` — a policy tension between `decision-guide.md`'s required `accepted_by` field and this product's own gate, predating this mandate. Filed as `OQ-37` for the owner rather than invented a fix | Silently widening the gate's `Allowed` list, or silently moving `accepted_by` out of the `DEC-` file against the method's own format | If wrong, the owner's answer supersedes `OQ-37` and this run (or a follow-up) applies it | `.control/questions/assumptions.md` `OQ-37` |
| Iter 10 | Fix round 1/2 result | `1d15f5c` genuinely resolves the clamping finding — verified by reading the diff (removed `Math.clamp` in Slint, `main.rs` now floors a negative to 0 in Rust instead) and by independently re-running fmt/clippy/full suite and the export gate myself, not trusting claude-byok's own report | Accepting the "0 hard violations" self-review claim in its report | A ticket closed with a bug the self-review missed | `1d15f5c` |
| Iter 10 | claude-byok touched the ledger | Its fix commit corrected one literal path fragment I'd left in a Decisions-table cell (via its own export-gate check) — recorded as a rule deviation worth noting, not reverted, since the correction itself is accurate | Reverting the correction to enforce "coordinator-only" strictly, which would reintroduce a real gate finding | None — content is correct either way | This row |
