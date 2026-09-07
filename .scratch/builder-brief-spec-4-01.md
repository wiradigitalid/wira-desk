# Builder brief — `SPEC-4-01`, `wdi-build` Step 2 (mandate `DEC-017`)

You are the **builder** for one ticket. A coordinator wrote the failing tests, will review your diff,
and holds every merge, push, and registry write. You write production code and nothing else.

## Run the engine, do not improvise a process

Invoke **`/implement`** and give it this brief and the ticket. This repo's copy of that skill is
model-invocable on purpose (`.claude/skills/implement/SKILL.md`) — if the Skill tool refuses it, read
that file and carry out its process, and say in your final report which of the two you did.

Do **not** reach for any `bmad-*` skill. Thirteen of them are retired at this gate and denied in
`.claude/settings.json`; `bmad-build` is the closest to hand and the furthest from allowed.

## The ticket

`.scratch/spec-4-shortcuts-pane-consolidation/issues/01-shortcuts-pane-regroups-per-dec-014.md`

Read it in full, **including "Amendment 1"**, which is where most of the work is. Then read, in this
order, because the ticket is meaningless without them:

- `.control/decisions/DEC-014-the-shortcuts-pane-regroups-into-five-taxonomic-groups.md` — the taxonomy
- `.control/decisions/DEC-018-the-declared-shortcut-sequence-has-one-home-in-shared.md` — why this
  ticket crosses two crates
- `.constitution/project/codebase-stack-guide.md` — the stack, the commands, and how `.slint` markup
  relates to the Rust that binds it. It was corrected today; trust it over anything you remember about
  this crate using egui, which it has not for some time.

## Your job, stated as the only thing that counts

Seven tests are committed and **red**. Make them green by changing production code. That is the whole
definition of done for this step:

| Crate | Test |
|---|---|
| `settings` | `app::tests::shortcut_field_group_declares_five_taxonomic_groups` |
| `settings` | `app::tests::snap_maximize_now_sorts_after_every_snap_variant` |
| `settings` | `app::tests::snap_custom_labels_no_longer_repeat_the_group_name` |
| `settings` | `app::tests::the_declared_sequence_matches_the_shared_source` |
| `settings` | `app::tests::grouping_never_reorders_the_declared_sequence` |
| `settings` | `app::tests::field_declaration_order_includes_third_snap_fields` |
| `daemon` | `hook::tests::the_daemon_precedence_order_matches_the_shared_source` |

`shared::constants::SHORTCUT_DECLARED_ORDER` already holds the target order. It is the specification.
Read the order from it; do not retype it as a literal anywhere.

Work you will need, from the ticket's own checklist:

- `crates/settings/src/app.rs` — `ShortcutField::ALL`'s order, `group()`'s five arms, `label()` for the
  four `SnapPercent*` fields
- `crates/settings/src/main.rs` — the three `group_rows("...")` calls become five, wired to five
  `ShortcutRowData` model vectors instead of three. The names are yours to choose, consistent with the
  existing `rows_switching`/`rows_snap`/`rows_move`
- `crates/settings/ui/panes/shortcuts_pane.slint` — a `ShortcutGroup` per new group, heading text
  exactly matching `group()`'s strings, in declared order
- `crates/daemon/src/hook.rs` — `Chords::in_declared_order`, `load_shortcuts_from_config`'s row table, and the
  positional `Chords { ... resolved[N] }` mapping, **every index of which shifts**
- `dec_011_collision_favors_percent_snap_bottom_over_legacy_stack_default` in the same file — its
  literal indices and its inline 16-entry array move. Its **outcome must not change**: percent-snap
  still beats legacy stack. Only the numbers that express it move
- Grep for the literal `"(custom %)"` across markup and prose and fix stale quotations of the old label

## Five things that will fail your work if you do them

1. **Do not edit, weaken, rename, or delete any test to make it pass.** Not an assertion, not a
   guard, not a fixture. The two `..._matches_the_shared_source` tests exist because two declared
   orders silently disagreed; a test that cannot fail is worse than the bug. The one permitted test
   edit is the `DEC-011` fixture's *indices*, named above, with its outcome unchanged.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** If the code turns out right and a
   document wrong, say so in your report and stop there — the coordinator routes it. Do not absorb it
   as a code patch and do not edit the document yourself.
3. **Do not widen scope.** `SPEC-4-02` retires the Layout pane and moves `stack_width_percent`;
   `SPEC-4-03` does the tooltip, centring and scroll polish; `SPEC-4-04` fixes `DEF-5`. All three are
   blocked on this ticket. Leave `Pane::Layout` alone — it dies in `SPEC-4-02`, not here.
4. **Do not `git push`, do not open a PR, do not merge, do not touch `main`.** Commit to the branch you
   are on (`ticket/SPEC-4-01`) and stop. Pushing is the coordinator's hand.
5. **Do not guess at a failure.** When a test or build fails and you do not know why, run
   `wdi-systematic-debugging` before proposing a fix. A third failed fix attempt means escalate in your
   report, not try a fourth.

## Verification — run it, do not assume it

From the repository root of **this** worktree. `WIRADESK_SKIP_MANIFEST` is not optional: the daemon
links an elevation manifest that otherwise applies to the test harness, which then cannot launch at
all, and the failure looks like a broken toolchain.

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`CARGO_TARGET_DIR` is already set in your environment and points at the one shared, already-warm
target tree this run uses. **Do not change it, do not unset it, and do not pass `--target-dir`.** One
build location is an explicit instruction from the repo owner; a second target tree costs a full
dependency rebuild and risks two sessions racing.

The full workspace suite must be green — not only the seven tests above. A ticket that passes its own
tests and breaks a neighbour's has not finished. `undocumented_unsafe_blocks` and `missing_safety_doc`
are `deny` in the workspace lints, so any new `unsafe` block without a real `SAFETY:` comment is a
compile error; this ticket should need no new `unsafe` at all.

## Report back

Plain and honest, in this shape:

1. Which route you took for the engine — Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. The seven tests, each PASS or FAIL. Never round up.
3. `cargo fmt` / `cargo clippy` / `cargo test --workspace` — the actual final counts, including the
   suite total, not "all green".
4. Every file you changed, and for each, one line on why.
5. Anything you found that the ticket did not anticipate — a stale string, a document the code now
   contradicts, a test whose indices moved. Report it; do not fix beyond the ticket.
6. Anything you could not do, with the specific reason.

Your report is not evidence. The coordinator re-runs everything and reads the diff. Say what is true.
