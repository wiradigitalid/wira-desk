# Planning archive

**Read this before anything below it.**

These are **historical planning artifacts** from the development of this product — a PRD,
an architecture spine, epics and stories, meeting minutes, readiness assessments, and a
technical spec. They are published so development can continue with its reasoning intact.
They are **not** product documentation.

Four things follow from that, and they matter:

**Decisions here may have been superseded, and superseded ones are not marked
individually.** Making the history look consistent with the present would falsify it. The
most important instance: **this project is MIT licensed.** Any statement in this archive
that implies otherwise — including references to a closed-source build process — is
historical.

**Numbers here are internal requirement targets, not product claims.** RAM budgets, latency
figures, and binary size goals appear throughout as things the design aimed at. Several were
never met, and at least one was found to be unachievable by construction and formally
replaced. For what the product actually does and what can be verified about it, read
`README.md`, `SECURITY.md`, and `PRIVACY.md` in the repository root instead.

**The product was originally named WinTick.** That name appears throughout, including in
file and directory names. The rename is described in the root `README.md`.

**Some content is redacted**, marked inline as `[redacted: <class>]`. The classes are
commercial strategy, marketing strategy, and cost figures. Local machine paths and personal
remarks were removed without markers.

## What is here

| Directory | Contents |
| --- | --- |
| `planning-artifacts/` | PRD and addendum, architecture spine, epics and stories, meeting minutes, readiness assessments, a sprint change proposal, and UX design documents |
| `implementation-artifacts/` | Per-story specifications, validation reports, and the sprint status ledger |
| `specs/` | The technical spec kernel and its conventions |
| `brainstorming/` | The intent document from the original ideation session |

Two kinds of artifact are deliberately absent: per-session AI memory logs, which are
regenerated on each run and carried the densest concentration of redacted material, and the
generated HTML keepsakes of the naming exploration.

## Using these with BMAD

This archive is the output of the BMAD workflow. The tooling itself is not committed — see
`CONTRIBUTING.md` for how to install it. The artifacts here are readable and useful without
it; BMAD is what regenerates and extends them.
