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
| The row's title-and-description block | **none** |
| Each of the five `ShortcutGroup` headings (`Text { text: root.heading }`) | **none** |

Only the four percentage controls and the enable toggle (`"Enable " + root.title`) carry labels. So
no test can locate the keycap to compare its centre against the toggle's, and none can locate a
heading to measure the pane's width against.

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

## Out of scope, deliberately

- Anything covered by `DEF-5` (typed digits not committing via a real keystroke) — `SPEC-4-04`. This
  ticket may touch the same file but must not fold that defect's fix in as a side effect; if it is found
  and fixed here anyway, it still needs `SPEC-4-04`'s own dedicated test, not a note in this ticket.
