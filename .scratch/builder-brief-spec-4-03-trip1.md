# Builder brief — `SPEC-4-03` **return trip 1**, `wdi-build` Step 2 (mandate `DEC-017`)

The three visual defects you delivered are genuinely fixed in the markup, and the coordinator has
verified that by reproducing the measurements independently. **This trip is the panel's must-fix
list, not a redo.** One item is a user-visible behavioural regression neither review axis predicted.

The coordinator's half of this trip is already committed at `0e93b9b` — the guards, the mutation
reports, the two stale markup comments, `CHANGELOG`, `3p.md`, `specs.yaml`. **Do not redo those**,
and read the ticket's Amendment 3 before you start: it records which of the panel's own
instructions turned out to be wrong.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. The repo's copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and
carry out its process, and say which you did. No `bmad-*` skill — thirteen are retired at this gate.

## Read first, in this order

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/03-row-polish-tooltip-centring-no-scroll.md`
   — **Amendment 2** is your work list, **Amendment 3** is what the coordinator learned running the
   mutations, including two corrections to Amendment 2's own instructions.
2. `.what/settings/02-rules/rules-settings.md` — `LBR-ST-5` and `LBR-ST-14`, both `status: active`.
   Item 1 below is a live conflict with `LBR-ST-5`.
3. `.constitution/project/codebase-stack-guide.md` — the commands, and how `.slint` markup relates
   to the Rust that binds it.

## Item 1 — the Tab-order regression. `wdi-systematic-debugging` FIRST

`FocusScope` defaults to `focus-on-tab-navigation: true` and `title_block` does not override it, so
all sixteen rows gained an undeclared Tab stop. A throwaway probe dispatching real
`WindowEvent::KeyPressed { Key::Tab }` measured it:

```
PROBE: tooltip surfaced after Some(1) Tab presses
```

**One press from a fresh window reaches a row's description block.** So it is not an extra stop
appended to the end of the order — it is at or near the front of it. `LBR-ST-5` requires *"a
deterministic Tab navigation order that starts with navigation tabs and terminates with action
buttons."* Tab from a fresh Settings window now lands on a row description instead of the sidebar.

- **Root-cause it with `wdi-systematic-debugging` before changing anything.** Why *one* press
  reaches it is not yet known, and the two obvious remedies pull opposite ways: suppressing the
  stop (`focus-on-tab-navigation: false`) protects `LBR-ST-5` but may cost the keyboard half of
  `FR-20`/`FR-21` that criterion 2 requires; declaring the stop in `app::focus_order` keeps the
  feature but must place it where `LBR-ST-5` says, not first. **Do not pick one blind.**
- Whichever holds, `crates/settings/src/app.rs`'s `focus_order` must end up **true**. Note honestly
  that it was already fictional before this ticket — no rendered element carries `f.label()` as an
  accessible name, and `focus_order_mismatch` only ever compares `focus_order`'s output to itself.
  This is drift into an existing hole, and it is still the hole this criterion walked into.
- Prove the final order with a **real `Key::Tab`** sequence, not `invoke_accessible_default_action()`.
  Proving a focus path through the accessibility harness instead of a keystroke is `DEF-5`'s exact
  mechanism, recurring inside the ticket series opened to fix `DEF-5`'s siblings.

## Item 2 — seven constants, six with no consumer

`theme.rs` gained seven `ControlSemantics`. Six have zero readers anywhere; the seventh is read
only by a test, which makes the constant a fixture that must match markup rather than a declaration
markup obeys — the coupling backwards. The strings appear a third and fourth time as raw literals
in the tests.

- **Delete the five `GROUP_HEADING_*` constants.** `ShortcutField::GROUPS` is already the one home
  for those five strings, `shortcuts_pane.slint` renders them through `root.heading`, and `app.rs`
  already guards `GROUPS` against `group()`. A second unenforced copy of a single-source array is
  what `DEC-018` exists to prevent.
- Keep `SHORTCUT_KEYCAP` and `SHORTCUT_ROW_DESCRIPTION`, but **consume them the way this file
  already consumes the other four** — an `in property` on `ShortcutRow`, filled from Rust in
  `main.rs` — instead of literals in markup. That plumbing exists eleven lines away.

## Item 3 — sixteen controls, two names

Both new labels are row-invariant, so the tree carries sixteen elements named `"Shortcut keycap"`
and sixteen named `"Shortcut description"`, one of them on a `FocusScope` and one on an
`accessible-role: button`. That is `DEF-2`'s literal shape. Eleven lines below, `"Enable " +
root.title` does it correctly. Make them per-row.

**`DEF-6` stays out of scope.** It is a *different* set — the four snap rows sharing one
accessible-name set across their percentage controls. Do not give those per-direction names here.

### The trap in this item, and it is the whole reason it is spelled out

Making these labels per-row **breaks every test that finds them by the invariant literal**, and it
breaks them *silently*: `find_by_accessible_label(&window, "Shortcut keycap")` will simply yield
nothing, and a loop that finds nothing passes. That is exactly the vacuity `b28d5ab` closed, and
these two guards are mutation-verified — the coordinator has seen each of them red:

| Guard | Mutation it has been seen red under |
|---|---|
| `row_height_is_independent_of_description_length` | `min-height` keyed to `description.character-count` gives pitches `[63, 57, 57]` |
| `five_groups_fit_the_default_window_width_with_no_horizontal_scroll` | keycap `max(135px …)` widened to `max(700px …)` gives right edge 1248 against 760 |

So, as part of this item:

- Update both guards to derive the per-row label from the **same one home** you introduce — the
  `theme` constant plus the row's own title — never from a hand-written literal.
- **Re-run those two mutations afterwards and report the failure text.** A guard that stopped
  finding its elements is indistinguishable from a guard that works, and this item is the single
  most likely way to introduce that.
- `accessible_names_are_unique` cannot see any of this: it iterates `ALL`, not the rendered tree.

## Item 4 — two lines that claim something they do not deliver

- `min-height: 50px` on the row: the panel's arithmetic says the row's natural height is already
  50px (30px cluster + 10px + 10px padding), so deleting the line changes no pixel. Mutation 2
  above shows the height guard catches a min-height that varies **between** rows, so keeping it is
  not unguarded — but it gives no purchase on removing it. Either give it a real assertion or
  remove it. Do not leave a line that looks like a guarantee and guarantees nothing.
- Remove the test-only `accessible-action-default => { self.focus(); }` on `title_block`
  (`shortcut_row.slint`). It exists for no production purpose — it is there so a test could focus
  the block. That is test-driven API in the component, and it goes with item 1's real-`Key::Tab`
  proof.

## What will fail your work

1. **Do not weaken, rename, or delete any existing test.** `shortcut_row_slint_snapshot.rs`,
   `shortcuts_pane_slint_snapshot.rs`, `theme.rs` and `app.rs` all carry mutation-verified guards.
   Where item 3 forces a test to change, it changes to keep measuring the same property — and you
   prove that with the mutation, not with an argument.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** `LBR-ST-5` is your constraint, not
   your output. If you conclude the rule is wrong, **report that** — it becomes a `DEC-`, and it is
   the coordinator's to open, never a code patch.
3. **Do not widen scope.** `SPEC-4-04` owns `DEF-5`; `DEF-6` is filed and explicitly not yours.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-03` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.** A
   previous iteration lost a committed file to a working-tree restore.
