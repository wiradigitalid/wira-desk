# Story 2.1 Context Validation Report

## Target

- Story: `2-1-asynchronous-keyboard-hook-foundation`
- File: `2-1-asynchronous-keyboard-hook-foundation.md`
- Baseline: `3ce68f4`
- Workflow: `story-2-1-asynchronous-keyboard-hook`
- Validation date: 2026-07-23

## Result

**PASS WITH APPLIED FIXES.**

The story is implementation-ready for Cursor. It defines stable acceptance-criteria IDs, complete hook ownership, effective ring capacity, exact input semantics, lifecycle migration, scope boundaries, and measurable verification gates.

A dedicated fresh-context validation agent was launched but could not return findings because the account reached its agent usage limit. This limitation does not erase the completed validation evidence: the final checklist pass was cross-checked against two independent successful analyses—one planning/specification analysis and one current-code/lifecycle analysis—and every resulting gap was applied to the story. A later fresh-context re-check remains recommended but is not a material specification blocker.

## Source Coverage

| Source | Coverage |
|---|---|
| `epics.md` | Epic 2 goal, Story 2.1–2.3 boundaries, FR/NFR mapping |
| PRD | FR-6/FR-7, callback budget, 16-slot/full-drop behavior, throttle |
| SPEC | CAP-1, asynchronous hook, zero-allocation and non-goals |
| Architecture Spine | AD-1, AD-2, AD-7, AD-8, thread topology |
| Story 1.5 | Refresh ordering, Tier-3, recovery, debug seam, review follow-ups |
| Current Rust code | Actual `HHOOK` ownership, tray state, health timer, shared types/constants |
| Microsoft Learn | Low-level hook affinity, message-loop requirement, key-state caveat, thread-message queue, priority, high-resolution timing |

## Critical Findings Applied

### C-01 — Thread messages would be lost through ordinary dispatch

`PostThreadMessageW` produces messages with `MSG.hwnd == 0`; `DispatchMessageW` does not route them to a window procedure. The original draft required a Hook Thread queue but did not explicitly require direct handling.

**Applied fix:** Tasks and lifecycle notes now require the Hook Thread loop to process hook-check, debug, and stop messages directly before dispatching any window-associated message.

### C-02 — Exact-match AC contradicted throttle/full-drop behavior

The first draft said every exact match enqueued a command, while another AC correctly required matched events to be dropped when throttled or when the ring was full.

**Applied fix:** `AC-2.1-002` now defines one `Command::Cycle` decision and publication only when throttle and capacity gates permit.

### C-03 — Existing hook lifecycle could accidentally remain active

Current code installs and refreshes `HHOOK` on the main/tray thread. Merely adding `hook.rs` would create two lifecycle owners and could leave duplicate live hooks.

**Applied fix:** Story scope now requires moving startup retry, callback, refresh, failure count, debug seam, shutdown, and final unhook together, while removing all production `HHOOK` ownership from `main.rs` and `tray.rs`.

## Enhancements Applied

### E-01 — Effective ring capacity

The story now rejects the common sentinel-slot implementation that exposes only 15 of 16 slots. It specifies monotonic cursors, Acquire/Release publication, exact full behavior, wrap-around tests, and a cross-thread SPSC stress test.

### E-02 — Typematic and key-up semantics

OS typematic repeats now use the same 50ms throttle instead of an implicit one-command-per-physical-press rule. Matched main-key releases are swallowed; modifier events continue through the chain.

### E-03 — Story 1.5 shutdown and re-entrancy follow-ups

The story now requires direct shutdown ordering on the Hook Thread, heartbeat termination after posting fails, joining outside `WndProc`, and resolution of the broad mutable `TrayData` borrow across `menu::show`.

### E-04 — Operational readiness and observability

Normal tray startup and heartbeat begin only after hook-ready. Runtime verification must confirm effective thread priority and callback timing rather than assuming API calls succeeded.

### E-05 — Callback-safe failure behavior

The callback does not roll back a published ring cursor, log, block, or retry when `PostMessageW` fails. A later successful wake drains queued work; diagnostics occur only at a safe boundary outside the callback.

## Scope Validation

The following remain explicitly deferred:

- Window enumeration and same-app matching: Story 2.2
- Minimized, ghost, and hung-window policy: Story 2.3
- Monitor and virtual-desktop isolation: Story 3.1
- VM/RDP bypass: Story 3.2
- Snapping: Story 3.3
- Stack layout: Story 3.4
- Shortcut UI/live reload: Story 4.2
- Dependency upgrades and unrelated architecture cleanup

## Verification Readiness

The story provides gates for:

- Formatting, check, clippy, shared tests, elevated daemon tests, and release build
- Ring FIFO/capacity/wrap/no-overwrite/concurrency
- Exact modifiers, injected input, throttle boundary, swallowing, and wake-only signaling
- Dedicated thread identity and effective priority
- Callback duration below 10ms using QPC or equivalent evidence
- Story 1.5 heartbeat, Tier-3, recovery, toast, debug seam, and shutdown regressions

## Residual Risks

1. The fresh-context validator agent could not complete due account usage limits; rerun validation later if an additional independent layer is desired.
2. Global-hook and daemon tests require an elevated Windows session.
3. `PostMessageW` failure after successful publication can delay a queued command until the next successful wake; callback safety takes precedence over rollback/retry.
4. The working tree contains unrelated inter-agent workflow adoption changes; Cursor must isolate the production diff for Story 2.1.

## Final Verdict

**Implementation-ready.** No unresolved material design decision remains for Cursor. Production code and tests were not changed during story creation or validation.
