# Builder brief — `SPEC-4-03`, `wdi-build` Step 2 (mandate `DEC-017`)

You are the builder for one ticket. A coordinator will review your diff and holds every merge,
push, and registry write.

**This ticket is different from the two before it: no failing tests are handed to you.** They could
not be written in advance, and § Amendment 1 of the ticket explains exactly why. **You write the
tests, through `tdd`, before the change each one describes.**

## Run the engine

Invoke **`/implement`** with this brief and the ticket. Repo copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and
carry out its process, and say which you did. No `bmad-*` skill — thirteen are retired at this
gate and denied in `.claude/settings.json`.

## Read first, in this order

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/03-row-polish-tooltip-centring-no-scroll.md`
   — the ticket, **including Amendment 1**, which is the seam agreement and the reason you are
   writing the tests.
2. `.how/settings/01-ux/DESIGN.md` — the bullets **"Row description as tooltip, not a truncated
   line"**, **"Row vertical centring"**, and **"No horizontal scroll"**. These are this ticket's
   design source, already written in the present tense for exactly this work. The corpus is ahead
   of the code here; your job is to make the code true, not to re-decide the design.
3. `.constitution/project/codebase-stack-guide.md` — the commands, and the section on how `.slint`
   markup relates to the Rust that binds it.

## Three defects, from the owner's live test

**1. The description draws as a permanently-visible, elided line.**
`crates/settings/ui/components/shortcut_row.slint` renders `root.description` as a `Text` with
`wrap: no-wrap; overflow: elide`. On several rows it always overflowed, so an ellipsis was the only
part of it some users ever saw. It becomes a **tooltip on the title-and-description block**,
carrying the full text, and the row draws nothing extra when pointer and focus are elsewhere.

Reachable **by keyboard focus, not only hover** — `FR-20`, `FR-21`. A row reached by Tab must be
able to surface its own description. This is the half most likely to be skipped, because hover is
the obvious reading of "tooltip".

Removing the always-visible line changes the row's height. That file's own header comment explains
why every text in it is single-line and what the divider positioning depends on — re-verify the row
height is still deterministic afterwards.

**2. The enable toggle reads higher than the keycap beside it.**
The percent block and the `ToggleSwitch` are already both inside the centring `VerticalLayout`, so
this is **not** a nesting problem — do not "fix" it by re-nesting and calling it done. It is a
cross-axis sizing question, and the honest first step is to **measure**: get the keycap's and the
toggle's `absolute_position()` and `size()` and see what the actual centres are. Fix what the
measurement shows. The fix must hold both for a one-line row and for a row whose keycap has wrapped
to its minimum width.

**3. Horizontal scroll at the default window width.**
Five groups and sixteen rows must fit without the shell growing a horizontal scrollbar. Where the
right-hand cluster (stepper + keycap + toggle) is too wide, **narrow the cluster's own layout** —
tighter spacing between its parts. Do not let the pane grow past its shell and do not add sideways
scrolling: a Windows settings dialog scrolls vertically only.

The width is `main_window.slint`'s `width: is_onboarding ? 580px : 760px`. This pane is the normal
window, so **760px**. Read it from there rather than hardcoding it a second time.

## The four tests, and where they go

| Test | Module |
|---|---|
| `description_renders_as_a_tooltip_not_a_visible_line` | `crates/settings/src/shortcut_row_slint_snapshot.rs` |
| `control_cluster_and_toggle_share_one_vertical_centre` | same |
| `five_groups_fit_the_default_window_width_with_no_horizontal_scroll` | **new** `crates/settings/src/shortcuts_pane_slint_snapshot.rs` |
| `all_five_group_headings_are_in_the_rendered_tree` | same new module |

Declare the new module beside the two existing `#[cfg(test)] mod` lines in
`crates/settings/src/main.rs`.

The last one is coverage `SPEC-4-01`'s review panel deferred to this ticket: three of the five
groups — *Snap to half*, *Snap to third*, *Resize, move & arrange* — have **no** real-window check
today. Iterate `ShortcutField::GROUPS` rather than restating the five strings. That is not style:
in `SPEC-4-02`, a hand-listed array in `theme.rs` silently missed two entries and a review caught
it, not the suite.

## What you will need, and one trap

`i_slint_backend_testing::ElementHandle` finds elements **by accessible label**, then exposes
`size()` and `absolute_position()`. That is the only route to geometry from a test.

**Today the keycap, the title-and-description block, and the five group headings carry no accessible
label at all** — so none of them is findable. Adding those labels is part of this ticket's work, not
a workaround: defect 1's keyboard-focus requirement already means the description must be in the
accessible tree, and a screen-reader user should reach a chord and a group name too. Put the new
names in `crates/settings/src/theme.rs` alongside the existing `ControlSemantics` constants **and
add them to `theme::ALL`** — that array is hand-maintained, its doc comment says so, and a constant
missing from it is invisible to both register tests.

**The trap:** in Slint's testing backend, accessible elements inside a scroll area are instantiated
**only once scrolled into view**. A group below the fold is not absent because your layout is
wrong — it is absent because nothing scrolled to it. Distinguish those two before concluding
anything, especially in the no-horizontal-scroll test.

Compare centres with an **explicit tolerance stated in the test**, not exact float equality.

## Mutation discipline — this ticket has no red suite, so this is what stands in for one

Every one of the four tests must be **seen red before the code that satisfies it**. Any that starts
green — the heading one may — must be broken deliberately, watched fail, and restored.

**Report, per test, which mutation you ran and what the failure said.** A guard never seen red is a
claim rather than proof, and here there is no handed-down red suite to lean on.

## What will fail your work

1. **Do not weaken, rename, or delete any existing test.** Several in
   `shortcut_row_slint_snapshot.rs` and `theme.rs` are mutation-verified.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** `DESIGN.md` is your source, not your
   output. If the code turns out right and it turns out wrong, report that; do not edit it.
3. **Do not widen scope.** `SPEC-4-04` owns `DEF-5` (typed digits not committing) — you may touch
   the same file but must not fold that fix in. `DEF-6` (all four snap rows sharing one
   accessible-name set) is filed and explicitly **not** this ticket's: do not give the snap rows
   per-direction names.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-03` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.**
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

The suite stands at **548 passed / 0 failed / 2 ignored**. Expect 552 with your four.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. `cargo fmt` / `clippy` / `cargo test --workspace` — actual final counts, not "all green".
3. **Per test: the mutation you ran, that it went red, and that you restored it.**
4. What the measurement in defect 2 actually showed — the two centres before your fix. If they were
   already equal, say so plainly rather than changing something to look busy; that would be a real
   finding about where the defect lives.
5. Which accessible labels you added, and that each is in `theme::ALL`.
6. Every file changed, one line each on why.
7. Anything the ticket or Amendment 1 did not anticipate.
8. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
