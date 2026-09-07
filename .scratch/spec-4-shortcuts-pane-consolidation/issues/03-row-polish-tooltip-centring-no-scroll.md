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

## Out of scope, deliberately

- Anything covered by `DEF-5` (typed digits not committing via a real keystroke) — `SPEC-4-04`. This
  ticket may touch the same file but must not fold that defect's fix in as a side effect; if it is found
  and fixed here anyway, it still needs `SPEC-4-04`'s own dedicated test, not a note in this ticket.
