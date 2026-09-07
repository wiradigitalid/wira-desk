---
id: SPEC-4-02
component: settings
satisfies: [UC-4, UC-9]
blocked_by: [SPEC-4-01]
status: done
tests:
  - app::tests::pane_enum_no_longer_declares_layout
  - app::tests::overlapping_stack_row_has_percent_true
  - shortcut_row_slint_snapshot::tests::stack_row_percent_commits_on_save_click
  - persistence::tests::stack_width_percent_round_trips_through_the_shortcut_row_path
  - app::tests::pane_declaration_order_is_the_navigation_index
  - app::tests::every_pane_index_round_trips_through_the_ui_boundary
  - app::tests::the_stack_row_carries_its_own_percent_bounds
  - app::tests::every_percentage_row_reaches_all_four_percent_seams
  - theme::tests::every_rendered_percent_control_name_comes_from_theme
---

# 02: Layout pane retired; stack width percentage moves onto the Overlapping Stack row

**What to build:** `crates/shared/src/config.rs`'s `layout.stack_width_percent` — currently edited only in
`crates/settings/ui/panes/layout_pane.slint`, which SPEC-4-01 leaves as the sole reason `Pane::Layout`
still exists — moves onto the `Stack` (Overlapping Stack) row in the Shortcuts pane's "Resize, move &
arrange" group, using the same `has_percent`/`percent`/`percent_changed`/`editing_changed` plumbing
`ShortcutRow` already gives every `Snap to custom` row. The `Layout` pane, its nav entry, and
`layout_pane.slint` are removed entirely. Read `DEC-014`'s accepted extension paragraph (the one starting
"Extension accepted the same day...") before starting; it names this exact change and its one open cost.

**Blocked by:** `SPEC-4-01` (the `Stack` row must already sit in its final "Resize, move & arrange" group)

- [x] `ShortcutField::has_percent()` returns `true` for `Stack`, alongside the four `SnapPercent*` fields.
- [x] `ShortcutField::get_percent`/`set_percent` (or their equivalent for `Stack`) read and write
      `cfg.layout.stack_width_percent` — the existing field, unrenamed; nothing here is a config migration.
- [x] `crates/settings/ui/components/shortcut_row.slint`'s `step_plus`/`step_minus` currently hardcode their
      bounds as literals (`base >= 1 && base < 99`) — correct for every `Snap to custom` row, wrong for `Stack`,
      which needs 10–100 (`layout_pane.slint`'s existing `min_percent`/`max_percent`). `ShortcutRow` gains
      per-row bound properties (e.g. `in property <int> percent_min: 1; in property <int> percent_max:
      99;`, overridden to `10`/`100` on the `Stack` row only) rather than a second hardcoded literal pair —
      setting `has_percent: true` on `Stack` without this change silently gives it the wrong range.
- [x] `Pane` (wherever it is declared — `crates/settings/src/app.rs` and/or the Slint side) drops `Layout`.
      Every place keyed on `Pane::Layout` — focus order, theme accessible-name tables, sidebar nav data —
      is updated in the same commit, not left as a dangling reference to a removed variant. `DEC-014`'s Cost
      section names this risk explicitly: grep for `Pane::Layout` and `layout_pane` and account for every
      hit, do not rely on the compiler to find all of them (a `&str` pane key or a UIA accessible-name table
      entry will not fail to compile). This includes any hardcoded pane-COUNT, not only named references —
      a sidebar nav-items array or test asserting exactly five entries is stale the moment the pane list
      drops to four, even where nothing in it literally says `Layout`.
- [x] The first-run onboarding wizard (`--onboarding`) copy is checked for a reference to the `Layout` pane
      by name; update or remove any step text that points a new user at a pane that no longer exists.
- [x] `crates/settings/ui/panes/layout_pane.slint` is deleted, along with its import and instantiation in
      `main_window.slint` and the sidebar's nav item for `Layout`.
- [x] `crates/settings/src/layout_pane_slint_snapshot.rs`'s existing tests for stack-width behaviour
      (`out_of_range_stack_width_is_refused_not_clamped` and its siblings) move to cover the same behaviour
      on the relocated control — either migrated into `shortcut_row_slint_snapshot.rs` or kept in a
      renamed file, but not left asserting against a pane that no longer exists. No existing assertion is
      weakened; the seam it runs at moves, the guard does not.
- [x] `DEF-2`'s fix (`STACK_WIDTH_SLIDER`/`STACK_WIDTH_INPUT` distinct accessible semantics in
      `crates/settings/src/theme.rs`) is carried to the relocated control rather than silently dropped —
      the two controls (stepper/typed-field) still need distinct accessible names on their new row.
