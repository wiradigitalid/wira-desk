---
type: lc
id: LC-shortcut-capturer
name: Shortcut Capturer
lc_type: control
container: settings
component: settings
owner: Wira Desk Core
area: input-capture
created: 2026-08-21
---

# LC-shortcut-capturer — Shortcut Capturer

## Responsibility

`LC-shortcut-capturer` implements listening-mode keyboard capture for shortcut fields (FR-18). While active it:

1. Ignores typed Unicode characters; only physical key-down events with modifier state matter.
2. Requires at least one modifier (Win, Ctrl, or Alt) before accepting a chord (SCN-01).
3. Rejects reserved or unsafe combinations before they reach `LC-config-writer`.
4. Announces listening state and validation results through UI Automation accessible values (FR-21, LBR-ST-6).
5. Cancels on Escape without mutating the saved configuration.

## State machine

See `.what/settings/03-domain/state-machines.md` — `Idle` ↔ `Listening`.

## Depends on

- `crates/settings/src/app.rs` — field focus and capture UI.
- `shared::shortcut::{validate_shortcut, ShortcutError}`.
- egui input pipeline (low-level key events, not `TextEdit` for chords).

## Interface

| Method | Returns |
| --- | --- |
| `begin_listening()` | Enters Listening state |
| `on_key_event(event)` | `Some(Shortcut)` or validation error |
| `cancel()` | Returns to Idle |

## Notes

- **Evidence:** [PARTIAL] `crates/shared/src/shortcut.rs`, settings UI capture handlers in `app.rs`.
