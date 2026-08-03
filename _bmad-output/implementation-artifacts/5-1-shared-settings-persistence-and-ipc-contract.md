---
baseline_commit: 9bc578f
workflow_id: story-5-1-settings-persistence-contract
---

# Story 5.1: Shared Settings, Persistence, and IPC Contract

Status: review

## Story

As a WinTick user,
I want my preferences represented and saved through one stable shared contract,
so that Settings and the background daemon cannot interpret or apply them differently.

## Acceptance Criteria

### AC-5.1-001 — Frozen defaults — **DONE, verified**
### AC-5.1-002 — Lossless round-trip and canonical shortcuts — **DONE, verified**
### AC-5.1-003 — Validation before replacement — **DONE, verified**
### AC-5.1-004 — Atomic persistence — **DONE, verified**
### AC-5.1-005 — IPC intent shape — **DONE** (message identifier and no-pointer rule verified; delivery to a live daemon not exercised)
### AC-5.1-006 — First-run contract — **DONE, verified**
### AC-5.1-007 — Independent verification — **DONE** — `cargo test -p settings` really runs

## Dev Notes

### Canonical Form Is a Shared Contract, Not a Display Detail

`Shortcut::to_canonical_string()` freezes modifier order as
`ctrl+win+alt+shift` + key. Without a fixed order, `"win+ctrl+a"` and
`"ctrl+win+a"` produce two different strings for the same shortcut, and every
text comparison anywhere in the system becomes quietly wrong.
`differently_ordered_input_canonicalizes_identically` and a round-trip test over
all six frozen defaults pin it.

`validate_shortcut()` returns the canonical string rather than `()`, so a caller
structurally cannot persist the user's raw input in a non-canonical form.

### Recovering the Reason for a Rejection

`Shortcut::parse` collapses three distinct failures into `None`.
`classify_parse_failure()` re-walks the tokens to distinguish an unsupported
token from a missing main key from two main keys — a user who typed `ctrl+a+b`
deserves a different message from one who typed `win+nonsense`.

### Ordering Is the Contract

`save_and_notify()` does validate → atomic write → signal, in that order. The
reload signal is emitted only after `Config::save` returns, which is the point
at which the completed file is visible. Signalling earlier would race the
daemon onto a half-written file.

`rejection_leaves_the_previous_file_intact` writes a good config, attempts a bad
one, and asserts the file bytes are unchanged.

### No Watcher, No Polling

`signal_reload()` posts `WM_APP_RELOAD_CONFIG` to the daemon's hidden window and
nothing else. No configuration pointer crosses the process boundary; the data
travels through the TOML file. A missing daemon returns `false` rather than
erroring — it will read the file when it next starts.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- **`cargo test -p settings` PASS 48/48 and `cargo test -p shared` PASS 25/25 —
  really executed.** The `settings` crate carries no `requireAdministrator`
  manifest, so unlike the daemon its tests run without elevation. This story is
  genuinely verified, not merely compiled.
- Persistence tests write to real temp files and assert byte-level
  non-modification on rejection.
- **`signal_reload()` has never been observed reaching a running daemon.** The
  message identifier and the no-pointer rule are verified; end-to-end delivery
  is not.

### File List
- `crates/shared/src/shortcut.rs` (modified — `to_canonical_string`, `name_from_vk`, 6 tests)
- `crates/settings/src/persistence.rs` (new)
- `crates/settings/Cargo.toml` (modified — `windows-sys`)
