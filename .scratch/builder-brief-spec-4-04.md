# Builder brief — `SPEC-4-04`, `wdi-build` Step 2 (mandate `DEC-017`)

You are the builder for one ticket: **the only defect fix in `SPEC-4`**, everything else in this
spec is a design change. A coordinator reviews your diff and holds every merge, push, and registry
write.

## What is already done, and what you must not redo

**The failing tests are handed to you.** Unlike `SPEC-4-03`, Step 1 is closed before you start: the
coordinator wrote two tests that encode `DEF-5`'s acceptance criteria and has seen them red. They
are in `crates/settings/src/shortcut_row_slint_snapshot.rs`:

| Test | Encodes |
|---|---|
| `a_real_keystroke_sequence_commits_a_typed_percentage` | digits typed as real character events land in the field and commit on departure |
| `a_real_keystroke_sequence_out_of_range_is_refused` | the refusal half — a typed out-of-range value is refused with an actionable error, never clamped, never saved |

**Your job is to make them pass by fixing the product, not by changing them.** The second one exists
because a fix that makes digits land could quietly drop the refusal and still look correct.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. The repo's copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and carry
out its process, and say which you did. No `bmad-*` skill — thirteen are retired at this gate.

## Read first, in this order

1. `.control/registry/defects.yaml`, the `DEF-5` row **in full** — including `why_it_hid` and
   `fix_direction`. It deliberately names no root cause, and neither should you until you have one.
2. `.scratch/spec-4-shortcuts-pane-consolidation/issues/04-defect-def-5-typed-digits-not-committing.md`
   — the ticket, whose first checkbox is the root-cause pass and lists five candidates to rule in or
   out. That list is candidates, **not a prescription of which is right**.
3. `.constitution/project/codebase-stack-guide.md` — the commands, and how `.slint` markup relates
   to the Rust that binds it.

## The one rule this ticket exists to respect

**`wdi-systematic-debugging` BEFORE any fix, without exception.** This is a bug with an unknown
cause, which is precisely the case this repo's standing rule names. Do not patch `input-type:
number`, do not reorder `pct_field_touch` against the `TextInput`, do not touch the daemon's
keyboard hook, until the root-cause pass says which one it is. A fix that makes the test pass
without explaining the defect is the thing this ticket must not produce — `DEF-5` was invisible for
a whole spec because every test drove the field through the path that already worked.

Report the root cause as a finding in its own right, whatever it turns out to be. If it is in the
daemon's global low-level keyboard hook affecting a same-process, non-chord keystroke, **stop and
report** — that is a significant, differently-owned finding and not this ticket's to fix.

## What will fail your work

1. **Do not weaken, rename, or delete the existing accessible-value tests.**
   `typed_percentage_commits_on_save_click` and its four siblings cover the automation and
   assistive-technology path, which this ticket does not touch. They stay green and unchanged.
2. **Do not weaken the two handed-down tests to make them pass.** If you believe one of them asserts
   the wrong thing, say so and stop; changing a guard to go green is the one thing no mandate
   reaches.
3. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** A deviation is reported, and it
   becomes a `DEC-` — never a code patch.
4. **Do not widen scope.** `DEF-6` (the four snap rows sharing one accessible-name set) is filed and
   is not yours. Neither is anything left open on `SPEC-4-03`.
5. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-04` and stop.
6. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.**
7. **Do not change `CARGO_TARGET_DIR` or pass `--target-dir`** — one build location is the repo
   owner's explicit instruction, and while you hold this dispatch you hold the build lock alone.
8. **No smoke test.** Verifying this live under a real daemon-attached elevated launch is a separate
   step; say plainly that only standalone mode was verified automatically, rather than implying both.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

`--no-fail-fast` is not optional.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. **The root cause, from `wdi-systematic-debugging`** — what actually stopped a keystroke reaching
   `typed_text`, and how you established it. Which of the ticket's five candidates you ruled out,
   and how.
3. Both handed-down tests passing, and the five accessible-value tests still green **and unchanged**
   — say the counts.
4. `cargo fmt` / `clippy` / `cargo test --workspace --no-fail-fast` — actual final counts.
5. Whether the fix also applies to the relocated `Stack`-row percent control, and if so that the
   same real-keystroke test covers it there.
6. Every file changed, one line each on why.
7. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.

---

## What the coordinator established before dispatching this

Both dependency checks from the draft are closed, and one of them is a finding you should have:

- `SPEC-4-03` and `SPEC-4-05` are both merged. Neither moved `"Snap percentage field"` or
  `"Snap percentage input"`, so the two handed-down tests find their elements and fail on `DEF-5`
  itself rather than at `.expect(...)`.
- **`SPEC-4-05`'s focus-tree change did NOT fix `DEF-5`.** That ticket nested the outer container
  inside `key_handler`, so an unconsumed key now bubbles from `pct_input` up to the window's
  `FocusScope` — a real change to keyboard routing in exactly this neighbourhood. The tests were
  written before it and re-run after it: still red, still `field reads "50" after typing 70`. So
  the cause is not that the digits had nowhere to go. **Rule that branch out and spend your search
  elsewhere** — the ticket's candidate list still has four other entries.
