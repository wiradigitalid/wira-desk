---
id: SPEC-4-03
component: settings
satisfies: [UC-4, UC-11]
blocked_by: [SPEC-4-01, SPEC-4-02]
status: ready-for-agent
tests:
  - shortcut_row_slint_snapshot::tests::description_renders_as_a_tooltip_not_a_visible_line
  - shortcut_row_slint_snapshot::tests::control_cluster_and_toggle_share_one_vertical_centre
  - shortcuts_pane_slint_snapshot::tests::five_groups_fit_the_default_window_width_with_no_horizontal_scroll
  - shortcuts_pane_slint_snapshot::tests::all_five_group_headings_are_in_the_rendered_tree
---

# 03: Row polish — tooltip description, centred control cluster, no horizontal scroll

**What to build:** Three independent visual defects the owner found on the live DEC-016 build, all in
`crates/settings/ui/components/shortcut_row.slint` and `crates/settings/ui/panes/shortcuts_pane.slint`,
after `SPEC-4-01`/`SPEC-4-02` have already given every row its final five-group shape and its final
control set (a `Stack` row now carries a percent field same as a `Snap to custom` row). Read
`.how/settings/01-ux/DESIGN.md`'s "Row description as tooltip", "Row vertical centring", and "No horizontal
scroll" bullets before starting — they are this ticket's design source, written for exactly this work.

**Blocked by:** `SPEC-4-01`, `SPEC-4-02`

- [ ] `ShortcutRow`'s `description` text stops rendering as an always-visible, single-line, elided
      (`overflow: elide`) `Text` under the title. It becomes a tooltip shown on hover/focus of the row's
      title-and-description block, carrying the full, un-truncated description string every time.
- [ ] The tooltip is reachable by keyboard focus, not only mouse hover — a row reached by Tab must be able
      to surface its own description the same way a mouse hover does, consistent with this component's
      existing accessibility bar (`FR-20`, `FR-21`).
- [ ] Removing the always-visible description line does not change the row's declared height in a way that
      breaks the divider positioning this component's own header comment warns about (see the note at the
      top of `shortcut_row.slint` on why every text here is single-line) — re-verify the row height is still
      deterministic once the description is no longer part of it.
- [ ] The row's right-hand cluster — percentage control where present, keycap, and enable toggle — is
      wrapped so all three sit on one shared vertical centre against the row's own height, as one visually
      centred group rather than three independently positioned children. The observed defect is the toggle
      reading higher than the keycap; the fix must hold for both a one-line-tall row and a row whose keycap
      wraps to its minimum width.
- [ ] `ShortcutsPane` and its five `ShortcutGroup`s render at the shell's declared default width —
      `main_window.slint:44`'s `width: is_onboarding ? 580px : 760px`, i.e. `760px` for the normal
      (non-onboarding) window this pane lives in — with no horizontal scrollbar. Where the row's right-hand
      cluster is currently too wide for that width,
      narrow the cluster's own layout (e.g. tighter spacing between stepper/keycap/toggle) rather than
      letting the pane grow wider than its shell or scroll sideways — a Windows settings dialog scrolls
      vertically only, per this component's existing scroll-hint design.
- [ ] The existing vertical scroll behaviour (scroll hint veil, pinned Key check band) is unaffected — this
      ticket touches row width and cross-axis alignment, not the pane's own vertical scroll mechanics.
- [ ] Full test suite green once, not only this ticket's own tests.

## Amendment 1 — 2026-09-07, seam agreement (coordinator, mandate `DEC-017`)

**This ticket's Step 1 is a seam agreement, not a red suite, and that is a departure worth stating.**
Every previous ticket in this spec had its acceptance criteria encoded as failing tests before any
code was dispatched. Here that is not possible, and the reason is structural rather than a shortcut.

All three criteria are properties of the **rendered geometry and the accessible tree**. The only way
to reach either from a test is `i_slint_backend_testing::ElementHandle`, which finds elements *by
accessible label* and then exposes `size()` and `absolute_position()`. And today:

