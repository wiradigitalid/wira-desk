# Builder brief — `SPEC-4-02`, return trip 1 of 2 (mandate `DEC-017`)

Your first build of this ticket was **correct in everything mechanical**, and a two-axis panel
verified that independently: the derived pane index tables, the markup renumbering, the snapshot
migration, the percent plumbing, the refused-not-clamped behaviour. **Do not redo any of it.**

One mechanism has to change. Read `## Amendment 3` in the ticket — that is your whole scope.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. Repo copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and
carry out its process, and say which you did. No `bmad-*` skill.

## Read first

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/02-layout-pane-retired-percent-moves-to-stack-row.md`
   — **Amendment 3 only**. Amendments 1 and 2 and the main list are done.
2. `.control/registry/defects.yaml`, the `DEF-2` row — including the note added at the end of it.
3. `crates/settings/src/theme.rs` lines 100-130, to see the vocabulary you will be extending.

## The problem, stated so you can judge the fix yourself

`crates/settings/ui/components/shortcut_row.slint:23-26` decides four accessible labels like this:

```slint
in property <string> accessible_label_field:
    (root.percent_min == 10 && root.percent_max == 100) ? "Stack width field" : "Snap percentage field";
```

That is a **validation range** standing in for a control's **identity**. Two things follow, and the
second is why this is a must-fix rather than a tidy-up:

1. Give a snap-percentage row a 10-100 range — a user-configurable percentage feature, so not
   hypothetical — and all four snap rows start announcing themselves to a screen reader as "Stack
   width …", colliding with the real Stack row. That is defect `DEF-2` reproduced in the pane
   `DEF-2` was filed against.
2. **Both Stack snapshot tests find their row by `find_by_accessible_label(&window, "Stack width
   input")`**, which resolves only because `Stack` alone happens to have those bounds. So the guard
   that proves stack width is refused-not-clamped would silently start asserting against a *snap*
   row while its messages still said "Stack width" — and still pass.

Separately, your commit deleted `focus_order(Pane::Layout)`, which was the one production reader of
`theme.rs`'s three `STACK_WIDTH_*` constants. They are now referenced only by `theme.rs`'s own two
tests, so `accessible_names_are_unique` — which `defects.yaml` names as `DEF-2`'s regression test —
asserts uniqueness across strings the UI never draws. That is not your mistake; retiring the pane
had to remove that arm. It does mean the fix below has a second job: make those constants live again.

## The work

- [ ] `crates/settings/src/theme.rs` gains the names Rust never declared: a "field" wrapper name for
      each kind (the markup currently spells `"Stack width field"` and `"Snap percentage field"`
      itself), plus the snap-percentage decrease/input/increase set. Reuse the three existing
      `STACK_WIDTH_*` constants — do not re-spell their strings.
- [ ] `ShortcutRowData` (`crates/settings/ui/panes/shortcuts_pane.slint`) carries the four
      accessible labels, populated in `crates/settings/src/main.rs` from those constants **per
      `ShortcutField`**. Identity comes from *which field the row is*. The seam already exists —
      your own commit put `percent_min`/`percent_max` through it, beside `title` and `description`.
- [ ] `shortcut_row.slint`'s four `accessible-label` bindings read those properties. The
      `percent_min == 10 && percent_max == 100` condition **disappears from all four sites**.
- [ ] Write `theme::tests::every_rendered_percent_control_name_comes_from_theme`: every accessible
      name a percentage control actually renders is one of the `theme.rs` constants. **Prove it can
      fail** — point one label at a bare literal, watch the test go red, restore it, and say in your
      report that you did. A guard never seen red is a claim, not proof; this repo's rules require it.
- [ ] The two Stack snapshot tests keep working with their row selection no longer contingent on a
      numeric coincidence. Do not weaken either assertion — the refused-not-clamped one in
      particular must still assert `150` uncommitted, the `SaveFeedback::Error` message containing
      both `"Stack width percentage"` and `"between 10% and 100%"`, and the saved config still at 50.

## Already done by the coordinator — do NOT redo

- `ShortcutField::percent_bounds()` reads `MIN_SNAP_PERCENT`/`MAX_SNAP_PERCENT` and
  `MIN_STACK_WIDTH_PERCENT`/`MAX_STACK_WIDTH_PERCENT` from `crates/shared/src/constants.rs`.
- `main.rs`'s `unwrap_or(...)` fallback reads those constants too.
- `app::tests::every_percentage_row_reaches_all_four_percent_seams` exists and is green.
- `defects.yaml`'s `DEF-2` row, `3p.md`, and `CHANGELOG.md [Unreleased]` are all updated.

## What will fail your work

1. **Do not edit, weaken, rename, or delete any existing test.** Several were verified by mutation
   and must stay able to fail. Adjusting how the two Stack snapshot tests *locate* their row is
   permitted and expected; changing what they assert is not.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** Report a document the code
   contradicts; do not edit it.
3. **Do not widen scope.** `SPEC-4-03` owns tooltip, vertical centring and no-horizontal-scroll on
   this same row. `SPEC-4-04` owns `DEF-5`. Do not extend `focus_order` to cover percent
   sub-controls — that is new scope applying equally to four rows nobody asked about.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-02` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.** Your last
   run discarded an uncommitted coordinator file that way. Commit your own work; leave everything
   else alone.
6. **Do not guess at a failure.** Unknown cause → `wdi-systematic-debugging` before any fix. A third
   failed attempt means escalate in your report, not try a fourth.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`CARGO_TARGET_DIR` is already set to the one shared, warm target tree. Do not change it, unset it,
or pass `--target-dir` — one build location is the repo owner's explicit instruction.

The suite stands at **547 passed / 0 failed / 2 ignored**. Expect 548 with your new test.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. `cargo fmt` / `clippy` / `cargo test --workspace` — actual final counts, not "all green".
3. **The mutation you ran on the new guard**: what you broke, that it went red, and that you restored it.
4. How the two Stack snapshot tests now select their row, and why that is no longer a coincidence.
5. Every file changed, one line each on why.
6. Anything Amendment 3 did not anticipate. Report it; do not fix beyond the brief.
7. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
