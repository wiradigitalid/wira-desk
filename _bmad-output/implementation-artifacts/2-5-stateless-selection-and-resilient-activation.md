---
baseline_commit: eddb21a
workflow_id: story-2-5-selection-and-activation
---

# Story 2.5: Stateless Selection and Resilient Activation

Status: review

## Story

As a WinTick user,
I want each shortcut to move focus to the next eligible window and recover from windows closing mid-cycle,
so that cycling remains predictable even while applications change.

## Acceptance Criteria

### AC-2.5-001 — Next, wrap, no-op
Selects the first eligible candidate after the active window; wraps to the beginning at most once; a sequence with no alternative target is a no-op.

### AC-2.5-002 — Active window absent
Chooses the first eligible non-active candidate deterministically; never loops indefinitely; never invents a cached position.

### AC-2.5-003 — Failure continuation
On activation failure, advances to the next untried candidate; each candidate attempted no more than once; exhaustion ends silently without crash or popup.

### AC-2.5-004 — Hung-target parity
A Not Responding eligible target receives the same bounded focus attempt as a responsive one; no probe, skip, restore, maximize, or wait.

### AC-2.5-005 — Independent verification
Pure selection tests, fake-activator tests, and a synthetic hung/closing-window harness cover next, wrap, no-op, failure continuation, and one-pass termination.

## Dev Notes

### Split Across Two Files

`cycling/selection.rs` is pure and imports no `windows_sys`.
`cycling/activation.rs` holds every Win32 call. The split matches the lane
ownership in AC-2.2-006 and keeps the testable half fully desktop-free.

### Why the Active Window Is Filtered, Not Skipped

`attempt_order()` removes the active window from the sequence rather than
skipping it inside a loop. The resulting sequence therefore *cannot* revisit it,
which is what bounds the pass structurally. `select_after()` then filters by an
explicit `tried` list, so "no more than once" holds without any loop counter.

`every_candidate_attempted_at_most_once` drives the real loop to exhaustion and
asserts both no-repetition and termination, with a hard length guard that would
catch an infinite loop instead of hanging the suite.

### Statelessness

Selection keeps no cursor between commands. Two identical calls return identical
results (`selection_keeps_no_cursor_between_calls`). A window opening or closing
between shortcuts therefore cannot desynchronize it — there is nothing to
desynchronize.

### Bounded Focus Attempt

`SetForegroundWindow` alone is refused by Windows unless the caller owns the
foreground. The documented remedy is a brief `AttachThreadInput` to the
foreground thread's input queue plus **one** retry, which `focus_attempt()`
does. Both calls return immediately; nothing polls, sleeps, or retries in a loop.

Validity is checked with `IsWindow`, which does not talk to the owning thread —
so a hung window passes it exactly like a healthy one, satisfying AC-2.5-004.
There is deliberately no `ShowWindow`/restore/maximize call anywhere in the file.

### Windows-sys Note

`AttachThreadInput` is exported from `Win32::System::Threading` in windows-sys
0.52, **not** from `UI::Input::KeyboardAndMouse` where the Win32 documentation
groups it. Verified against the vendored crate source after the first import
failed to resolve.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 19 unit tests (16 selection, 3 activation). Selection tests are fully
  desktop-free; activation tests use only bogus handles, which resolve
  identically in any session.
- Static gates PASS: workspace clippy exit 0; test binary compiles.
- **Test execution NOT performed** — requires elevation (`os error 740`).
- **`focus_attempt()` has never been executed against a real window.** The
  `AttachThreadInput` path is the highest-risk code in this story and is
  verifiable only on a live elevated desktop. Outstanding.

### File List
- `crates/daemon/src/cycling/selection.rs` (new)
- `crates/daemon/src/cycling/activation.rs` (new)
- `crates/daemon/src/cycling/mod.rs` (modified — submodule declarations)
