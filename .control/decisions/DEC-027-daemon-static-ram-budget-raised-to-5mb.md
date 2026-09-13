---
type: decision
id: DEC-027
status: accepted
touches:
  - .control/decisions/DEC-027-daemon-static-ram-budget-raised-to-5mb.md
  - .control/registry/decisions.yaml
  - .control/registry/goals.yaml
  - .control/registry/requirements-wira-desk.yaml
  - .control/registry/defects.yaml
  - .what/window-management/SRS-window-management.md
  - .how/window-management/SDD-window-management.md
  - .how/settings/SDD-settings.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
supersedes: null
superseded_by: null
created: '2026-09-13'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-13
---

# DEC-027 — The daemon's static RAM budget rises from 2 MB to 5 MB, and gains a defined metric

## Context

`BG-2` and `NFR-1` have promised a daemon "static RAM footprint under 2 MB" since the product was
first written down. The number appears in eight corpus places and is quoted in `FR-9`'s proof.

Two things have happened to it.

1. **`DEF-11` (open, low) found it unmeasured and unmet.** A smoke test for `DEC-017` read
   `WorkingSet64 = 29.7 MB` and `PrivateMemorySize64 = 4.0 MB` against a 2 MB promise. `DEF-11`
   recorded that nothing in the workspace asserts a memory ceiling, that `FR-9` never says which
   metric "static RAM" means, and that the reading came from a stale build resident for eight hours.
2. **SPEC-12 and SPEC-13 added a visual switcher.** A layered overlay window, a GDI back buffer, one
   live DWM thumbnail registration per visible card, and cached window icons did not exist when 2 MB
   was set. SPEC-14 widens candidate collection across monitors, which raises the card count again.

Manual evaluation of the 0.4.0 build observed ~3.6 MB.

## Decision

1. **The budget is "under 5 MB".** `BG-2`, `NFR-1`, `FR-9`'s proof, both SDDs, the spine's binary
   table, and `design-system.md` all carry the new figure.
2. **The hard ceiling stays 10 MB.** `NFR-1` already carried it and it is unchanged.
3. **The metric is now stated, because a budget without one cannot be met or missed.** The figure is
   **private bytes (`PrivateMemorySize64`), measured on a release build, idle, with the overlay
   closed.** Working set is explicitly *not* the governed number: it includes shareable pages the
   process does not own, it moves with system memory pressure, and the 29.7 MB `DEF-11` recorded is
   mostly not the daemon's cost. Where a figure is quoted outside the corpus it carries the metric
   with it.
4. **The overlay's transient cost is outside the idle budget and is not yet bounded.** Thumbnails and
   the back buffer exist only while the switcher is open. No number is set for them here; SPEC-14's
   cross-monitor widening is the first change that makes the peak scale with monitor count rather
   than with one monitor's window count.

## Why

Stating a budget the product has never been able to meet, against a metric nobody chose, is worse
than stating a looser one that can be checked. 2 MB was set for a headless daemon that drew nothing;
the product now deliberately draws an overlay, and that was an accepted, recorded choice
(`SPEC-12`, `SPEC-13`, `DEC-026`), not a regression. 5 MB private keeps the promise meaningful —
it is still an order of magnitude under a typical Electron tray utility, which is the comparison the
claim exists to win — while leaving the switcher room to exist.

## Cost

**This decision does the thing `DEF-11` explicitly warns against, and that is deliberate.**
`DEF-11`'s `fix_direction` reads: *"MUST NOT be 'fixed' by editing the number in `FR-9` to match
whatever is measured — that is the corpus learning to agree with whatever was built, and the promise
predates the measurement."* That warning is aimed at an **agent** silently reconciling a promise to a
measurement. This is the **owner** re-setting their own budget, with a stated reason that is not "we
measured 3.6" but "the product grew a feature the old budget predates". The distinction is the whole
of the difference, and it is recorded here so a later reader does not mistake one for the other.

**`DEF-11` is not closed by this.** Its second half stands untouched: **nobody has yet measured a
fresh idle release build.** The 4.0 MB private figure came from a daemon built three weeks before the
code it was judged against, and the 3.6 MB came from manual observation, not an instrumented run.
5 MB is therefore a budget set from reasoning, not from evidence, and the measurement is still owed.
`DEF-11` is retargeted rather than resolved.

**Nothing asserts it.** There is still no test, gate, or smoke step anywhere in the workspace that
checks a memory ceiling. The new number is exactly as unenforced as the old one; it is merely
honest about what it means. Making it enforceable is not scoped here.

## Owner decisions still open

- **Should anything measure this?** A smoke step reading `PrivateMemorySize64` on a fresh idle
  release build would turn `NFR-1` from a claim into a check, and would close `DEF-11` properly. Not
  scoped here.
- **Does the overlay's transient peak need its own budget?** Clause 4 leaves it unbounded. Worth a
  number once SPEC-14's cross-monitor collection lands and the peak can be observed against a real
  multi-monitor candidate set.
