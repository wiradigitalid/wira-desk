---
baseline_commit: eddb21a
workflow_id: story-2-3-live-zorder-discovery
---

# Story 2.3: Live Z-Order and Executable-Identity Discovery

Status: review

## Story

As a multitasking user,
I want WinTick to discover the current windows of my active application at the moment I invoke cycling,
so that stale process or Z-order information never sends me to the wrong application.

## Acceptance Criteria

### AC-2.3-001 — One sweep, no cache
`EnumWindows` invoked exactly once per request; returned top-to-bottom Z-order preserved; no Z-order or window cache retained between commands.

### AC-2.3-002 — Executable-basename grouping
Same basename across separate PIDs classified as the same application, case-insensitively; different PIDs do not prevent grouping; matching PIDs alone cannot equate different executables.

### AC-2.3-003 — Graceful metadata failure
A process that closes, denies query access, or changes state during discovery yields an unavailable candidate; discovery continues without crash, blocking retry, or user-facing error.

### AC-2.3-004 — Non-blocking API audit
Only non-blocking metadata APIs. No `SendMessage`, `GetWindowText`, focus API, eligibility policy, or Hook Thread code.

### AC-2.3-005 — Independent verification
Fresh ordered snapshot per command with correct multi-process grouping, requiring no eligibility, activation, monitor, virtual-desktop, or UI implementation.

## Dev Notes

### Implementation

`crates/daemon/src/cycling/source.rs`. `Win32CandidateSource::snapshot()` performs a
single `EnumWindows` sweep into a freshly allocated `Vec`, then indexes it into
`Candidate` values with dense `z_index` starting at 0. There is no `static`, no
`OnceLock`, and no memoization anywhere in the module — the absence of storage is
what satisfies "no cache" structurally rather than by convention.

Per-window facts use only: `IsWindowVisible`, `IsIconic`,
`GetWindowLongPtrW(GWL_EXSTYLE)`, `GetClassNameW`, `GetWindowThreadProcessId`,
`OpenProcess`, `QueryFullProcessImageNameW`, `CloseHandle`. All are non-blocking
metadata calls (AC-2.3-004).

### PID Containment

`identity_of()` obtains a PID solely to call `OpenProcess`. The PID never leaves
that function and never enters `AppIdentity`, so it structurally cannot become a
same-application key (AC-2.3-002).

### Failure Degradation

Five distinct failure points — zero HWND, zero thread/PID, `OpenProcess` refusal,
`QueryFullProcessImageNameW` failure, and zero-length result — all converge on
`AppIdentity::Unavailable`. `enum_proc` returns `TRUE` unconditionally: a window
we cannot describe must not truncate the sweep, or every window below it would
silently vanish from the Z-order.

### Windows-sys Note

`EnumWindows` is exported from `Win32::UI::WindowsAndMessaging` in windows-sys
0.52 (verified against the vendored crate source, not assumed). No new Cargo
feature was required.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 9 unit tests; the identity-semantics and degradation tests are fully
  desktop-free, while the snapshot-shape tests assert only structural
  invariants so they are stable in any session.
- Static gates PASS: workspace clippy exit 0 with no new warnings; test binary
  compiles.
- **Test execution NOT performed** — `cargo test -p daemon` requires elevation
  (`os error 740`). Deferred to the user.
- The helper-window runtime harness named in AC-2.3-005 is **not** implemented;
  it needs a live elevated desktop. Outstanding.

### File List
- `crates/daemon/src/cycling/source.rs` (new)
- `crates/daemon/src/cycling/mod.rs` (modified — submodule declaration)
