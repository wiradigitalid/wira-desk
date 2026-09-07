# Builder brief — `SPEC-4-01`, return trip 1 of 2 (mandate `DEC-017`)

The first build of this ticket was correct in everything it touched. A two-axis review panel then
found **a list it did not touch**, and one guard that was asked for but not delivered. You are
fixing those. Do not redo the reorder — it is right.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. This repo's copy is model-invocable
(`.claude/skills/implement/SKILL.md`); if the Skill tool refuses it, read that file and carry out
its process, and say which of the two you did. No `bmad-*` skill — thirteen are retired at this
gate and denied in `.claude/settings.json`.

## Read first

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/01-shortcuts-pane-regroups-per-dec-014.md`
   — **Amendment 2** is your work. Amendments 1 and the main list are already done.
2. `.control/decisions/DEC-018-the-declared-shortcut-sequence-has-one-home-in-shared.md` — its
   `## Why` now lists all three lists and marks which one this trip fixes.
3. `.constitution/project/codebase-stack-guide.md` — the stack and the commands.

## One test is RED. Make it green, and do not touch it.

```
persistence::tests::a_duplicate_names_the_holder_by_the_shared_declared_order
```

It currently reports:

```
assertion `left == right` failed: the later action in the declared sequence is the one refused
  left: "snapping.snap_third_left"
 right: "snapping.snap_maximize"
```

Settings refuses the *earlier* action and names the *later* one as the chord's holder, because
`validate_config`'s field table is still in the pre-`DEC-014` order. The daemon, reordered, unbinds
the opposite one. The pane and the daemon disagree about who keeps a shared chord.

### The fix, and it is not "reorder the literal"

`crates/settings/src/persistence.rs:111-192` — `validate_config`'s
`let fields: [(&'static str, &str, bool); 16] = [ ... ]`.

**Derive its order from `shared::constants::SHORTCUT_DECLARED_ORDER`** instead of hand-keeping a
sequence. Reordering the literal leaves a third copy that the next taxonomy change breaks all over
again; deriving it means the order has exactly one home, which is what `LBR-ST-14` demands.

Shape: iterate `SHORTCUT_DECLARED_ORDER` and map each key to its `(&str value, bool enabled)` pair
from `cfg`. A key-to-value `match` is **not** a second declared order — the iteration order comes
from the constant, so reordering the match arms cannot change behaviour. That is the same pattern
`hook.rs`'s `command_for` helper already uses in its guard test.

Keep the layer boundary that file's header defends: `shared` is already imported there
(`use shared::{config_path, Config, Shortcut};`), and depending on `shared::constants` is **not**
depending on the UI-facing `ShortcutField`. Update that header comment to say what is now true —
it currently claims `app::tests::field_declaration_order_is_the_precedence_order` guards this
table "from the other side", which was never true and is the reason this drift went unseen.

Behaviour that must not change: `validate_config` still returns
`Err((name, ShortcutError::DuplicateShortcut(first_name)))` on the **first** duplicate found,
still skips disabled actions, and still validates every chord's syntax before the uniqueness pass.
Only the order of the walk moves.

## Two smaller items in Amendment 2

- **`main.rs` stops carrying its own copies of the five group headings.** Select rows by indexing
  `ShortcutField::GROUPS` (already committed in `crates/settings/src/app.rs`) rather than passing
  string literals to `group_rows`. A typo in one of those literals makes `group_rows` return an
  empty model and draws an empty group with no error anywhere — that is why the const exists.
- **`Chords`' struct field declaration order** (`crates/daemon/src/hook.rs:606-614`): move
  `snap_maximize` and `move_next_monitor` to their new positions, after the percent fields. Named
  fields, so there is no semantic change — but `hook.rs` currently shows two different orders for
  the same sixteen actions and the struct is the first one a reader meets.

## What will fail your work

1. **Do not edit, weaken, rename, or delete any test.** Not the red one, not the two new guards
   (`the_row_table_binds_every_chord_to_its_own_action`, `the_group_headings_have_one_home`), not
   `the_daemon_precedence_order_matches_the_shared_source`. All three were verified by mutation:
   they can fail, and they must stay able to.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** Report a document that disagrees
   with the code; do not edit it and do not patch code to match it.
3. **Do not redo Amendment 1 or the main checklist.** The reorder across `ShortcutField::ALL`,
   `group()`, `label()`, the markup, `in_declared_order`, the row table and the `resolved[N]`
   mapping is correct — a panel verified all sixteen mappings by hand. Changing it now can only
   break it.
4. **Do not widen scope.** `Pane::Layout` still exists and dies in `SPEC-4-02`. The five
   near-identical `ShortcutGroup` markup blocks stay as they are. The 9-slot fixture in
   `unbinding_makes_the_loser_unreachable_end_to_end` stays as it is.
5. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-01` and stop.
6. **Do not guess at a failure.** Unknown cause → run `wdi-systematic-debugging` before proposing
   a fix. A third failed attempt means escalate in your report, not try a fourth.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`CARGO_TARGET_DIR` is already set to the one shared, warm target tree this run uses. Do not change
it, unset it, or pass `--target-dir`; one build location is the repo owner's explicit instruction.

The whole workspace must be green — the suite stood at 537 passed / 0 failed / 2 ignored before
the three new guards were added. Expect 540 passed once the red one goes green.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. The red test: PASS or FAIL. Then confirm the three guards above are still present, unmodified,
   and passing.
3. `cargo fmt` / `clippy` / `cargo test --workspace` — actual final counts, not "all green".
4. Every file changed, one line each on why.
5. Anything you found that Amendment 2 did not anticipate. Report; do not fix beyond the brief.
6. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
