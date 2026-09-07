---
artifact: .control/decisions/DEC-016-autopilot-mandate-for-spec-2-spec-3-delivery.md
skill: wdi-autopilot
date: 2026-09-07
---

# Memlog — autopilot run DEC-016

## Resume

- Iteration: 14 — every spec closed; § Finish reached; smoke test dispatched.
- Run branch: `autopilot/DEC-016`, HEAD `10eb0e6`. **PR #17 open, draft**:
  https://github.com/wiradigitalid/wira-desk/pull/17. Not pushed since `f9840fe` — pushing once more at
  Finish, not per intermediate commit.
- Stopped at: **Capacity.** Both panel axes clean on `SPEC-3-02` (Standards `afc44e6affecd82f1`, Spec
  `abc48b8283f11c210`) — closed `SPEC-3-02` and `SPEC-3`. Cleared the `review-trace` advisory
  (`SDD-settings.md`, structure+prose, clean on the trivial delta, re-stamped `f7760db`). Ran
  `wdi-reconcile`: found and fixed two real gaps — `BR-9`'s Enforcement still said "Not yet built" for a
  rule that just shipped, and cited the retired one-off toggle as "today's only existing toggle";
  `inventory-db.md`'s `[layout]` schema row still named the retired `enable_overlapping_stack` key instead
  of the real `stack_shortcut_enabled`, and didn't mention the new per-action `*_enabled` fields at all.
  `.control/generated/status.md` now reads `promise_progress: 100%`, RTM `28/28` green, every spec
  `closed`, `gate_readiness: 100%` — **this is § Finish.** Dispatched the smoke test to `claude-byok`
  (job `btmzzj4s0`, confirmed running) per `smoke_test: agent`, covering `FR-25`/`FR-28`/`FR-29` and the
  `SPEC-2-01` UI fix, explicitly told to report honestly what this headless session cannot reach (the
  elevated daemon, real keypress injection) rather than round up.
- Blocked: —
- Parked: —
- Next: check job `btmzzj4s0`. On exit: record its result per `FR` verbatim (never rounded up), then
  `wdi-report` intent `progress`, raise the mandate to `applied`, push, watch CI on the final head, mark
  the PR ready only if green, cancel the loop, write the Finish report.

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
| Iter 11 | CI verdict, `f9840fe` | All 4 checks green — recorded, PR stays draft anyway (only Finish un-drafts it) | Marking ready early since it's green | None — matches the rule exactly | run `34108292593` |
| Iter 11 | `SPEC-3-01` verification | Independently re-ran fmt/clippy/full suite/export-gate before trusting `bd03n1l84`'s report; its claimed internal review does not count as Step 3 | Accepting the builder's self-review as the panel | A false-green ticket if the report and artifact disagree | `5c9c8ac`, this ledger |
| Iter 12 | Close `SPEC-3-01` | Both panel axes clean, same follow-up independently found (orphaned constant) — confirmed real by reading `theme.rs` myself, no behaviour delta, not a return-trip trigger | Treating a duplicate follow-up as two separate findings, or triggering a return trip for a non-behavioural issue | If wrong, a later ticket removes the dead code anyway | ticket file, `9c1257b` |
| Iter 12 | Bundle the follow-up | Carried the orphaned-constant cleanup into `SPEC-3-02`'s builder brief instead of a dedicated dispatch | A separate tiny dispatch just for dead-code removal | None — cheap either way | `.scratch/builder-brief-spec-3-02.md` |
| Iter 13 | `SPEC-3-02` verification | Independently re-ran fmt/clippy/full suite/export-gate, and grepped for the retired field/constant myself, before trusting `bcthvyq4n`'s report | Accepting the builder's self-review as Step 3 | A false-green ticket if the report and artifact disagree | `c803a1d`, this ledger |
| Iter 14 | Clear `review-trace` advisory | Re-ran `wdi-review` (structure+prose) on `SDD-settings.md`'s delta since its last review — a one-word pane-name correction — found clean, re-stamped | Ignoring the advisory since it doesn't fail the gate | Next reconcile pass re-flags if actually stale | `SDD-settings.md` |
| Iter 14 | `wdi-reconcile` after `SPEC-3` | Found real drift: `BR-9` Enforcement said "Not yet built" for a shipped rule and cited the retired toggle; `inventory-db.md`'s `[layout]` row named the retired field instead of the real one, omitted the new per-action fields entirely. Fixed both, present tense | Treating either as merely "behind the code" (both were load-bearing — a reader would misjudge what's built or what key to look for) | A reader trusting either claim | `10eb0e6` |
| Iter 14 | Reached § Finish | `status.md`: 100% promise progress, RTM 28/28 green, every spec closed. Dispatched the smoke test to `claude-byok` per `smoke_test: agent`, told explicitly to report what's unreachable from this headless session rather than round up | Coordinator running the smoke test itself (mandate reserves it for claude-byok only) | None — matches the mandate's own setting | `btmzzj4s0`, `.scratch/smoke-test-dec-016.md` |