- [x] `stack_width_percent`'s existing out-of-range-refused-not-clamped behaviour (`layout_pane.slint`'s
      `commit_typed`) is preserved exactly on the relocated control.
- [x] Full test suite green once, not only this ticket's own tests.

## Amendment 1 — 2026-09-07, coordinator (mandate `DEC-017`)

**The pane index is the same hazard `SPEC-4-01` just spent a return trip on.** That ticket needed
two must-fixes because a positional integer — `resolved[N]` — was hand-kept in more places than
anyone had counted. `Pane` has the identical shape, and this ticket removes a variant from the
middle of it.

Dropping `Pane::Layout` shifts `VmExceptions` 3→2 and `About` 4→3 in **four** independent places,
none of which the compiler checks, and two of which are markup where a wrong number is silent:

| Site | What it holds |
|---|---|
| `crates/settings/src/main.rs:136-141` | `Pane -> int`, including `Pane::Layout => 2` |
| `crates/settings/src/main.rs:330-337` | `int -> Pane`, including `2 => Pane::Layout` |
| `crates/settings/ui/components/sidebar.slint:125,132,139,146,153` | five hardcoded `selected: current_pane == N` |
| `crates/settings/ui/main_window.slint:285` | `if root.current_pane == 2 : LayoutPane` |

`Pane::ALL` is also declared `[Pane; 5]` (`crates/settings/src/app.rs:33`) — the hardcoded count
this ticket's own checklist already warns about.

A wrong number here does not crash and fails no test this ticket already names. It navigates to the
wrong pane, or highlights one sidebar entry while showing another pane's content.

- [x] `crates/settings/src/main.rs`'s two index tables are **derived from `Pane::ALL`'s position**
      rather than hand-numbered: forward by `position()`, reverse by `get()`.
      `ShortcutField::from_index` (`crates/settings/src/app.rs:119-124`) is the prior art in this
      same file, its out-of-range behaviour included — fall back to the first pane rather than
      panicking, because the index arrives across a UI boundary.
- [x] `app::tests::pane_declaration_order_is_the_navigation_index` asserts each `Pane`'s
      discriminant equals its position in `Pane::ALL`, exactly as
      `field_declaration_order_is_the_precedence_order` does for `ShortcutField`.
- [x] `app::tests::every_pane_index_round_trips_through_the_ui_boundary` asserts `Pane -> int ->
      Pane` is the identity for all four panes, and that an out-of-range index returns the first
      pane instead of panicking.
- [x] The markup's hardcoded `current_pane == N` comparisons are renumbered for four panes and
      **counted**: `sidebar.slint` carries exactly four nav entries, `main_window.slint` exactly
      four pane bodies. State in the closing report how the count was verified, because no Rust
      test can see these numbers.

**Why a guard and not just care.** `SPEC-4-01`'s panel proved by mutation that swapping two
positional indices left its own guard green while chords bound to the wrong actions. Same
discipline here: once the guards exist, swap two pane indices, watch them go red, restore. A guard
never seen red is a claim rather than proof.

**The percent bounds trap, confirmed by reading the code.** This ticket's main list already warns
about it; both halves are verified. `crates/settings/ui/components/shortcut_row.slint:39,46`
hardcode `1`/`99` (`base >= 1 && base < 99`), and `crates/settings/ui/panes/layout_pane.slint:9,10`
carry `min_percent: 10`, `max_percent: 100`. Setting `has_percent: true` on `Stack` without
per-row bounds silently gives stack width the wrong range.

- [x] `app::tests::the_stack_row_carries_its_own_percent_bounds` pins `Stack`'s range at 10-100 and
      the four `SnapPercent*` rows at 1-99, so the two cannot be collapsed back into one literal
      pair.

