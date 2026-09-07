# Builder brief — `SPEC-4-02`, return trip 2 of 2 (mandate `DEC-017`)

**One change, four lines.** The re-panel found no must-fix in your mechanism: identity-from-range is
genuinely gone, all eight names are live and rendered, the Stack tests select by identity, and no
assertion was weakened. Everything else the coordinator has already landed. Read `## Amendment 4`
in the ticket — the single unchecked box at the end is your whole scope.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. Repo copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and
carry out its process, and say which you did. No `bmad-*` skill.

## The work

`crates/settings/src/main.rs` — the label match you added. It reads:

```rust
match field {
    ShortcutField::Stack => ( /* STACK_WIDTH_* */ ),
    _ => ( /* SNAP_PERCENT_* */ ),
}
```

The `_` arm covers the four `SnapPercent*` fields **and** the eleven fields that have no percentage
at all. So the snap family is assigned by "not Stack" rather than by identity.

- [ ] Enumerate `ShortcutField::SnapPercentLeft | SnapPercentRight | SnapPercentTop |
      SnapPercentBottom` for the snap family, and keep `_` for the remaining, non-percent fields.

**Why this is a must-fix and not a tidy-up.** Every sibling percent match on this enum already does
it that way — `has_percent`, `percent_bounds`, `percent` and `set_percent` each list the four
explicitly and reserve `_` for the no-percent rest (all in `crates/settings/src/app.rs`). And
`3p.md`'s entry for this very ticket credits the removal of a `_ => {}` catch-all as the fault the
ticket was caught on; a new one now sits beside that sentence. A future sixth percentage field
would silently inherit snap labels, and the guard cannot see it — a valid snap-family assignment
passes.

That is the whole change. Do not restructure the match into a method, do not touch `app.rs`, and do
not touch `theme.rs` — the coordinator rewrote its two register tests in this trip and they are
verified by mutation.

## What will fail your work

1. **Do not edit, weaken, rename, or delete any test.** Several are mutation-verified. In
   particular `theme::tests::accessible_names_are_unique`,
   `theme::tests::every_rendered_percent_control_name_comes_from_theme` and
   `theme::tests::listening_state_has_a_spoken_announcement` are all load-bearing and were all
   confirmed able to fail.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.**
3. **Do not widen scope.** `SPEC-4-03` owns tooltip/centring/scroll; `SPEC-4-04` owns `DEF-5`; the
   newly filed `DEF-6` (all four snap rows share one accessible-name set) is explicitly **not**
   this ticket's — do not give the four snap rows per-direction names.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-02` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.**
6. **Do not guess at a failure.** Unknown cause → `wdi-systematic-debugging` first.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`CARGO_TARGET_DIR` is already set to the one shared, warm target tree. Do not change it, unset it,
or pass `--target-dir`.

The suite stands at **548 passed / 0 failed / 2 ignored**. Expect the same count — this change adds
no test and removes none. **A different count is a finding, and you should report it rather than
explain it away.**

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. `cargo fmt` / `clippy` / `cargo test --workspace` — actual final counts.
3. The exact match arms before and after, so the diff can be checked against the ask.
4. Confirm no test file is in your diff at all.
5. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
