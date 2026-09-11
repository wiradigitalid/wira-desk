---
id: SPEC-9-03
component: settings
satisfies: [FR-32]
blocked_by: [SPEC-9-02]
status: ready-for-agent
tests:
  - app::tests::card_dividers_render_full_bleed_across_panes
---

# 03: CardDivider full-bleed consistency across all settings panes

**What to build:** Standardize the card container layout across all panes to match the reference pattern established in `shortcuts_pane.slint`: Card containers use `padding: 0px` (or `padding: 0px; spacing: 0px;`), while individual row components provide internal `padding-left: 16px; padding-right: 16px;`. This ensures that `CardDivider` instances span full-bleed from edge to edge of the Card without unintended horizontal indentations.

**Blocked by:** `SPEC-9-02` (SettingToggleRow provides row-level internal padding)

## Acceptance Criteria

- [ ] In `general_pane.slint`:
      - Card 2 (Spatial Lock & UX Honesty) container uses `padding: 0px; spacing: 0px;`.
      - `CardDivider` renders edge-to-edge between the two rows, matching the divider appearance in `shortcuts_pane.slint`.
- [ ] In `mouse_pane.slint`:
      - Card 2 (Button & Wheel Mapping Presets) container uses `padding: 0px; spacing: 0px;` (updated from `padding: 4px`).
      - `CardDivider` between preset rows renders full-bleed without 4px insets.
- [ ] In `about_pane.slint`:
      - The update block card uses consistent full-bleed divider separation between the automatic updates toggle row and the manual check/install action row.
- [ ] Divider line thickness (1px) and token (`Palette.stroke_divider`) remain pixel-exact and consistent across dark and light modes.
