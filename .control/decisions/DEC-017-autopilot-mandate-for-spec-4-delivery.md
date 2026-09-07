---
type: mandate
id: DEC-017
status: accepted
touches:
  - .control/decisions/DEC-017-autopilot-mandate-for-spec-4-delivery.md
  - .control/memlog/autopilot-DEC-017.md
  - .control/registry/decisions.yaml
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-07
---

# DEC-017 — Autopilot mandate for SPEC-4 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1`
through `SPEC-3` closed, `SPEC-4` open at 0/4 tickets with its `SPEC.md` already written and reviewed
(`eb667fa`, edge-case-hunter) — through every runnable `FR`/spec/ticket to a single reviewable PR,
without further owner check-ins until Finish or a parked row. Parameters live on this decision's row in
`decisions.yaml` under `mandate:`.

In scope: `SPEC-4`'s four tickets (`SPEC-4-01` to `SPEC-4-04`, a strict `depends_on` chain), delivering
`FR-7`, `FR-15`, `FR-18`, `FR-26`, `FR-28`, and applying `DEC-014` plus defect `DEF-5`. Not work: the
eight `FR` rows the RTM marks `exempt: true, broken_at: no_uc` — they have no `UC`, are excluded from
the promise count, and inventing one to "finish" them is the failure `DEC-016`'s own preflight row
already refused once.

## Why

The owner wants every open `FR`/ticket/spec delivered unattended, on the split `DEC-012` and `DEC-016`
both proved out and which the owner restated this turn: coding and the smoke test belong to a separate
`claude` CLI profile (`CLAUDE_CONFIG_DIR=~/.claude-byok`), invoked headlessly by the coordinator; the
coordinator keeps specs, documents, registry writes, this ledger, unit tests, review, and every merge.

One change from `DEC-016`, at the owner's explicit instruction: **there is exactly one build location.**
The run's isolated sibling worktree (see `worktree` on this decision's row) is pinned to the main
checkout's already-warm target directory (see `build_target_dir`), so dependencies stay compiled, no
second target tree is ever created, and cargo's own lock serialises anything that would otherwise race.
The coordinator holds every `cargo` invocation; claude-byok receives source and the smoke test, never a
concurrent build of its own.

Because `claude-byok` resolves to a non-Claude model through 9router, every step it produces is judged
from the artifact — the diff, the suite, the export gate — and never from its own completion report.
That is already the method's rule for any builder; it is restated here because the asymmetry makes it
load-bearing rather than a formality.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch and its worktree are the only place ticket work lands, and
`main` is reached through exactly one PR the owner merges.

Three known gaps this mandate accepts rather than hides. `codebase-stack-guide.md` still describes the
settings UI as `eframe`/`egui` when the crate depends on `slint`, so it is corrected before any coding
is dispatched — a builder reading it would reach for the wrong toolkit. `review-trace` is stale on
`.how/settings/SDD-settings.md`, so `wdi-review` re-runs on it before `SPEC-4` closes. And the last
run's automated pass could not drive the Settings window at all (`SetForegroundWindow` refused to a
background process), which is why `DEF-5` was found by hand; `SPEC-4` is entirely a UI change, so if
that path is unreachable again the smoke result is recorded **NOT VERIFIABLE**, never PASS, and the
owner is handed a live test script instead.