| What a test must find | Its accessible label |
|---|---|
| The keycap (`Rectangle`, `width: max(135px, lbl.width + 24px)`) | **none** |
| The row's title-and-description block | **none** — it does not exist yet |
| Each of the five `ShortcutGroup` headings (`Text { text: root.heading }`) | ~~none~~ **it already has one** |

**Corrected 2026-09-08, after the review panel caught it:** the heading row of that table was
wrong. Slint's compiler auto-assigns `accessible-role: text` and `accessible-label: self.text` to
every `Text` that does not set them itself (`lower_accessibility.rs`), so the five headings were
labelled all along and were findable. Only the keycap — a `Rectangle`, which gets no such
default — and the title block, which did not yet exist, were genuinely unreachable.

That does not change the conclusion (the two row-level criteria still could not be tested in
advance, and the heading test was allowed to start green), but the stated reason was wrong for a
third of the table, and a reader would have inherited the error.

**Making those elements reachable is this ticket's own work, not scaffolding for it.** The tooltip
criterion already requires it: *"The tooltip is reachable by keyboard focus, not only mouse hover"*
(`FR-20`, `FR-21`). A description reachable by focus is a description in the accessible tree. The
keycap and the headings follow from the same standard — a screen-reader user navigating this pane
should reach a chord and a group name, and today reaches neither.

So the three named tests are the **builder's**, written through `tdd` before the change they
describe, at the seam agreed here. That is `wdi-build` Step 2's own provision — `/implement` uses
`tdd` at the agreed seams — rather than an exception to it.

### The seam, agreed

- `crates/settings/src/shortcut_row_slint_snapshot.rs` for the two row-level tests, using the
  harness already in that file: `i-slint-backend-testing`'s `TestingBackend`, `run_on_ui_thread`,
  `setup_shortcuts_window`.
- A **new** `crates/settings/src/shortcuts_pane_slint_snapshot.rs` for the pane-level ones, declared
  beside the two existing `#[cfg(test)] mod` lines in `crates/settings/src/main.rs`.
- Geometry via `ElementHandle::absolute_position()` and `::size()`. Centres compared with an
  explicit tolerance, stated in the test, not an exact float equality.

### Two properties of the harness, established before you start

- **Accessible elements inside a scroll area exist in the tree only once scrolled into view.** The
  `SPEC-4-02` builder hit this authoring `every_rendered_percent_control_name_comes_from_theme` and
  worked around it by scrolling first. The no-horizontal-scroll test meets it head-on: a group below
  the fold is not absent because the layout is wrong, it is absent because nothing scrolled to it.
  Distinguish those two before concluding anything.
- **`main_window.slint:43` is the width source** — `width: is_onboarding ? 580px : 760px`. The
  criterion is the normal window, so **760px**. Read it from there rather than restating it.

### Added: the heading coverage `SPEC-4-01`'s panel deferred to here

`SPEC-4-01`'s review found that three of the five groups — *Snap to half*, *Snap to third*,
*Resize, move & arrange* — have no real-window check at all; only *Snap to custom* is covered, and
that incidentally, by a percentage test that happens to live in it. It was deferred precisely
because it belongs in the `shortcuts_pane_slint_snapshot` module this ticket creates, and creating
that file twice was the thing to avoid.

- [ ] `shortcuts_pane_slint_snapshot::tests::all_five_group_headings_are_in_the_rendered_tree`
      asserts each of `ShortcutField::GROUPS`' five headings is findable, iterating that constant
      rather than restating the five strings — `theme::ALL`'s lesson from `SPEC-4-02`, where a
      hand-listed array silently missed two entries.

### Mutation discipline, unchanged

Each of the four tests must be **seen red before the code that satisfies it**, and any that starts
green — the heading one may — must be broken deliberately, watched fail, and restored. Report which
mutation you ran for each. A guard never seen red is a claim, not proof, and this ticket has no
red suite handed to it, so that discipline is the only thing standing in for one.

## Amendment 2 — 2026-09-08, return trip 1 of 2 (`wdi-build` Step 3 panel)

The three visual defects are genuinely fixed in the markup — both axes agree, and the centring fix
was verified by reproducing its measurement independently. What the panel found is in **what proves
it**, plus one behavioural regression neither axis predicted and the coordinator confirmed by probe.

