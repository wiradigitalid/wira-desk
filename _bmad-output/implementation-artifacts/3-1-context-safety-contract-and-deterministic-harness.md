---
baseline_commit: 9b38a3b
workflow_id: story-3-1-context-safety-contract
---

# Story 3.1: Context-Safety Contract and Deterministic Harness

Status: review

## Story

As a multi-context desktop user,
I want WinTick's spatial and shortcut-passthrough decisions governed by one stable policy contract,
so that independently delivered context-safety components behave consistently.

## Acceptance Criteria

### AC-3.1-001 — Contract shape
Defines immutable inputs and explicit outcomes for spatial eligibility and foreground bypass classification; introduces no Z-order cache, cross-actor shared mutable state, or second command-payload path.

### AC-3.1-002 — Backward-compatible schema
A config containing only `vm_bypass.bypass_processes` retains its documented defaults; missing `bypass_classes` receives its documented default; process-basename and window-class identifiers remain independently configurable.

### AC-3.1-003 — Normalization outside the callback
Identifiers are normalized into Hook Thread-owned immutable policy data before Hook activation; callback evaluation requires no parsing, allocation, file I/O, logging, or lock acquisition.

### AC-3.1-004 — Deterministic harness
Fake monitor, virtual-desktop, and foreground-identity adapters cover same/different monitors, current/non-current virtual desktops, process matches, class matches, confirmed non-matches, and adapter failures; **spatial uncertainty produces an ineligible decision**; **foreground-identity uncertainty produces a conservative passthrough decision**.

### AC-3.1-005 — Lane ownership
`context/spatial.rs` and `context/virtual_desktop.rs` → Story 3.2; `context/vm_bypass.rs` → Story 3.3; `context/mod.rs`, `worker.rs`, `hook.rs`, and final composition reserved for Story 3.4 convergence.

### AC-3.1-006 — Independent verification
Tests pass without installing a keyboard hook, creating a COM object, enumerating windows, or changing foreground focus.

## Dev Notes

### The Two Decisions Fail in Opposite Directions

This is the central idea of the contract, and getting it backwards would be a
user-visible bug in either direction:

| Decision | On uncertainty | Why |
| --- | --- | --- |
| Spatial eligibility | **Ineligible** (fail closed) | Guessing throws focus across the user's workspace |
| Foreground bypass | **Passthrough** (fail open) | Guessing swallows a keystroke inside a VM or RDP session |

`no_uncertainty_combination_is_ever_eligible` enumerates all 12 combinations of
origin/candidate/desktop knowledge and asserts eligibility only for the fully
known case, so the closed-failure property cannot regress silently.

### `Option` Means "Unknown", Never "None Of Them"

`SpatialContext::origin_monitor`, `SpatialFacts::candidate_monitor`, and
`on_current_virtual_desktop` are all `Option`. In every case `None` means the
lookup failed — not "no monitor" or "not on a desktop". The contract documents
this explicitly because the two readings lead to opposite decisions.

### Confirmed Non-Match vs Uncertainty

`BypassDecision::Intercept` is returned **only** when both the process and the
class are known and neither matches. If either lookup failed, nothing can be
confirmed and the key passes through. This is what the AC's phrase "confirmed
non-matches" requires — an unknown foreground is not a non-match.

One deliberate refinement: a positive match still wins even when the *other*
identifier is unknown. Otherwise a VM window whose class lookup happened to
fail would stop bypassing, which is exactly the regression this story exists to
prevent. `a_match_still_wins_when_the_other_identifier_is_unknown` pins it.

### Callback-Path Purity

`BypassPolicy::from_config()` does all trimming, lowercasing, and empty-entry
filtering. `BypassPolicy::classify()` then compares with `eq_ignore_ascii_case`
against pre-normalized entries — no allocation, no parsing, no lock, no I/O
(AC-3.1-003). `policy_normalizes_at_construction_not_evaluation` asserts the
normalization happened up front.

### Schema Compatibility Is Verified From the Consuming Side

Story 2.6 froze `bypass_classes` in `shared`; this story re-verifies the same
guarantee through `BypassPolicy`, so a future schema change breaks a test in
both the producing and consuming crate.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 24 unit tests, **all desktop-free**: no hook, no COM, no enumeration, no
  focus change. The module imports no `windows_sys` at all, which is the
  structural evidence for AC-3.1-006.
- Static gates PASS: `cargo clippy --workspace --all-targets` exit 0 in both
  debug and release; test binary compiles.
- **Test execution NOT performed** — `cargo test -p daemon` needs elevation
  (`os error 740`). These tests need no desktop, only the elevated launch.
- `context/spatial.rs`, `context/virtual_desktop.rs`, and `context/vm_bypass.rs`
  are deliberately **not created** — they belong to Stories 3.2 and 3.3.
- Sequencing caveat: `epics.md` gates Epic 3 behind an accepted Story 2.6, which
  is still `in-progress`. This story was built anyway because it depends only on
  the *frozen contracts* from Epic 2 — and those are the one part of Epic 2 with
  real test evidence (`cargo test -p shared` 18/18). It does not depend on any
  unverified Epic 2 runtime behaviour.

### File List
- `crates/daemon/src/context/mod.rs` (new)
- `crates/daemon/src/main.rs` (modified — `mod context;`)
