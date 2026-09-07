# Builder brief — `SPEC-4-02`, `wdi-build` Step 2 (mandate `DEC-017`)

You are the builder for one ticket. A coordinator wrote the failing tests, will review your diff,
and holds every merge, push, and registry write. You write production code.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. This repo's copy is model-invocable
(`.claude/skills/implement/SKILL.md`); if the Skill tool refuses it, read that file and carry out
its process, and say which of the two you did. No `bmad-*` skill — thirteen are retired at this
gate and denied in `.claude/settings.json`.

## Read first, in this order

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/02-layout-pane-retired-percent-moves-to-stack-row.md`
   — the ticket, **including Amendment 1 and Amendment 2**. Amendment 2 tells you exactly what is
   already committed so you do not rebuild it.
2. `.control/decisions/DEC-014-the-shortcuts-pane-regroups-into-five-taxonomic-groups.md` — read
   the paragraph beginning "Extension accepted the same day", which is what this ticket delivers,
   and its `## Cost` section, which names the dangling-reference risk.
3. `.constitution/project/codebase-stack-guide.md` — the stack, the commands, and how `.slint`
   markup relates to the Rust that binds it.

## Six tests define done. Four are red; do not touch any of them.

| Test | Currently |
|---|---|
| `app::tests::pane_enum_no_longer_declares_layout` | **RED** — `Pane::ALL.len()` is 5, wants 4 |
| `app::tests::overlapping_stack_row_has_percent_true` | **RED** — `Stack.has_percent()` is false |
| `app::tests::the_stack_row_carries_its_own_percent_bounds` | **RED** — `Stack` disagrees with itself |
| `persistence::tests::stack_width_percent_round_trips_through_the_shortcut_row_path` | **RED** — writes 42, reads back 50 |
| `app::tests::pane_declaration_order_is_the_navigation_index` | green guard — keep it green |
| `app::tests::every_pane_index_round_trips_through_the_ui_boundary` | green guard — keep it green |

**A seventh is yours to write**, because it cannot exist before the control it tests:

```
shortcut_row_slint_snapshot::tests::stack_row_percent_commits_on_save_click
```

Write it through `tdd`, at the seam the other tests in
`crates/settings/src/shortcut_row_slint_snapshot.rs` already use — `i-slint-backend-testing`'s
`TestingBackend`, controls located by accessible label. It is an acceptance criterion, not an
optional extra. Note the caveat in `defects.yaml`'s `DEF-5`: that harness sets values through the
accessible-value setter, which is a *different code path* from a physical keystroke. Do not claim
your test proves typed input works; `SPEC-4-04` owns that.

## Two seams are already committed — use them, do not reinvent them

- `ShortcutField::percent_bounds() -> Option<(u32, u32)>` in `crates/settings/src/app.rs` already
  returns `Some((10, 100))` for `Stack` and `Some((1, 99))` for the four `SnapPercent*` rows. The
  numbers live in Rust on purpose. Feed the markup from it.
- `Pane::from_index` in the same file already mirrors `ShortcutField::from_index`, out-of-range
  fallback included.

## The work

- [ ] `ShortcutField::has_percent()` returns `true` for `Stack`.
- [ ] `ShortcutField::percent()` and `set_percent()` read and write `cfg.layout.stack_width_percent`
      for `Stack`. **`set_percent` ends in a `_ => {}` catch-all**, so leaving `Stack` out of it
      fails *silently* — the row draws a control that discards every edit — rather than failing to
      compile. That is what the red round-trip test is catching right now.
- [ ] `ShortcutRowData` (`crates/settings/ui/panes/shortcuts_pane.slint`) grows `percent_min` and
      `percent_max`, populated in `crates/settings/src/main.rs` from `percent_bounds()`.
      `crates/settings/ui/components/shortcut_row.slint`'s `step_plus` and `step_minus` read those
      properties instead of their hardcoded `1`/`99` (lines 39 and 46).