**One silent-failure seam worth knowing before you start.** `ShortcutField::set_percent`
(`crates/settings/src/app.rs`) ends in a `_ => {}` catch-all. Adding `Stack` to `has_percent()`
without adding it to `set_percent` therefore **fails silently** — the row draws a percent control
that discards every edit — rather than failing to compile.

## Amendment note — one existing test retires rather than migrates

`app::tests::the_layout_pane_no_longer_claims_snapping` asserts `Pane::Layout.label() == "Layout"`
and that `from_label("Layout")` resolves. It cannot survive the variant's removal. Its intent — that
this pane does not claim to hold snapping controls — is satisfied completely by the pane no longer
existing, so it **retires**. Say so in the closing report; do not delete it silently, and do not
invent a replacement assertion for a pane that is gone.

`m.set_pane(Pane::Layout)` sits inside an unrelated test and needs a different pane.
`crates/settings/src/layout_pane_slint_snapshot.rs` is declared as a module in
`crates/settings/src/main.rs`; removing the file means removing that declaration too.

## Amendment 2 — 2026-09-07, Step 1 landed (coordinator)

Six of the seven tests are committed. **Four are red**, and they are the definition of done:

| Test | Red because |
|---|---|
| `app::tests::pane_enum_no_longer_declares_layout` | `Pane::ALL.len()` is 5, wants 4 |
| `app::tests::overlapping_stack_row_has_percent_true` | `Stack.has_percent()` is false |
| `app::tests::the_stack_row_carries_its_own_percent_bounds` | `Stack` disagrees about whether it has a percentage |
| `persistence::tests::stack_width_percent_round_trips_through_the_shortcut_row_path` | writes 42, reads back 50 |

That last one is the silent seam this ticket's main list warned about, caught in the act:
`set_percent`'s `_ => {}` swallowed the write and the default survived. Nothing failed to compile
and no user would have seen an error — the control would simply have discarded every edit.

Two are **green guards**, both seen red under a deliberate `Pane::ALL` reorder and restored:
`pane_declaration_order_is_the_navigation_index` and
`every_pane_index_round_trips_through_the_ui_boundary`.

### Two corrections to this ticket's own test list

- `main::tests::overlapping_stack_row_has_percent_true` is now **`app::tests::`**. `has_percent` is
  a `ShortcutField` method, and `crates/settings/src/main.rs` has no `tests` module — asserting it
  there would mean standing up a real Slint window to build the row model, which
  `shortcut_row_slint_snapshot` already owns. One fact, one place.
- `shortcut_row_slint_snapshot::tests::stack_row_percent_commits_on_save_click` is **not**
  committed. It needs the percentage control to exist on the `Stack` row, so it cannot be written
  before the work it tests. **The builder writes it, through `tdd`, at the seam the other
  `shortcut_row_slint_snapshot` tests already use** (`i-slint-backend-testing`'s `TestingBackend`,
  controls located by accessible label). It is an acceptance criterion, not an optional extra.

### Where the bounds live, decided

`ShortcutField::percent_bounds() -> Option<(u32, u32)>` is committed and already returns
`Some((10, 100))` for `Stack` and `Some((1, 99))` for the four `SnapPercent*` rows. **The numbers
live in Rust, not as markup literals**, because markup cannot be asserted on and a range that only
exists in `.slint` is a range nothing can guard — which is exactly how `shortcut_row.slint`'s
`1`/`99` and `layout_pane.slint`'s `10`/`100` came to disagree in the first place.

- [x] `ShortcutRowData` (`crates/settings/ui/panes/shortcuts_pane.slint`) grows `percent_min` and
      `percent_max`, fed from `percent_bounds()`, and `shortcut_row.slint`'s `step_plus`/
      `step_minus` read those instead of their hardcoded `1`/`99`.

`Pane::from_index` is also committed, mirroring `ShortcutField::from_index` including its
out-of-range fallback. What remains is rewiring `main.rs`'s two hand-numbered tables to it and to
`pane as i32`.

## Amendment 3 — 2026-09-07, return trip 1 of 2 (`wdi-build` Step 3 panel)

Everything mechanical in `3b8210b` checks out and **must not be redone**: the derived pane index
tables, the markup renumbering (VmExceptions 3→2, About 4→3, exactly four entries each side), the
snapshot migration with every surviving assertion carried verbatim, the percent plumbing, and the
refused-not-clamped behaviour with its 10-100 message. Both axes verified those independently.

