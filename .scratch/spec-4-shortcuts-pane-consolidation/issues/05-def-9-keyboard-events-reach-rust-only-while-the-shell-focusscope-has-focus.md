---
id: SPEC-4-05
component: settings
satisfies: [UC-4]
blocked_by: [SPEC-4-03]
status: ready-for-agent
tests:
  - shortcut_row_slint_snapshot::tests::description_renders_as_a_tooltip_not_a_visible_line
---

# 05: Defect DEF-9 — keyboard events reach Rust only while the shell's own FocusScope has focus

**What to build:** Fix `DEF-9`. `crates/settings/ui/main_window.slint` declares
`key_handler := FocusScope` (line 149) as a **sibling** of the outer `Rectangle` that holds every
pane (line 179), and `key_handler` has no children. Slint delivers a key event to the focused
element and bubbles it up that element's **ancestors**, so a sibling subtree never sees it.
`root.key_pressed_event(...)` therefore fires only while `key_handler` itself holds focus — and it
is the only entry point to the Key Check diagnostic, Escape-cancels-capture,
`CaptureState::Listening` chord capture, and onboarding step 2. Read `DEF-9` in full first.

**Why this is its own ticket rather than a third `SPEC-4-03` trip.** The nesting and the
`pct_input.focus()` route into it both predate that ticket — verified at `0c6f405~1` — so this is
not a regression it introduced, and by `wdi-build`'s own classification a pre-existing defect the
diff does not touch is a follow-up rather than a must-fix. What that ticket did change is how
easily you land here: one Tab press now moves focus off `key_handler`, by design, as its criterion
2 requires. Its two return trips are spent, so this is where the fix belongs.

**Ordered before `SPEC-4-04`.** Both tickets are about keyboard input in this window, and this one
changes which element receives a key. `DEF-5`'s own candidate list includes "whether the click
actually lands keyboard focus on `pct_input`", so a root-cause pass run before this lands would be
run against a focus tree that is about to change, and may have to be redone. That is a sequencing
judgement, not a claim about `DEF-5`'s cause.

- [ ] **Root-cause with `wdi-systematic-debugging` before writing the fix.** The structural reading
      above is the coordinator's, from the brace depths and Slint's routing rules — it is not a
      run. Confirm the mechanism yourself before changing anything, and say what you confirmed it
      with. In particular establish whether an unconsumed key from a focused `title_block` or
      `pct_input` reaches `key_handler` today at all.
- [ ] Nest the outer `Rectangle` **inside** `key_handler` so every unconsumed key bubbles up to
      it. `forward-focus: key_handler` stays, so a fresh window still starts with focus there.
- [ ] `SPEC-4-03`'s Tab traversal MUST survive:
      `shortcut_row_slint_snapshot::tests::description_renders_as_a_tooltip_not_a_visible_line`
      stays green, and reverting your change MUST still red it — that test is mutation-verified and
      it is the one thing standing between this fix and re-breaking criterion 2.
- [ ] The `listening_field == -1` guard MUST keep working: a Tab pressed while a row is capturing is
      recorded as part of the chord and does not move focus. With `key_handler` as an ancestor
      rather than a sibling this condition is now reachable from a focused row, which is the whole
      point, so re-read it rather than assuming it still means what it did.
- [ ] Escape MUST cancel a capture from a state where focus has already moved — press Tab, click a
      keycap, press Escape. That is the path `DEF-9` describes and the one no test can see.
- [ ] Say plainly which of these you could verify automatically and which you could not.
      `DEF-8` records why: `on_key_pressed_event` is registered in `main()`, not in
      `bind_callbacks`, so the callback is unset in every test and a forwarded call is a silent
      no-op there. **MUST NOT** fake a guard through `invoke_accessible_default_action()` or
      `set_accessible_value` to get around that — that is `DEF-5`'s own mechanism and a returnable
      finding in itself.
- [ ] Full test suite green once, not only this ticket's own tests.

## The smoke steps this ticket owns

`SPEC-4-03`'s Amendment 4 wrote two smoke steps for the Tab/`DEC-005` fix, and both start from a
**fresh window** — which is the one state that already worked. They cannot see `DEF-9`. Replace
them with these, each starting from a state where focus has already moved:

1. Open Settings, press **Tab once**, then click a keycap and press `Ctrl+Alt+P`. The chord must be
   recorded. Then start another capture and press **Escape** — it must cancel.
2. Open Settings, click a **percentage field** (the pre-existing route), then click a keycap and
   press a chord. Same expectation.
3. With nothing capturing, press `Ctrl+Alt+Tab` after a Tab press and read the Key Check band: it
   must report the window as having seen the key, never a third-party claim.

## Out of scope, deliberately

- `DEF-7` — no `SidebarItem` and neither action button is focusable, so the shell has no Tab order
  at all. Shell-wide, and `focus_order` is named there.
- `DEF-8` — moving the keyboard registration into `bind_callbacks` so any of this can be tested.
  Do it there, not here; this ticket must not grow into a test-seam refactor.
- `DEF-5` — `SPEC-4-04`'s, and it is ordered after this one.
