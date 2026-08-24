---
type: lc
id: LC-settings-shell
name: Settings Shell
lc_type: ui-composite
container: settings
component: settings
owner: Wira Desk Core
area: presentation
created: 2026-08-21
---

# LC-settings-shell — Settings Shell

## Responsibility

`LC-settings-shell` is the egui/eframe presentation layer for `wiradesk-settings.exe`. It owns:

1. Application frameless shell (`with_decorations(false)`), navigation tabs (General, Shortcuts, Layout & Snapping, VM & Exceptions, About), and adaptive light/dark theming (FR-19, AD-11a).
2. Wiring shortcut fields to `LC-shortcut-capturer` listening mode (FR-18).
3. Tab order across all interactive controls (FR-20, LBR-ST-5).
4. First-run onboarding panels when launched with `--onboarding` (FR-17, UC-5).
5. Surfacing validation errors inline before save is enabled.

The shell never writes `config.toml` directly; all persistence goes through `LC-config-writer`.

## Depends on

- `crates/settings/src/app.rs` — egui application state and panels.
- `crates/settings/src/theme.rs` — system theme detection and Segoe UI tokens.
- `LC-shortcut-capturer` — capture state for shortcut fields.
- `LC-config-writer` — save, reload signal, auto-start toggle.
- eframe with `accesskit` feature (AD-11a).

## Interface

### Inbound

| Event | Action |
| --- | --- |
| User edits field | Update in-memory `Config` draft |
| Save clicked | Delegate to `LC-config-writer::save` |
| Auto-start toggled | Delegate to `LC-config-writer::set_autostart` |

### Outbound

| Call | Target |
| --- | --- |
| `begin_listening(field_id)` | `LC-shortcut-capturer` |
| `save(config)` | `LC-config-writer` |

## Notes

- **Accessibility:** Toggle and capture states must expose UI Automation values, not color alone (FR-21, LBR-ST-6).
- **Evidence:** [PARTIAL] `crates/settings/src/app.rs`, `crates/settings/src/theme.rs`.