### The Tab-order regression — measured, not inferred

The two axes disagreed. Standards reasoned that `main_window.slint`'s `key-pressed` handler ends in
`accept` for every key, so Tab could not arrive at all. Spec found that `FocusScope` defaults to
`focus-on-tab-navigation: true` and was not overridden, so all sixteen rows gain an undeclared Tab
stop. A throwaway probe dispatching real `WindowEvent::KeyPressed { Key::Tab }` settled it:

```
PROBE: tooltip surfaced after Some(1) Tab presses
```

**Tab traverses fine — Standards' inference was wrong — and the title block is reachable in ONE
press from a fresh window.** So it is not merely an extra stop appended to the order; it is at or
near the front of it. `LBR-ST-5` (`status: active`,
`.what/settings/02-rules/rules-settings.md`) requires *"a deterministic Tab navigation order that
starts with navigation tabs and terminates with action buttons."* Tab from a fresh Settings window
now lands on a row's description block instead of the navigation sidebar. That is user-visible.

- [ ] **Root-cause it with `wdi-systematic-debugging` before changing anything.** Why one press
      reaches it is not yet known, and the two obvious remedies pull opposite ways: suppressing the
      stop (`focus-on-tab-navigation: false`) protects `LBR-ST-5` but may cost the keyboard half of
      `FR-20`/`FR-21` that criterion 2 requires; declaring the stop in `app::focus_order` keeps the
      feature but must place it where `LBR-ST-5` says, not first. Do not pick one blind.
- [ ] Whichever holds, `crates/settings/src/app.rs`'s `focus_order` must end up **true**. It was
      not updated, and `focus_order_mismatch` only ever compares `focus_order`'s output to itself,
      so nothing was going to catch it. Note honestly that the declaration was already fictional
      before this ticket — no rendered element carries `f.label()` as an accessible name — so this
      is drift into an existing hole rather than a live guard broken. It is still the hole this
      criterion walked into.
- [ ] Prove the final order with a **real `Key::Tab`** sequence, not
      `invoke_accessible_default_action()`. That call fires the
      `accessible-action-default => { self.focus(); }` hook added at `shortcut_row.slint:76-78`,
      which exists for no production purpose — it is there so the test can focus the block. That is
      test-driven API in the component and it goes with the fix.

**Why this one matters beyond itself:** proving a focus path through the accessibility harness
instead of a real keystroke is `DEF-5`'s exact mechanism, recurring inside the ticket series opened
to fix `DEF-5`'s siblings — and the counter-technique already sat 300 lines above in the same file.

### Seven constants, no consumers

Both axes, and the coordinator, independently. `theme.rs` gained seven `ControlSemantics`; six have
zero readers anywhere and the seventh is read only by a test — which makes the constant a fixture
that must match markup rather than a declaration markup obeys, the coupling backwards. The strings
appear a third and fourth time as raw literals in the new tests.

- [ ] **Delete the five `GROUP_HEADING_*` constants.** `ShortcutField::GROUPS` is already the one
      home for those five strings, `shortcuts_pane.slint` renders them through `root.heading`, and
      `app.rs` already guards `GROUPS` against `group()`. A second unenforced copy of a
      single-source array is what `DEC-018` exists to prevent.
- [ ] Keep `SHORTCUT_KEYCAP` and `SHORTCUT_ROW_DESCRIPTION`, but **consume them the way this file
      already consumes the other four** — an `in property` on `ShortcutRow`, filled from Rust in
      `main.rs` — instead of literals in markup. That plumbing exists eleven lines away.

### Sixteen controls, two names

- [ ] Both new labels are row-invariant, so the tree carries sixteen elements named
      `"Shortcut keycap"` and sixteen named `"Shortcut description"`, one of them on a `FocusScope`
      and one on an `accessible-role: button`. That is `DEF-2`'s literal shape and a new instance of
      `DEF-6`'s class, citing the same `FR-20`/`FR-21` criterion 2 invokes. Eleven lines below,
      `"Enable " + root.title` does it correctly. Make them per-row.
      `accessible_names_are_unique` cannot see this: it iterates `ALL`, not the rendered tree.
