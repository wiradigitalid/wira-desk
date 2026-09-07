---
type: decision
id: DEC-013
status: applied
touches:
  - .control/decisions/DEC-013-the-hygiene-gate-permits-the-handle-on-a-decisions-accepted-by-line.md
  - .control/questions/answered.md
  - .control/questions/assumptions.md
  - .control/registry/decisions.yaml
  - scripts/verify-public-export.ps1
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-07
---

# DEC-013 — The hygiene gate permits the maintainer handle on a decision's `accepted_by` line

## Decision

`scripts/verify-public-export.ps1`'s maintainer-handle check permits the handle on a
`.control/decisions/*` file's `accepted_by:` line, and nowhere else in that folder.

## Why

The method requires the handle to be there. `decision-guide.md` makes `accepted_by` a required field
naming **a person and a date**, and its one alternative form — `accepted_by: DEC-<mandate>`, for a
decision accepted under a mandate — is explicitly forbidden for a mandate itself. Neither `DEC-011` nor
`DEC-012` was accepted by delegation, so neither could avoid naming the owner. The gate carried no
`Allowed` entry for `.control/decisions/*`, which made every decision the owner accepts in person a
publication-hygiene failure: `DEC-011` was already failing on `main` before this work started, and
`DEC-012` inherited it. Two consecutive CI runs on identical trees (`34051075317`, `34051968117`)
reported the same two findings, so the failure is deterministic and not a flake.

The handle is already public through commit authorship, so naming the person who accepted a decision adds
no exposure. That is the same reasoning the gate already records inline for `_bmad-output/*` and
`design-system/*`.

The exemption is scoped to the `accepted_by:` line rather than to the folder, because the gate's own
history says why. When its `Allowed` list was last widened, narrowing the exemption to the handle
appearing **as attribution** — instead of exempting those paths wholesale — is what preserved its ability
to catch three references to the private repository URL in the design-system readme. A folder-wide
exemption here would blind it to that same class of leak written anywhere else inside a decision file.

This is a widening of the gate, which its own banner warns against. It is defensible only because the
finding it silences is one the method itself mandates, and because the scope is one line whose format
another guide fixes — not because the finding was inconvenient.

## Cost

The gate no longer reads the handle on a decision's `accepted_by:` line, so anything else written on that
one line in that one folder passes unseen. Priced and accepted: `decision-guide.md` fixes the line's
format, and any drift from it is a defect in the decision file rather than something this gate is the
right place to catch.

This is one more special case in a file whose purpose is to resist special cases, and each one makes the
next request harder to refuse. The line-scoped form is what keeps it arguable; a path-scoped one would
not have been.

Anyone auditing this repository for publication readiness must now know that the maintainer handle
legitimately appears in `.control/decisions/*`, and that its absence there would be the anomaly.
