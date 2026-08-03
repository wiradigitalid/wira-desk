---
baseline_commit: f02cda2
workflow_id: story-3-3-vm-rdp-passthrough
---

# Story 3.3: VM and Remote Desktop Passthrough Decision Adapter

Status: review

## Story

As a VM or Remote Desktop user,
I want WinTick to recognize when my shortcut belongs to the guest environment,
so that the physical key combination passes through without interference or synthetic reinjection.

## Acceptance Criteria

### AC-3.3-001 — Process match
A foreground window owned by a configured VM/RDP process returns `Passthrough` on case-insensitive basename comparison.

### AC-3.3-002 — Class match
A configured class returns `Passthrough` even when process identity alone does not match.

### AC-3.3-003 — Confirmed non-match
A fully resolved identity matching neither returns `ContinueWinTickMatching` and does not alter Epic 2 exact-shortcut, throttle, or swallowing policy.

### AC-3.3-004 — Conservative failure
Unresolvable identity returns `Passthrough`, exposes a deferred diagnostic signal handled outside the callback, and never requests `SendInput` or another reinjection mechanism.

### AC-3.3-005 — Callback budget
The Hook Thread collector uses bounded non-blocking Win32 queries and reusable fixed buffers; no allocation, logging, config parsing, lock acquisition, sleep, or Worker call.

### AC-3.3-006 — Deterministic harness
Default identifiers, configured identifiers, mixed casing, duplicates, non-matches, and identity-query failures all match the frozen Story 3.1 contract; prepared policy stays immutable for the lifetime of the active Hook configuration.

## Dev Notes

### Zero Allocation Drove the Whole Design

AC-3.3-005 forbids allocation in the callback, which rules out building a
`ForegroundIdentity { Option<String> }` per keystroke. So:

- `HookIdentityCollector` owns two fixed arrays (`[u16; 256]` class,
  `[u16; 260]` path) and reuses them on every event.
- `WideIdentity<'a>` borrows slices out of those buffers; its lifetime prevents
  it outliving the next collection.
- `eq_ignore_ascii_case_wide()` compares a pre-normalized policy `&str` directly
  against raw UTF-16, so no `String` is constructed on either side.
- `basename_range()` locates the executable name by index rather than slicing
  into a new allocation.

### Contract Correction Returned to Its Owning Story

Story 3.1 originally named the non-bypass outcome `Intercept`; this story's AC
names it `ContinueWinTickMatching`. Per AC-3.4-001, the correction was applied
in `context/mod.rs` — the owning story — rather than aliased here.

### Two Classification Paths, One Answer

`BypassPolicy::classify()` (owned `String`, Worker-side) and `classify_wide()`
(borrowed UTF-16, Hook-side) implement the same rule. They could drift, so
`wide_and_owned_classification_agree` drives both through six identity shapes
and asserts equality. A divergence would mean the Hook and Worker disagreed
about whether the user is inside a VM.

### Deferred Diagnostics

The callback cannot log, so identity failures bump a `static AtomicU64`. The
Worker drains it later at a safe boundary. `identity_failure_count()` and
`reset_identity_failures()` are the accessors; wiring them to the Tier-2 path
belongs to Story 3.4.

### No Reinjection

`SendInput` and every other synthetic input API are absent from the file. The
adapter only decides.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 20 unit tests. The comparison, basename, and classification tests are fully
  desktop-free; `collect()` touches the live foreground but asserts only
  structural invariants.
- Static gates PASS in debug and release.
- **Test execution NOT performed** — needs elevation.
- The allocation guarantee is **structural** (fixed arrays, borrowed slices),
  not measured. An allocation-counting gate would need a runtime harness.

### File List
- `crates/daemon/src/context/vm_bypass.rs` (new)
- `crates/daemon/src/context/mod.rs` (modified — submodule decls, `Intercept` → `ContinueWinTickMatching`)