### The two axes split on severity, and the coordinator adjudicated must-fix

Standards scored the accessible-label mechanism **must-fix**; Spec scored it **follow-up**. They
agreed on every fact. Spec supplied the one that decides it, and it is not about today's labels:

**Both new Stack tests locate their row by `find_by_accessible_label(&window, "Stack width input")`,
which resolves to the Stack row only because `Stack` alone happens to have bounds (10, 100).** The
row selection is a coincidence, not a declaration. Give a snap row a 10-100 range and the migrated
refused-not-clamped guard starts `.next()`-ing a snap row while its messages still say "Stack
width" — a guard asserting against the wrong control and still passing.

Second ground, independent of the first: `.control/registry/defects.yaml`'s `DEF-2` row names
`app::tests::focus_order_has_no_duplicates`, `focus_order_is_stable_across_calls` and
`focus_order_starts_with_navigation_and_ends_with_actions` as its guards. `3b8210b` deleted
`focus_order(Pane::Layout)` — the one production reader of `theme.rs`'s three `STACK_WIDTH_*`
names — so those three tests no longer touch these controls at all, and
`theme::tests::accessible_names_are_unique` now iterates constants the UI never draws. A registry
pointing at guards that no longer guard is corpus drift, which `wdi-build` returns.

`DEF-2`'s symptom has **not** returned: the three controls on the new row do carry three distinct
names. This is about the guard, and about identity being inferred from a number.

- [x] `crates/settings/ui/components/shortcut_row.slint:23-26` stops deciding four accessible
      labels with `(root.percent_min == 10 && root.percent_max == 100)`. The condition disappears
      from all four sites.
- [x] `crates/settings/src/theme.rs` gains the two names the markup currently spells itself and
      Rust never declared — the "field" wrapper for each kind (`"Stack width field"` and
      `"Snap percentage field"`) — plus the snap-percentage decrease/input/increase set, so all
      eight names live in the layer that can be asserted on. The four existing `STACK_WIDTH_*`
      constants are reused, not re-spelled.
- [x] `ShortcutRowData` carries the four labels, populated in `crates/settings/src/main.rs` from
      those constants **per `ShortcutField`**. Identity comes from which field the row is. That
      seam already exists — this commit put `percent_min`/`percent_max` through it.
- [x] `theme::tests::every_rendered_percent_control_name_comes_from_theme` asserts every accessible
      name a percentage control actually renders is one of the `theme.rs` constants. This is the
      half that makes `accessible_names_are_unique` — `DEF-2`'s own named guard — mean something
      again. Prove it can fail: point one label at a literal, watch it go red, restore.
- [x] Once the labels come from field identity, re-point the two Stack snapshot tests at whatever
      makes their row selection explicit rather than range-contingent, without weakening either
      assertion.

### Already landed by the coordinator in this trip, do not redo

- `ShortcutField::percent_bounds()` now reads `MIN_SNAP_PERCENT`/`MAX_SNAP_PERCENT` and
  `MIN_STACK_WIDTH_PERCENT`/`MAX_STACK_WIDTH_PERCENT` from `crates/shared/src/constants.rs` instead
  of hardcoding all four numbers. That hardcoding was coordinator code from `1051f10`, and
  `persistence.rs` already validated against those same constants.
- `main.rs`'s `unwrap_or((1, 99))` literal pair reads the constants too.
- `app::tests::every_percentage_row_reaches_all_four_percent_seams` closes the gap Spec found:
  `has_percent`, `percent`, `set_percent` and `percent_bounds` must agree for **every** field, and
  the write must actually land. `Stack`'s own round-trip only proved it for `Stack`, while
  `set_percent`'s `_ => {}` is exactly what this ticket was caught on. Seen red under a mutation
  that drops `Stack` back through the catch-all, and restored.
- `defects.yaml`'s `DEF-2` row records that its named guards stopped covering these controls.

### Recorded, not fixed

- The migrated snapshot test hardcodes `50` rather than `DEFAULT_STACK_WIDTH_PERCENT`. Copied
  verbatim from the deleted file — pre-existing, not a new deviation.
