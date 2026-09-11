---
id: SPEC-9-02
component: settings
satisfies: [UC-14]
blocked_by: []
status: ready-for-agent
tests:
  - app::tests::toggle_switches_are_vertically_centered_in_cards
---

# 02: Vertical alignment of toggle switches and SettingToggleRow component

**What to build:** Create a reusable `SettingToggleRow` Slint component in `crates/settings/ui/components/setting_toggle_row.slint`. Wrap the `ToggleSwitch` in `VerticalLayout { alignment: center; }` to guarantee vertical centering relative to adjacent title and description text. Refactor `general_pane.slint`, `mouse_pane.slint`, `about_pane.slint`, and `onboarding.slint` to use this component, removing hardcoded `y: Npx` hacks.

**Blocked by:** None

## Acceptance Criteria

- [ ] New component `crates/settings/ui/components/setting_toggle_row.slint` defined:
      - Accepts properties: `title: string`, `description: string`, `in-out property <bool> checked`, `enabled: bool` (default true), `accessible_label: string`.
      - Exposes callback: `toggled(bool)`.
      - Internal layout wraps `ToggleSwitch` in `VerticalLayout { alignment: center; }`.
      - Standard horizontal padding: `padding-left: 16px; padding-right: 16px;`.
- [ ] `general_pane.slint` refactored:
      - "Start Wira Desk with Windows" row uses `SettingToggleRow`.
      - "Spatial Preservation Lock" row uses `SettingToggleRow` (with `enabled: false`).
      - "Show Unresponsive Windows (UX Honesty)" row uses `SettingToggleRow`.
- [ ] `mouse_pane.slint` refactored:
      - "Enable Mouse Navigation" card uses `SettingToggleRow`.
- [ ] `about_pane.slint` refactored:
      - "Check for updates automatically" uses `SettingToggleRow` (or centered `ToggleSwitch` wrapper without `y: 4px`).
- [ ] `onboarding.slint` refactored:
      - Removes `y: 3px` offset from toggle rows.
- [ ] Visual verification confirms all toggle pills are mathematically vertically centered with adjacent multi-line text across all themes (dark/light).
