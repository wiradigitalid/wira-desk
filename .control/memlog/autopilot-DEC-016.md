---
artifact: .control/decisions/DEC-016-autopilot-mandate-for-spec-2-spec-3-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-016

## Resume

- Iteration: 10 — run branch pushed at `SPEC-2` close; PR #17 opened draft; `SPEC-3-01` building.
- Run branch: `autopilot/DEC-016`, HEAD `f9840fe`, pushed. **PR #17 open, draft**:
  https://github.com/wiradigitalid/wira-desk/pull/17.
- Stopped at: **Capacity.** Pushed at `SPEC-2` close per the mandate's own rule (first push opens the PR).
  CI on `f9840fe` (run `34108292593`): `dependencies`/`publication-hygiene`/`secrets` PASS, `build` still
  running (it typically takes ~11-12 min per `DEC-012` precedent). `SPEC-3-01` builder (`bd03n1l84`)
  confirmed still running.
- Blocked: —
- Parked: —
- Next: check both `bd03n1l84` and PR #17's `build` check. On `bd03n1l84` exit: verify independently
  (never the report), dispatch the review panel. Once `build` concludes: judge per Step 5 (only mark ready
  if green; red keeps it draft, reported red, not patched to force green). `SPEC-3-02` is next after
  `SPEC-3-01` closes — `blocked_by: [SPEC-3-01]`, not started in parallel.

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
| Iter 7 | Adjudicate Standards-axis must-fix | Confirmed the unused `DEFAULT_STACK_WIDTH_PERCENT` finding is real (read `constants.rs`, `config.rs` myself) but reclassified follow-up, not must-fix — no acceptance-criterion break, no behaviour delta, `config.rs` untouched by this ticket's own diff; bundled into the fix round anyway since it's cheap | Sending it back to Step 2 on its own, or silently accepting the reviewer's must-fix label without reading the lines | If wrong, a second return trip fixes it in isolation | `config.rs:172`, this row |
| Iter 7 | Adjudicate Spec-axis must-fix | Confirmed real by reading `main.rs:291-408` myself: `on_percent_changed` discards a different row's pending value instead of draining it, unlike its three sibling handlers in the same file — reachable by mouse, silent data loss, breaks checkbox 3. Genuine must-fix, return to Step 2 | Accepting the ticket as closed on the panel's mixed verdict | A shipped ticket with the exact bug it was meant to eliminate | `main.rs:397-408`, ticket file |
| Iter 7 | Return trip 1/2 | Amended the ticket in place with a dated "Return trip 1/2" section (reproduction, fix instruction, bundled constant fix); status stays `ready-for-agent` (never left it — builder doesn't touch ticket status). Redispatched `claude-byok` fresh (`bq40cmkpl`) rather than reusing `bafaw9fiy`'s context | Giving the fix as a chat instruction instead of a ticket amendment | Cap is 2 return trips; a second failure escalates | ticket file, `bq40cmkpl` |
| Iter 8 | Fix-round verification | Read `git show 867a509` myself before trusting `bq40cmkpl`'s report; independently re-ran fmt/clippy/full suite (520/0 failed)/export gate | Trusting the completion summary | A false-green ticket if the report and artifact disagree | `867a509`, this ledger |
| Iter 8 | Panel re-dispatch on fix round | Re-ran the WHOLE panel fresh (Standards `a96267b6e0bba1ad0`, Spec `a13e70ff3d92f7f89`) rather than only re-checking the one prior finding — a fix can introduce its own defects | Spot-checking just the fixed line | A new regression the fix itself introduced going unreviewed | This ledger |
| Iter 9 | Close `SPEC-2-01` / `SPEC-2` | Both re-review agents clean; closed the ticket (registry + status `done`) and the spec (`inventory.py` 0 gaps, RTM green, `specs.yaml` → `closed`) | Trusting the panel's self-report alone | Reopen if a later finding contradicts it | ticket file, `specs.yaml`, `d54c209` |
| Iter 9 | `wdi-reconcile` after spec close | Found real drift: 3 corpus files (`LC-settings-shell.md`, `SDD-settings.md`, `design-system.md`) still named the pane `"Layout & Snapping"` after this spec renamed it to `"Layout"`. Checked `EXPERIENCE.md`/`DESIGN.md` too — already correct, left untouched. Fixed all 3, present tense | Leaving the stale name since it's "just a label" | A reader looking for a tab that no longer exists under that name | `f7760db` |
| Iter 9 | `SPEC-3-01` dispatch | Dispatched fresh `claude-byok` (`bd03n1l84`) for the per-action enable-flag ticket; `SPEC-3-02` not started in parallel since it's `blocked_by: [SPEC-3-01]` | Building both `SPEC-3` tickets at once | None — just the correct dependency order | `bd03n1l84` |
| Iter 10 | Push + open the one PR | Pushed `autopilot/DEC-016` and opened PR #17 as **draft** at `SPEC-2`'s close, per the mandate's own rule (first push opens the PR; the coordinator pushes at every spec close, not per ticket) | Waiting until Finish to push everything at once | None — this is exactly what the rule specifies | PR #17, `f9840fe` |
