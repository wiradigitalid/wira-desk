---
type: srs
component: settings
status: reviewed
created: 2026-08-21
updated: 2026-08-25
satisfies: [FR-7, FR-13, FR-16, FR-17, FR-18, FR-19, FR-20, FR-21]
reviewed:
  date: '2026-08-25'
  sha: '0f02673'
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — settings

## Decision Summary

The `settings` component provides the standalone on-demand graphical configuration shell and interactive first-run onboarding tutorial (`wiradesk-settings.exe`). It manages shortcut customization via live physical key listening, toggles silent elevated logon auto-start via Windows Task Scheduler, adapts dynamically to Windows light/dark system themes, and guarantees full keyboard navigation and screen reader accessibility through Windows UI Automation.

## Why

Configuration customization and user onboarding are episodic, UI-intensive tasks that require rich graphical controls, accessibility tree support, and theme synchronization. Isolating these capabilities into a distinct executable spawned on-demand keeps the background daemon lean (<2 MB RAM), eliminates UI runtime memory bloat from the background service, and ensures rendering operations never stall input hooks.

## Actor Register

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Power User | Desktop user wanting customized shortcut chords, auto-start management, or diagnostic preferences. | Customize primary/fallback shortcuts, toggle auto-start on boot, modify passthrough lists. |
| New User | First-time user encountering Wira Desk upon installation or initial launch. | Step through interactive mock window cycling simulation or dismiss onboarding via Skip Tutorial. |

## UC Catalogue

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-4 | Change a keyboard shortcut in Settings | Power User | FR-7, FR-18 | no |
| UC-5 | Complete or skip the first-run tutorial | New User | FR-17 | no |
| UC-6 | Turn auto-start on boot on or off | Power User | FR-13 | no |

## Constraints

- Must adhere strictly to Architecture Spine invariants AD-1, AD-5, AD-11, AD-11a, AD-12, and AD-13.
- Must persist configuration changes atomically to `%APPDATA%\WiraDesk\config.toml` before dispatching `WM_APP_RELOAD_CONFIG` to the daemon (BR-1, BR-2, AD-5).
- Must record the auto-start preference and hand task registration to the daemon, which creates the task with `/RL HIGHEST` for `%USERNAME%` and an empty/secure Start In directory to prevent DLL hijacking (BR-4, AD-13). This component never invokes `schtasks` itself.
- Must refuse to save a configuration in which two actions carry the same chord, naming both fields; the daemon's answer to the same condition is different by design and is not this component's to apply (BR-6, DEC-009).
- Must draw the Shortcuts pane, order keyboard focus within it, and resolve collision precedence from **one** declared sequence of editable actions, never from a second independent list (LBR-ST-14, LBR-ST-5).
- Must provide complete keyboard navigation (Tab/Shift+Tab, arrow keys, Escape/Enter) across all interactive dialogs (FR-20).
- Must expose full UI Automation properties (names, roles, states, shortcut values) to assistive technologies via AccessKit (FR-21, AD-11a).
- Must render UI in pure native Rust (`egui`/`eframe`) without webview wrappers or heavy runtime frameworks (AD-11).

## Non-Goals

- Executing window cycling, snapping, or low-level keyboard interception (delegated to `window-management`).
- Maintaining persistent tray icon lifecycle or handling Explorer restart broadcasts (owned by `window-management`).
- Cloud account synchronization or remote profile backups.

## Prerequisite

- Writable user directory at `%APPDATA%\WiraDesk\`.
- Platform configuration schema and IPC reload contracts available (`app-config`, `ipc-reload-signal`).
- Background daemon running or available for elevated onboarding invocation.

## Success Signal

A user customizes a shortcut combination in Settings, saves preferences, and immediately triggers window cycling with the new combination on the very next keystroke without requiring a daemon restart or system reboot.

## Assumptions, Risks, and To Be Confirmed

### Assumptions
- AccessKit integration in `eframe` satisfies screen reader accessibility compliance (Windows Narrator) across target Windows 10 and 11 builds.
- Windows Task Scheduler `ONLOGON` tasks for `%USERNAME%` execute reliably without triggering repeated UAC prompts.

### Risks
- Potential configuration write races if multiple settings instances were launched simultaneously, mitigated by single-instance dialog behavior and atomic file replacement.

### To Be Confirmed
- None; all architectural invariants and functional requirements are stated and traced. Ratification is the owner's act at G3, recorded in `gates_passed`.

## Gate Checklist · [G3]

- ★ Every functional requirement (FR-7, FR-13, FR-16..21) mapped to a usecase or carries explicit `no_uc:` justification? Yes.
- ★ All use case titles phrased as natural user sentences? Yes.
- ★ Actor Register complete and aligned with PRD journeys? Yes.
- ★ Invariants AD-1, AD-5, AD-11, AD-11a, AD-12, AD-13 and cross-component business rules BR-1..6 respected? Yes.

## Design Reference · [G3]

Paired SDD: `.how/settings/SDD-settings.md`. UX Design: `.how/settings/01-ux/DESIGN.md`. Binds invariants AD-1, AD-5, AD-11, AD-11a, AD-12, AD-13.

---

## Slots

- `02-rules/rules-settings.md`: Local component rules (LBR-ST-1..14).
- `03-domain/domain-model.md`: Conceptual domain entities (`user-shortcut-preference`, `onboarding-completion`, `auto-start-preference`).
- `03-domain/state-machines.md`: First-run tutorial progression and shortcut capture listening state machine.
- `04-usecases/`: Detailed step-by-step flows (`UC-4-change-shortcut.md`, `UC-5-first-run-tutorial.md`, `UC-6-toggle-auto-start.md`).
- `05-scenarios/`: Edge-case branching scenarios (`SCN-01-invalid-shortcut-rejected.md`, `SCN-02-autostart-task-create-fails.md`).

## Open Items

*(No open items; questions resolved in `.control/questions/answered.md`)*