6. **Do not guess at a failure.** Unknown cause means `wdi-systematic-debugging` before any fix. A
   third failed attempt means escalate in your report, not try a fourth.
7. **No smoke test.** The app is not launched in this trip; that is a separate step, and the
   owner's mandate is that it happens once, at the end.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

`--no-fail-fast` is not optional: a runner that stops at the first failure hides the rest, and this
trip touches guards across three modules.

`CARGO_TARGET_DIR` is already set to the one shared, warm target tree. **Do not change it, unset
it, or pass `--target-dir`** — one build location is the repo owner's explicit instruction, and
while you hold this dispatch you hold the build lock alone.

The suite stands at **553 passed / 0 failed / 2 ignored**. Say what it is when you finish.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. **Item 1: the root cause, from `wdi-systematic-debugging`** — why one Tab press reaches the
   block — then which remedy you chose and why, and the real-`Key::Tab` sequence that proves the
   final order. If `LBR-ST-5` and `FR-20`/`FR-21` cannot both be satisfied, say so plainly; that is
   a decision, not a defect, and it comes back to the coordinator.
3. **Item 3: the two mutations re-run, with their failure text.** This is the report line most
   likely to be quietly skipped, and the one that matters most.
4. `cargo fmt` / `clippy` / `cargo test --workspace --no-fail-fast` — actual final counts.
5. Every file changed, one line each on why.
6. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
