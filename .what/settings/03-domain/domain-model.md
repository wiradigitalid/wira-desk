---
type: model
component: settings
layer: conceptual
created: 2026-08-21
updated: 2026-08-21
---

# Model — settings

Conceptual domain model for the `settings` component. Represents domain entities, relationships, state transitions, and invariants governing user preferences, first-run onboarding, and auto-start configuration. Physical file layouts, TOML serialization models, and window message structs belong in `.how/settings/05-model/data-model.md`.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| `user-shortcut-preference` | The user's customized physical key combinations for primary cycling, fallback cycling, window snapping, and application passthrough lists. | Shortcut action identifier (e.g. `cycle_primary`, `cycle_fallback`, `snap_left`) |
| `onboarding-completion` | The status record indicating whether the initial interactive simulation tutorial has been completed, skipped, or is still pending. | User profile configuration identity and completion timestamp |
| `auto-start-preference` | The persistent configuration controlling whether Wira Desk launches silently at user logon via Windows Task Scheduler with elevated privileges. | Scheduled task identifier (`WiraDesk`, frozen as `shared::constants::TASK_NAME`) and target user profile |

## Relationships

- `user-shortcut-preference` is **persisted within** the shared platform configuration schema (`app-config`).
- `onboarding-completion` **gates** the presentation of the interactive tutorial dialog on application startup.
- `auto-start-preference` **governs** the registration and removal of the Windows Scheduled Task, which the daemon performs when it reloads configuration; this component only records the preference.
- Saving `user-shortcut-preference` **triggers** an atomic write to `app-config` followed by an explicit `ipc-reload-signal` dispatch.

## State Lifecycle

### onboarding-completion Lifecycle

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| `Pending` | `Completed` | User finishes interactive cycling simulation through mock windows | New User |
| `Pending` | `Skipped` | User clicks or activates "Skip Tutorial" button | New User |
| `Completed` / `Skipped` | `Pending` | User manually resets tutorial state from Settings advanced options | Power User |

### Shortcut Listening State

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| `Idle` | `Listening` | User focuses or clicks into a shortcut capture field | Power User |
| `Listening` | `Captured` | User presses a valid modifier + non-modifier key combination | Power User |
| `Listening` | `Cancelled` | User presses Escape or clicks away without entering a valid chord | Power User |
| `Captured` | `Idle` | Captured shortcut validated, stored in memory, and field unfocused | Power User |

## Invariants

- **Atomic Persistence Invariant:** Configuration changes must be completely and safely committed to `%APPDATA%\WiraDesk\config.toml` before the `WM_APP_RELOAD_CONFIG` IPC message is dispatched to the daemon (BR-1, BR-2, AD-5).
- **Physical Key Listening Invariant:** Shortcut input fields must capture raw physical keystroke combinations directly and must not accept arbitrary typed text strings (FR-18).
- **Skip Tutorial Availability Invariant:** The "Skip Tutorial" action must remain accessible and clearly visible on every step of the first-run onboarding flow (FR-17).
- **Per-User Task Alignment Invariant:** Auto-start scheduled tasks must always be created with `/RU %USERNAME%` and `/RL HIGHEST`, never under the `SYSTEM` account (BR-4, AD-13).
- **Full Accessibility Invariant:** All interactive settings controls, toggles, and modal dialogs must be fully navigable via keyboard and expose name, role, and state to screen readers via Windows UI Automation (FR-20, FR-21, AD-11a).
