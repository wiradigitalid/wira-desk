# Builder brief — `SPEC-4-03` **return trip 2 of 2**, `wdi-build` Step 2 (mandate `DEC-017`)

**This is the cap.** `wdi-build` allows two return trips and this is the second, so a result that
still carries an unresolved must-fix does not come back a third time — the ticket is recorded as
blocked and the run moves on. One item, and it is about twelve lines.

Trip 1 is **accepted**. The coordinator verified it from the diff and by re-running everything, and
also broke it deliberately to check the guards were real: dropping the per-row label plumbing reds
four tests, and reverting your two `reject` lines reds the tooltip test at
`shortcut_row_slint_snapshot.rs:554`. The trap the last brief warned about did not materialise.
**Your root cause on item 1 was right, and the panel's framing of it was wrong** — the ticket's
Amendment 4 retracts the coordinator's "Tab-order regression" claim, because the probe behind it was
measuring the testing backend. Read Amendment 4 before you start.

## Read first

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/03-row-polish-tooltip-centring-no-scroll.md`
   — **Amendment 4**, the whole of it. It carries the verification, the retraction, and this
   must-fix with its remedy.
2. `.control/decisions/DEC-005-key-check-reports-observations-never-predictions.md` — `status:
   applied`, and the reason this is a must-fix rather than a nitpick. The four-row table is the
   point.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. The repo's copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and carry
out its process, and say which you did. No `bmad-*` skill.

## The one item

`key_handler` in `crates/settings/ui/main_window.slint` returns `reject` for `Tab`/`Backtab`
**before** calling `root.key_pressed_event(...)`, and the same in `key-released`. The Rust side
therefore never sees Tab, and two things depended on it.

**1. `DEC-005` is contradicted.** The key check is a correlation of *what the daemon's hook saw*
against *what the Settings window saw*, and `DEC-005`'s table is the only set of claims it may make.
The window still receives the Tab event — the markup drops it before reporting it — so the
"window saw" signal is now **false**. Press `Ctrl+Alt+Tab` with the daemon running and the check
reads hook yes / window no, which that table defines as *"Another application claimed it, but Wira
Desk's hook receives it first"*. That is a fabricated diagnosis about an application that does not
exist, and inventing a verdict out of a broken signal is precisely what `DEC-005` was written to
refuse.

**2. A chord containing Tab can no longer be captured.** `map_slint_key` maps `"\t"` and `U+0009` to
`"tab"` in two separate deliberate arms, `app.rs`'s token map renders it `"Tab"`, and the
`CaptureState::Listening` branch pushes it into the combo. Tab is a supported chord key by
construction, and `Ctrl+Alt+Tab` was bindable before trip 1.

### The remedy

Forward the event **first**, then reject only when nothing is capturing. `MainWindow` already
carries `in property <int> listening_field: -1`, so the markup can read it with no new plumbing:

```slint
key-pressed(event) => {
    root.key_pressed_event(
        event.text,
        event.modifiers.control,
        /* ... exactly the arguments it passes today ... */
    );
    if (event.text == Key.Tab || event.text == Key.Backtab) && root.listening_field == -1 {
        return reject;
    }
    accept
}
```

Same shape in `key-released`. Two things to get right:

- **Order matters.** The forward must happen before the `reject` returns, or nothing is fixed.
- **The capture guard matters.** Without `listening_field == -1`, pressing Tab mid-capture would
  both record the chord and move focus out of the row.

Forwarding Tab when nothing is listening is safe, and it is worth knowing why rather than trusting
it: the callback's only branches after the key-check observer are Escape, the `Listening` capture,
and onboarding step 2's Win+backtick. Tab falls through all three and reaches nothing else.

## No test for this, and that is deliberate — do not invent one

`key_handler` **never receives keys in the testing backend.** Measured: a probe dispatching a plain
`'b'` leaves `key_check.last_display` empty, with and without
`WindowEvent::WindowActiveChanged(true)`. So there is no honest automated guard for this path, and
`wdi-build`'s rule is that a criterion no test can express is reported rather than faked.

**MUST NOT** reach for the accessible layer to fake one. Driving this through
`invoke_accessible_default_action()` or `set_accessible_value` would be `DEF-5`'s exact mechanism —
a test that proves the automation path while the keystroke path stays broken — inside the ticket
series opened to fix `DEF-5`'s siblings. A test that cannot fail is a returnable finding in itself.

It goes on the smoke-test list instead, and the coordinator has already written the two steps into
Amendment 4. If you find a way to reach `key_handler` from a `#[test]` that does **not** go through
the accessible layer, say so in your report and show the measurement — that would be a real finding
and worth its own ticket, but do not spend this trip on it.

## What will fail your work

1. **Do not weaken, rename, or delete any test.** Four guards in this ticket are mutation-verified,
   two of them by the coordinator this iteration.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** `DEC-005` is the constraint here,
   not the thing to edit.
3. **Do not widen scope.** `DEF-7` (no sidebar item or action button is focusable, so the shell has
   no Tab order at all) is filed and is explicitly **not** yours — it is a shell-wide change and
   escalating it was the right call. `DEF-5` is `SPEC-4-04`'s. `DEF-6` is nobody's yet.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-03` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.**
6. **Do not change `CARGO_TARGET_DIR` or pass `--target-dir`** — you hold the build lock alone.
7. **No smoke test.** It happens once, at the end of the run, and not from this dispatch.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

The suite stands at **554 passed / 0 failed / 2 ignored**. This change should not move that number:
there is no test for the path, and every other test must stay green. If the count changes, say why.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. The final `key-pressed` and `key-released` handlers, quoted, so the order of forward-then-reject
   is visible without reading the diff.
3. That the tooltip test still passes — Tab must still traverse when nothing is capturing. If it
   went red, the `reject` is no longer reached and the fix is wrong.
4. `cargo fmt` / `clippy` / `cargo test --workspace --no-fail-fast` — actual final counts.
5. Every file changed, one line each on why.
6. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
