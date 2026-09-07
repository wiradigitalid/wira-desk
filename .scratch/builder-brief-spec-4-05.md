# Builder brief — `SPEC-4-05`, `wdi-build` Step 2 (mandate `DEC-017`)

One defect, `DEF-9`, and it is the most severe thing this spec has surfaced: after one Tab press in
Settings, a row you click to rebind sits in "Listening…" forever and Escape cannot cancel it.

## Run the engine

Invoke **`/implement`** with this brief and the ticket. The repo's copy at
`.claude/skills/implement/SKILL.md` is model-invocable; if the Skill tool refuses, read it and carry
out its process, and say which you did. No `bmad-*` skill.

## Read first, in this order

1. `.scratch/spec-4-shortcuts-pane-consolidation/issues/05-def-9-keyboard-events-reach-rust-only-while-the-shell-focusscope-has-focus.md`
   — the ticket. Its checkboxes are your acceptance criteria and it carries the smoke steps.
2. `.control/registry/defects.yaml`, the **`DEF-9`** row in full — including `why_it_hid`, which
   explains why nothing caught this, and `DEF-8` beside it, which explains why no test can.
3. `.control/decisions/DEC-005-key-check-reports-observations-never-predictions.md` — `applied`.
   Its four-row table is what breaks when the window's half of the correlation goes missing.
4. `.scratch/spec-4-shortcuts-pane-consolidation/issues/03-row-polish-tooltip-centring-no-scroll.md`
   — **Amendment 5 only**, which is where this defect was found and classified, and why it is a
   ticket of its own rather than a third trip on that one.

## The shape of it

`crates/settings/ui/main_window.slint:149` declares `key_handler := FocusScope` as a **sibling** of
the outer `Rectangle` at `:179` that holds every pane — both direct children of the `Window` — and
`key_handler` has no children of its own. Slint delivers a key to the focused element and bubbles it
up that element's **ancestors**, so a sibling subtree never receives it.

`forward-focus: key_handler` at `:147` is why this works at all today: a freshly opened window has
focus there. The moment anything else takes focus — `pct_input.focus()` when you click a percentage
field, or, since `SPEC-4-03`, one Tab press — `root.key_pressed_event(...)` stops firing, and that
callback is the only entry point to four behaviours: the Key Check diagnostic, Escape-cancels-
capture, `CaptureState::Listening` chord capture, and onboarding step 2's Win+backtick.

**Root-cause it before you change anything.** The reading above is the coordinator's, from brace
depths and Slint's routing rules — it is not a run. `wdi-systematic-debugging` first, and say what
you confirmed the mechanism with. Establish in particular whether an unconsumed key from a focused
`title_block` or `pct_input` reaches `key_handler` today at all.

The remedy the ticket names is to nest the outer `Rectangle` **inside** `key_handler`. If your
root-cause pass finds a better one, say so and why — but do not pick one blind, and do not widen
into `DEF-7`'s territory (making sidebar items and action buttons focusable) or `DEF-8`'s (moving
the keyboard registration into `bind_callbacks`).

## Two things that must not break

- **`SPEC-4-03`'s Tab traversal.**
  `shortcut_row_slint_snapshot::tests::description_renders_as_a_tooltip_not_a_visible_line` is
  mutation-verified: reverting the `reject` lines reds it at line 554. It must stay green, and it
  must still red when reverted. It is the only thing standing between this fix and re-breaking that
  ticket's criterion 2.
- **The `listening_field == -1` guard** at `main_window.slint:158` and `:171`. With `key_handler` as
  an ancestor rather than a sibling, that condition becomes reachable from a focused row for the
  first time — which is the point of the fix. Re-read it rather than assuming it still means what it
  meant when nothing but `key_handler` could reach it.

## No automated test for the new behaviour, and do not fake one

`DEF-8`: `on_key_pressed_event` is registered inside `main()` (from line 721), while
`bind_callbacks` ends at line 473 — and every test builds its window through `bind_callbacks`, so
the callback is unset and a forwarded call is a silent no-op in the suite.

**MUST NOT** reach for `invoke_accessible_default_action()` or `set_accessible_value` to manufacture
a guard. That is `DEF-5`'s exact mechanism — proving the automation path while the keystroke path
stays broken — and it is a returnable finding in itself. Verification of the new behaviour is the
ticket's three smoke steps, which run at the end of the run and not from this dispatch.

If you find a way to reach `key_handler` from a `#[test]` that does **not** go through the
accessible layer, report it with the measurement. That would be a real finding and would unblock
`DEF-8`, but do not spend this ticket on it.

## What will fail your work

1. **Do not weaken, rename, or delete any test.** Several in `shortcut_row_slint_snapshot.rs`,
   `shortcuts_pane_slint_snapshot.rs`, `theme.rs` and `app.rs` are mutation-verified.
2. **Do not touch `.what/`, `.how/`, or any `applied` `DEC-`.** `DEC-005` is the constraint here.
   A deviation is reported and becomes a `DEC-`, never a code patch.
3. **Do not widen scope.** `DEF-7`, `DEF-8`, `DEF-10` and `DEF-5` are all filed and none is yours.
4. **Do not `git push`, open a PR, merge, or touch `main`.** Commit to `ticket/SPEC-4-05` and stop.
5. **Do not `git checkout`, `git restore`, `git stash`, or `git reset` the working tree.**
6. **Do not change `CARGO_TARGET_DIR` or pass `--target-dir`** — you hold the build lock alone.
7. **Do not guess at a failure.** Unknown cause means `wdi-systematic-debugging` before any fix. A
   third failed attempt means escalate in your report, not try a fourth.

## Verification — run it

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

The suite stands at **554 passed / 0 failed / 2 ignored**. If your change moves that number, say
why.

## Report back

1. Engine route: Skill-invoked `/implement`, or read-and-followed its `SKILL.md`.
2. **The root cause from `wdi-systematic-debugging`** — what you confirmed, and how. If the
   coordinator's structural reading was wrong in any part, say which part.
3. The final `key_handler` block quoted, so the nesting is visible without reading the diff.
4. That the tooltip test still passes, **and** that reverting your change still reds it.
5. `cargo fmt` / `clippy` / `cargo test --workspace --no-fail-fast` — actual final counts.
6. Which of the ticket's criteria you could verify automatically and which you could not.
7. Every file changed, one line each on why.
8. Anything you could not do, with the specific reason.

Your report is not evidence. The diff is re-read and every command re-run. Say what is true.
