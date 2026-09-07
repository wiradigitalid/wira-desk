---
topic: wdi-upgrade content pass for wdi_method 0.6.8
artifact: .control/registry/specs.yaml
skill: wdi-upgrade
updated: 2026-09-07
---

- (event) `wdi-method update` had already run today (`.control/wdi-method.yaml`: `wdi_method: 0.6.8`,
  `bmad_method: 6.11.0`, `installed_at: 2026-09-07`, `engines.model_invocation: enabled`) before this pass
  started — this session ran the content half the mechanical update stops short of.
- (event) Ran the full Step 1 checklist against the current corpus. Eleven of twelve probes were already
  in the new shape, most from work already landed earlier this same day (this session's own PRD/blueprint/
  component/build passes, and a prior session's SPEC-1 work):
  1. `requirements.yaml` (old split) — absent. Per-PRD `requirements-<slug>.yaml` already in use.
  2. `specs.yaml` W-ids/epics/stories — present, and correctly left alone: `wdi-build`'s to re-cut, not
     this skill's; already documented as frozen history in prior memlog entries.
  3. `brief.md` old sections — absent. `## Goals` is already the pointer line to `goals.yaml`.
  4. `prd.md` old sections — absent. Already the 7-section order (Why This Initiative through
     Constraints and Guardrails), `FR` prose already lives only in the registry.
  5. SRS UC Catalogue rows — absent in both components' SRS files. Already pointer-only.
  6. SDD Inherited Constraints old shape — absent in both SDDs.
  7. C4 L2 old matrix — absent.
  8/9. Old `.control/generated/{brief,blueprint,prd-*}.md` — absent; `.what-rendered/`/`.how-rendered/`
     already exist and are what `validate.py --generate` writes.
  10. Stale pointers at moved pages — none found outside the skill's own instructional prose (a false
     positive from grepping the skill file itself).
  11. `docs/agents/issue-tracker.md` — already the wdi-method-seeded version (contains "seeded by
     `wdi-method`"); `docs/agents/domain.md` likewise. Not this session's own doing — seeded by today's
     `update` run.
- (decision) Item 12 was the one real finding: every `spec_folder` in `specs.yaml` pointed under
  `_bmad-output/specs/`, none under `.scratch/`. Moved all five folders that still hold live ticket
  content — `w1-verify-0-1-0`, `w4-arrangement`, `spec-1-percentage-and-thirds-snap`,
  `spec-2-shortcuts-pane-fixes`, `spec-3-per-action-shortcut-enable-disable` — via `git mv` to
  `.scratch/<same-slug>/`, keeping each existing lowercase slug (already begins with its spec's id in the
  case-insensitive sense the corpus's own file-naming convention already uses everywhere else). Rewrote
  every `spec_folder:` value in `specs.yaml` (top-level and the frozen `epics`/`stories` nesting alike) to
  match. `W2`/`W3` (`spec-first-run-tutorial`, `spec-settings-window`) were deliberately left pointing at
  their old, now-nonexistent `_bmad-output/specs/` path rather than a fabricated `.scratch/` one — those
  folders were deleted outright earlier this session (already distilled, no flat `tickets:` index reads
  them), so they never existed under `.scratch/` and a `.scratch/` path there would be a false record.
  Both rows carry an inline comment saying so.
- (event) Discovered why deleting W2/W3 earlier this session raised no `cites-resolve` finding even though
  `3p.md`/`docs/3p.md` still cite their old paths in backticks: `cites-resolve`'s own `DESTINATION` tuple
  exempts every `_bmad-output/` citation from existence-checking outright (`validate.py`, "a path a run
  WILL PRODUCE, not one a document cites as existing"). `.scratch/` carries no such exemption, so this
  migration is what makes those citations live again — confirmed by `validate.py --generate` finding
  nothing new after the move (only the pre-existing `uc-scheduled` UC-8 gap, unrelated to this pass).
- (decision) The owner's points on G5 no longer using `bmad-method` and mandating the `mattpocock/skills`
  engines instead are already true of the currently-installed `wdi-build` (retires `bmad-spec`,
  `bmad-build`, `bmad-build-auto`, `bmad-code-review` by name) and confirmed by `.control/wdi-method.yaml`
  (`engines.model_invocation: enabled` for all six: `to-spec`, `to-tickets`, `implement`, `tdd`,
  `code-review`, `domain-modeling`) — nothing to change there; that content is the WDI Method package's,
  not this skill's or this session's to hand-edit. `_bmad/custom/` carries no vestigial customize.toml for
  any of the four retired skills, checked and confirmed clean.
- (event) `_bmad-output/` still holds pre-method legacy content untouched and out of this pass's scope:
  `brainstorming/`, `planning-artifacts/`, `prior-knowledge/`, and one orphaned `specs/spec-wintick/`
  folder with no `specs.yaml` row pointing at it (pre-dates the WDI Method migration; not a pending or
  done spec under this method, so not part of the "retire bmad-output specs" ask). Left as-is; flagged
  for the owner rather than deleted unprompted.
- (event) `validate.py --generate` — GREEN except the pre-existing `uc-scheduled` UC-8 gap (update-check,
  never had a ticket in any wave, unrelated to this pass; already reported in an earlier session entry).
