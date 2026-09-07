---
type: decision
id: DEC-015
status: applied
touches:
  - .control/decisions/DEC-015-the-hygiene-gate-permits-internal-ids-under-scratch.md
  - .control/registry/decisions.yaml
  - scripts/verify-public-export.ps1
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-07
---

# DEC-015 — The hygiene gate permits internal ids under `.scratch/`

## Decision

`scripts/verify-public-export.ps1`'s "internal requirement or process vocabulary" check adds `.scratch/*`
to its `Allowed` list, the same exemption `_bmad-output/*` already carries and for the identical reason.

## Why

`wdi-method 0.6.8`'s upgrade (`.control/memlog/upgrade-0.6.8.md`) moved every spec's planning content —
`SPEC.md`, ticket files, story files — from `_bmad-output/specs/<slug>/` to `.scratch/<slug>/`, per
`docs/agents/issue-tracker.md`'s new convention. The gate's own inline comment already states the test
this exemption applies: *"An identifier is opaque when it is referenced far from its meaning — these are
the meaning."* `AD-2`, `FR-26`, `BR-6` and the like inside a spec's `SPEC.md` are exactly that — reference,
not leakage — the same category of file `_bmad-output/*` was exempted for, now living at a new address.

CI on `af5fc11` confirmed this precisely: 19 findings, every one inside a file this session's own
`wdi-upgrade` pass moved from an already-exempt path to a non-exempt one, nothing newly written. The
gate did exactly its job — catching a real consequence of the move that the upgrade pass itself did not
think to check, since `verify-public-export.ps1` is a script, not a corpus document `cites-resolve` or any
other validator reads.

## Cost

Anything else later written under `.scratch/` for a reason other than spec/ticket content — an ad hoc
scratch note, per `docs/agents/issue-tracker.md`'s own second use for the directory — also stops being
checked for this pattern. Priced and accepted: the same trade `_bmad-output/*`'s exemption already made,
and `.scratch/`'s own documented purpose (planning and tracking content, never shipped product source)
matches the exemption's boundary the same way.

Anyone auditing this repository for publication readiness must now know `.scratch/` is the planning
archive's new address and carries the same allowance `_bmad-output/` did.