- [ ] `Pane` drops `Layout`, and `main.rs`'s two hand-numbered index tables are **derived** —
      forward as `pane as i32`, reverse through `Pane::from_index` — not renumbered by hand.
- [ ] Renumber the markup's hardcoded pane indices for four panes:
      `crates/settings/ui/components/sidebar.slint` (five `selected: current_pane == N`) and
      `crates/settings/ui/main_window.slint` (the `if root.current_pane == N` bodies). VmExceptions
      moves 3→2 and About 4→3. **Say in your report how you verified the counts**, because no Rust
      test can see these numbers.
- [ ] Delete `crates/settings/ui/panes/layout_pane.slint`, its import and instantiation in
      `main_window.slint`, and the sidebar's Layout nav item.
- [ ] Delete `crates/settings/src/layout_pane_slint_snapshot.rs` **and** its `#[cfg(test)] mod`
      declaration in `crates/settings/src/main.rs`. Its stack-width assertions
      (`out_of_range_stack_width_is_refused_not_clamped` and siblings) **move** to the relocated
      control — migrated into `shortcut_row_slint_snapshot.rs`. No assertion is weakened; the seam
      moves, the guard does not.
- [ ] Carry `DEF-2`'s distinct accessible semantics across: `STACK_WIDTH_DECREASE`,
      `STACK_WIDTH_INPUT`, `STACK_WIDTH_INCREASE` in `crates/settings/src/theme.rs` (declared at
      110-120, referenced again at 239-241 and 275-277). The stepper and the typed field still need
      distinct accessible names on their new row. Dropping them silently regresses a fixed defect.
- [ ] Preserve `stack_width_percent`'s out-of-range-refused-not-clamped behaviour exactly
      (`layout_pane.slint`'s `commit_typed` is the current implementation).
- [ ] Check the first-run onboarding copy (`--onboarding`) for any reference to the `Layout` pane
      by name, and update or remove it. Pointing a new user at a pane that no longer exists is a
      user-visible defect, not a doc nit.
- [ ] `grep -rn "Pane::Layout\|layout_pane\|LayoutPane"` over `crates/` returns nothing, and
      account for every hit you removed. `DEC-014`'s Cost section says not to rely on the compiler:
      a `&str` pane key or a UIA accessible-name table entry will not fail to compile.
- [ ] Full workspace suite green once, not only this ticket's own tests.

## What will fail your work

1. **Do not edit, weaken, rename, or delete any of the six committed tests.** The two green guards
   were verified by mutation — they can fail, and must stay able to. The one permitted test removal
   is `crates/settings/src/layout_pane_slint_snapshot.rs`, and only by *migrating* its assertions.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** Report a document the code now
   contradicts; do not edit it, and do not patch code to match a document.
3. **Do not widen scope.** `SPEC-4-03` does tooltip, vertical centring and no-horizontal-scroll
   polish on this same row afterwards. `SPEC-4-04` fixes `DEF-5` (typed digits not committing).
   Leave both alone.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-02` and stop.
5. **Do not guess at a failure.** Unknown cause → run `wdi-systematic-debugging` before proposing a
   fix. A third failed attempt means escalate in your report, not try a fourth.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`CARGO_TARGET_DIR` is already set to the one shared, warm target tree this run uses. Do not change
it, unset it, or pass `--target-dir` — one build location is the repo owner's explicit instruction.

The suite stands at 137 passed / 4 failed in `settings` and 347 / 58 green elsewhere. Expect every
`settings` test green, plus your new snapshot test, minus whatever
`layout_pane_slint_snapshot.rs`'s assertions become after migrating.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. Each of the six tests: PASS or FAIL. Then your new snapshot test.
3. `cargo fmt` / `clippy` / `cargo test --workspace` — actual final counts, not "all green".
4. **How you verified the markup pane counts**, since no Rust test covers them.
5. Which assertions moved out of `layout_pane_slint_snapshot.rs` and where they landed.
6. Every file changed, one line each on why.
7. Anything the ticket did not anticipate. Report it; do not fix beyond the ticket.
8. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
