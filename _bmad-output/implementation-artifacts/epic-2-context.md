# Epic 2 Context: Core Window Cycling Experience

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Deliver instant, app-specific window focus cycling via precise keyboard shortcuts, with hook-side interception fast enough that Windows never drops the low-level hook. Epic 2 builds the input pipeline (hook thread, ring buffer, worker boundary) before window enumeration and filtering in later stories.

## Stories

- Story 2.1: Asynchronous Keyboard Hook Foundation
- Story 2.2: Stateless Z-Order & App-Specific Matching
- Story 2.3: Status Filtering & UX Honesty

## Requirements & Constraints

- Global low-level keyboard hook on a dedicated time-critical thread with a message loop.
- Hook callback must stay bounded (no allocation, I/O, locks, or blocking work).
- Commands cross to the worker via a fixed 16-slot SPSC ring carrying raw `u8` values only.
- Anti-macro throttle at 50ms; exact shortcut matching including modifier absence.
- Preserve Story 1.5 hook health, Tier-3 escalation, recovery, and clean shutdown.

## Technical Decisions

- Existing hidden-window/main thread is the Worker; no extra command thread.
- Hook Thread owns all `HHOOK` lifecycle; health timer posts to Hook Thread queue only.
- `WM_APP_COMMAND_READY` is wake-only; worker drains the ring.
- `windows-sys 0.52` pinned; no dependency upgrades in this epic tranche.

## Cross-Story Dependencies

- Depends on Epic 1 (elevation, tray, error protocol, heartbeat foundation).
- Stories 2.2+ require 2.1 ring/worker boundary before adding enumeration and focus logic.