- [ ] `min-height: 50px` on the row is **inert** — the row's natural height is already 50px
      (30px cluster + 10px + 10px padding), so deleting the line changes no pixel and breaks no
      test. Either give it a real assertion or remove it. Do not leave a line that looks like a
      guarantee and guarantees nothing.

### Landed by the coordinator in this trip

Recorded here so the builder does not redo them, and because three are the coordinator's own errors:

- [x] Amendment 1's premise about the heading labels was wrong; corrected above.
- [x] `five_groups_fit_the_default_window_width_with_no_horizontal_scroll` could pass finding
      nothing — proven by emptying the heading labels, then closed (`b28d5ab`).
- [ ] Its bound is still the **window** edge (760px), not the pane's content edge (~740px after the
      175px sidebar and 20px padding), so up to ~36px of real overflow passes. Tighten it, and run a
      **geometry** mutation — widening the cluster — because every mutation so far proved only that
      the test detects *missing elements*, never *actual width*. Nothing was narrowed for defect 3
      and the width assertion has never been observed failing.
- [ ] The height guard measures two **title blocks**, not two rows, and after this change they are
      structurally identical by construction — it cannot go red for the property it names. Rewrite
      it to measure real row heights across two shapes: a one-line row and a row carrying the
      conflict or "Disabled" line, which is the shape `shortcut_row.slint`'s own comment says broke
      historically.
- [ ] `scroll_by(window, 1200.0)` is "scroll up by an amount currently believed to exceed the
      extent" wearing the name of a primitive. `SPEC-4` is adding rows. Name it with its reason or
      make it saturating, unify the two divergent scroll ladders in one file, and share the helper
      into `shortcut_row_slint_snapshot.rs`, which still inlines raw dispatches — the commit message
      claimed a reach it did not have.
- [ ] The measured counters double-count, because the outer loop revisits scroll positions. Harmless
      for `> 0`, misleading as coverage.
- [ ] `shortcut_row.slint`'s header comment and the note above the cluster both still describe the
      always-visible wrapping description. Both are now false in the direction that matters: a
      reader repairing a badly-wrapping tooltip would read them as licence to make it single-line,
      undoing defect 1.
- [ ] `CHANGELOG.md` `## [Unreleased]` — this adds behaviour and new files, so it fails the patch
      test twice, and it is user-facing in exactly the way that section's preamble describes.
- [ ] `3p.md` overstates two things after `b28d5ab` (the width test now proves `>= 1` keycap and
      toggle, not "all"; row height is a 50px floor, not a deterministic 50px), carries no entry for
      `b28d5ab`, and records no per-test mutation report though the brief required one.

### Recorded, not fixed

- The tooltip is placed at `y: root.height + 2px` — **outside the row's own bounds**. On the last
  visible row the ScrollView clips it; elsewhere it overdraws the next row's divider, which is the
  exact failure the header comment was written about. `z: 100` orders only within one row's
  children, so it cannot win against a later sibling row. No criterion covers placement and the
  testing backend renders nothing, so this cannot be proven here — it is a hand-verifiable risk and
  belongs in the smoke test rather than in a tree-presence assertion.
- `shortcut_row_slint_snapshot.rs`'s remaining `if let Some(save_btn)` silent-skip, the same idiom
  `b28d5ab` condemned three tests away.
- An `eprintln!("MEASUREMENT: …")` ships unconditionally rather than only on failure.
- `verify-public-export.ps1` does not scan `.slint` at all — `$textExtensions` omits it — so the two
  markup files in this ticket were never checked by the publication gate. Pre-existing, out of
  scope, and worth the owner knowing.
- The centring test covers one-line rows with and without a stepper, not the two-line shape the
  ticket named. Low risk (keycap height is fixed at 30px) but it is the shape with a history.

## Out of scope, deliberately

- Anything covered by `DEF-5` (typed digits not committing via a real keystroke) — `SPEC-4-04`. This
  ticket may touch the same file but must not fold that defect's fix in as a side effect; if it is found
  and fixed here anyway, it still needs `SPEC-4-04`'s own dedicated test, not a note in this ticket.
