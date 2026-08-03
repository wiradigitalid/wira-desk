---
baseline_commit: eddb21a
workflow_id: story-2-4-eligibility-policy
---

# Story 2.4: Eligible-Window Filtering and UX Honesty

Status: review

## Story

As a desktop user,
I want cycling to include only real visible application windows while still exposing a hung window,
so that the cycle remains useful without hiding application failures from me.

## Acceptance Criteria

### AC-2.4-001 — Base decisions
Visible non-iconic top-level application windows remain eligible; hidden, minimized, `WS_EX_TOOLWINDOW`, ghost-class, and defined shell-overlay windows are excluded.

### AC-2.4-002 — UX honesty for hung windows
A Not Responding real application window remains eligible; no responsiveness probe, automated skip, restore, maximize, or ghost-surrogate replacement occurs.

### AC-2.4-003 — Purity audit
No live enumeration, process query, focus operation, `SendMessage`, or `GetWindowText`; no dependency on monitor, virtual-desktop, VM/RDP, arrangement, or Settings behavior.

### AC-2.4-004 — Table-driven determinism
Every frozen fixture and combinations of exclusion facts produce deterministic decisions; a synthetic hung-window fact remains eligible; tests require no sibling-lane implementation.

## Dev Notes

### Implementation

`crates/daemon/src/cycling/eligibility.rs`. `WindowEligibility` implements
`EligibilityPolicy`; `evaluate_facts()` is the underlying pure function. The
module imports nothing from `windows_sys` at all — that import-level absence is
the audit evidence for AC-2.4-003, stronger than a code-reading claim.

`evaluate_facts()` takes the active *identity* rather than the whole
`ActiveContext`, so the policy cannot see the foreground handle and therefore
cannot drift into selection concerns.

### Frozen Exclusion Precedence

When a window matches several exclusion facts at once, first match wins:

1. ghost class → 2. shell surface → 3. hidden → 4. iconic → 5. tool window →
6. unavailable identity → 7. different application

This ordering is part of the contract. Without it, combination cases would be
implementation-defined and two correct-looking implementations could disagree.
Five dedicated tests pin the precedence pairwise.

### How the Hung-Window Requirement Is Met

`WindowFacts` carries no responsiveness field, so a hung window and a responsive
window are *literally the same value* to this policy — it cannot probe or skip
even if someone later wanted it to. The hung case is still handled correctly
because Windows creates a separate `Ghost` class window, which is excluded,
while the real application window stays eligible. `contract_exposes_no_responsiveness_input`
asserts this indistinguishability directly.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 17 unit tests, all desktop-free. Two of them cross-check the production policy
  against `fixtures::expected_decisions()` and against Story 2.2's
  `ReferencePolicy`, so drift in either direction fails the build.
- Static gates PASS: workspace clippy exit 0; test binary compiles.
- **Test execution NOT performed** — requires elevation (`os error 740`).
  Deferred to the user. These tests need no desktop, only the elevated launch.

### File List
- `crates/daemon/src/cycling/eligibility.rs` (new)
- `crates/daemon/src/cycling/mod.rs` (modified — submodule declaration)
