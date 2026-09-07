---
id: SPEC-4-02
component: settings
satisfies: [UC-4, UC-9]
blocked_by: [SPEC-4-01]
status: ready-for-agent
tests:
  - app::tests::pane_enum_no_longer_declares_layout
  - app::tests::overlapping_stack_row_has_percent_true
  - shortcut_row_slint_snapshot::tests::stack_row_percent_commits_on_save_click
  - persistence::tests::stack_width_percent_round_trips_through_the_shortcut_row_path
  - app::tests::pane_declaration_order_is_the_navigation_index
  - app::tests::every_pane_index_round_trips_through_the_ui_boundary
  - app::tests::the_stack_row_carries_its_own_percent_bounds
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

- [ ] `ShortcutField::has_percent()` returns `true` for `Stack`, alongside the four `SnapPercent*` fields.
- [ ] `ShortcutField::get_percent`/`set_percent` (or their equivalent for `Stack`) read and write
      `cfg.layout.stack_width_percent` — the existing field, unrenamed; nothing here is a config migration.
- [ ] `crates/settings/ui/components/shortcut_row.slint`'s `step_plus`/`step_minus` currently hardcode their
      bounds as literals (`base >= 1 && base < 99`) — correct for every `Snap to custom` row, wrong for `Stack`,
      which needs 10–100 (`layout_pane.slint`'s existing `min_percent`/`max_percent`). `ShortcutRow` gains
      per-row bound properties (e.g. `in property <int> percent_min: 1; in property <int> percent_max:
      99;`, overridden to `10`/`100` on the `Stack` row only) rather than a second hardcoded literal pair —
      setting `has_percent: true` on `Stack` without this change silently gives it the wrong range.
- [ ] `Pane` (wherever it is declared — `crates/settings/src/app.rs` and/or the Slint side) drops `Layout`.
      Every place keyed on `Pane::Layout` — focus order, theme accessible-name tables, sidebar nav data —
      is updated in the same commit, not left as a dangling reference to a removed variant. `DEC-014`'s Cost
      section names this risk explicitly: grep for `Pane::Layout` and `layout_pane` and account for every
      hit, do not rely on the compiler to find all of them (a `&str` pane key or a UIA accessible-name table
      entry will not fail to compile). This includes any hardcoded pane-COUNT, not only named references —
      a sidebar nav-items array or test asserting exactly five entries is stale the moment the pane list
      drops to four, even where nothing in it literally says `Layout`.
- [ ] The first-run onboarding wizard (`--onboarding`) copy is checked for a reference to the `Layout` pane
      by name; update or remove any step text that points a new user at a pane that no longer exists.
- [ ] `crates/settings/ui/panes/layout_pane.slint` is deleted, along with its import and instantiation in
      `main_window.slint` and the sidebar's nav item for `Layout`.
- [ ] `crates/settings/src/layout_pane_slint_snapshot.rs`'s existing tests for stack-width behaviour
      (`out_of_range_stack_width_is_refused_not_clamped` and its siblings) move to cover the same behaviour
      on the relocated control — either migrated into `shortcut_row_slint_snapshot.rs` or kept in a
      renamed file, but not left asserting against a pane that no longer exists. No existing assertion is
      weakened; the seam it runs at moves, the guard does not.
- [ ] `DEF-2`'s fix (`STACK_WIDTH_SLIDER`/`STACK_WIDTH_INPUT` distinct accessible semantics in
      `crates/settings/src/theme.rs`) is carried to the relocated control rather than silently dropped —
      the two controls (stepper/typed-field) still need distinct accessible names on their new row.
- [ ] `stack_width_percent`'s existing out-of-range-refused-not-clamped behaviour (`layout_pane.slint`'s
      `commit_typed`) is preserved exactly on the relocated control.
- [ ] Full test suite green once, not only this ticket's own tests.

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

- [ ] `crates/settings/src/main.rs`'s two index tables are **derived from `Pane::ALL`'s position**
      rather than hand-numbered: forward by `position()`, reverse by `get()`.
      `ShortcutField::from_index` (`crates/settings/src/app.rs:119-124`) is the prior art in this
      same file, its out-of-range behaviour included — fall back to the first pane rather than
      panicking, because the index arrives across a UI boundary.
- [ ] `app::tests::pane_declaration_order_is_the_navigation_index` asserts each `Pane`'s
      discriminant equals its position in `Pane::ALL`, exactly as
      `field_declaration_order_is_the_precedence_order` does for `ShortcutField`.
- [ ] `app::tests::every_pane_index_round_trips_through_the_ui_boundary` asserts `Pane -> int ->
      Pane` is the identity for all four panes, and that an out-of-range index returns the first
      pane instead of panicking.
- [ ] The markup's hardcoded `current_pane == N` comparisons are renumbered for four panes and
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

- [ ] `app::tests::the_stack_row_carries_its_own_percent_bounds` pins `Stack`'s range at 10-100 and
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

- [ ] `ShortcutRowData` (`crates/settings/ui/panes/shortcuts_pane.slint`) grows `percent_min` and
      `percent_max`, fed from `percent_bounds()`, and `shortcut_row.slint`'s `step_plus`/
      `step_minus` read those instead of their hardcoded `1`/`99`.

`Pane::from_index` is also committed, mirroring `ShortcutField::from_index` including its
out-of-range fallback. What remains is rewiring `main.rs`'s two hand-numbered tables to it and to
`pane as i32`.

## Out of scope, deliberately

- The five-group taxonomy and label changes — `SPEC-4-01`, already landed by the time this starts.
- Tooltip, vertical-centring, and no-horizontal-scroll polish — `SPEC-4-03`, which touches this same row
  again once its final control set (stepper + keycap + toggle, all three now on `Stack`) is in place.