- `delta_y: -600.0` is a geometry-dependent magic number, and `setup_shortcuts_window` already
  scrolls, so the scroll in both migrated tests is dead motion. Harmless; it fails loudly through
  `.expect(...)` if the row moves.
- The four pre-existing snap tests take `.next()` on `"Snap percentage input"`, shared by four
  rows, relying on the first being `SnapPercentLeft`. Prior art, not this ticket's doing.
- `percent_bounds`'s `Option<(u32, u32)>` shape is Primitive Obsession by the smell baseline;
  naming a type for it is out of scope here.

## Amendment 4 — 2026-09-07, return trip 2 of 2 (the cap)

The re-panel returned **no must-fix on the mechanism** — identity from range is genuinely gone, all
eight names are live and rendered, the Stack tests select by identity, and no assertion was
weakened. Both axes then converged on two things the fix left behind. Standards scored both
must-fix; Spec scored the first follow-up only because this brief's "do not edit existing tests"
collided with Amendment 3's intent. **Adjudicated must-fix**: extending a test's coverage is not
weakening it — that rule exists to stop guards being softened, not to freeze them.

### Landed by the coordinator in this trip

- [x] Both register tests in `crates/settings/src/theme.rs` now iterate one `pub const ALL:
      &[ControlSemantics]` instead of hand-keeping their own arrays. They had drifted badly: of
      eighteen declared constants, **twelve** were covered for a non-empty name and **eleven** for
      uniqueness. `ONBOARDING_BACK_BUTTON` was in neither and `SHORTCUT_CONFLICT_SWAP` missing
      from uniqueness — both long before this ticket. Standards proposed the derived array over
      simply adding five lines, and it is the better fix for the reason `DEC-018` already gives.
- [x] Proven: renaming `SNAP_PERCENT_INPUT` to `"Stack width input"` — `DEF-2`'s literal symptom —
      now fails `accessible_names_are_unique`. Under the hand-kept array it passed green.
- [x] `every_rendered_percent_control_name_comes_from_theme` asserts **assignment**, not set
      membership. Spec named two mutations that passed the membership version: a snap row handed
      the stack family (which reproduces `DEF-2` *and* re-breaks the Stack tests' row selection),
      and an intra-row slot swap. Both now fail. The intra-row swap fails **only** this test, so
      it is the sole guard for that class.
- [x] The stronger check deliberately does **not** restate which field gets which family — a test
      that repeats the production mapping proves only that the mapping equals itself. It asserts
      two properties that hold whatever the mapping is: a row never mixes families or slots, and
      exactly one row wears the stack family.
- [x] `DEF-6` filed for the accessibility gap Standards found: all four custom-percentage rows
      announce the same four names, so a screen-reader user cannot tell the left-edge control from
      the top-edge one. Pre-existing — the retired pane produced the same strings — and out of
      `SPEC-4`'s remaining scope, so it is recorded rather than folded in.

### Coordinator error in this trip, recorded

Rewriting the two register tests by splicing between them **deleted**
`theme::tests::listening_state_has_a_spoken_announcement`. The suite still went green, and the
deletion was caught only by comparing the test count against `12708f1` (143 → 142). Restored
verbatim. It is the exact fault this brief forbids a builder from committing, and it was mine.

### The one item left for the builder

- [x] `crates/settings/src/main.rs`'s label match uses `_ =>` for the snap family, so it covers the
      four `SnapPercent*` **and** eleven non-percent fields — identity derived from "not Stack".
      Every sibling percent match on this enum enumerates instead: `has_percent`, `percent_bounds`,
      `percent` and `set_percent` each list the four explicitly and reserve `_` for the no-percent
      rest. `3p.md`'s own entry for this ticket credits removing a `_ => {}` as the root cause it
      was caught on, and a new one now sits beside that sentence. Enumerate
      `SnapPercentLeft|Right|Top|Bottom`; keep `_` for the rest.

**This is the second and final return trip.** A must-fix surviving it escalates rather than opening
a third.

## Out of scope, deliberately

- The five-group taxonomy and label changes — `SPEC-4-01`, already landed by the time this starts.
- Tooltip, vertical-centring, and no-horizontal-scroll polish — `SPEC-4-03`, which touches this same row
  again once its final control set (stepper + keycap + toggle, all three now on `Stack`) is in place.
