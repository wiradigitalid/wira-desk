---
title: '2-1-asynchronous-keyboard-hook-foundation'
type: 'feature'
created: '2026-07-23'
status: blocked
baseline_revision: '3ce68f46efcd316025572fa27dd612c1164b0f9d'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/implementation-artifacts/2-1-asynchronous-keyboard-hook-foundation.md'
warnings:
  - dirty-working-tree
  - branch-mismatch-feat-story-1.5
---

<intent-contract>

## Intent

**Problem:** Story 1.5 installs `WH_KEYBOARD_LL` on the tray message-loop thread with a dummy callback; keyboard commands, SPSC ring IPC, and dedicated hook-thread ownership required by AD-1/AD-2 are missing.

**Approach:** Migrate hook lifecycle to a dedicated Hook Thread with bounded `LowLevelKeyboardProc`, static 16-slot ring to the existing hidden-window Worker, wake-only `WM_APP_COMMAND_READY`, and re-home heartbeat refresh/shutdown while preserving Story 1.5 severity behavior.

## Boundaries & Constraints

**Always:** Hook Thread owns `HHOOK`; callback is allocation-free; exact `Shortcut` match; 50ms throttle; 16-slot FIFO SPSC; Story 1.5 health/Tier-3/recovery preserved; no product doc edits from this run.

**Block If:** Ordered SSOT blob fingerprints change without handover (`CHAIN_INTEGRITY_BLOCKED`); material upstream spec ambiguity blocks safe coding (`coding-to-spek`).

**Never:** Story 2.2+ enumeration/focus; config live reload; `windows-sys` upgrade; editing validation report or story corpus from Cursor.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| HAPPY_PATH | Exact primary shortcut | One `Cycle` enqueued + wake | Swallow matched keys |
| THROTTLE | Repeat < 50ms | Drop command, still swallow | No ring push |
| RING_FULL | 16 pending commands | 17th push fails, swallow | No overwrite |
| INJECTED | `LLKHF_INJECTED` | Pass through | No modifier mutation |
| HOOK_FAIL_START | 5 install failures | Fatal dialog + exit | Same as Story 1.2 policy |

</intent-contract>

## Code Map

- `crates/daemon/src/main.rs` -- remove inline hook install; start tray host only
- `crates/daemon/src/tray.rs` -- Worker/tray messages; no `HHOOK` ownership
- `crates/daemon/src/health.rs` -- heartbeat to Hook Thread queue
- `crates/daemon/src/ring.rs` -- NEW SPSC ring
- `crates/daemon/src/hook.rs` -- NEW Hook Thread + callback
- `crates/daemon/src/worker.rs` -- NEW drain boundary
- `crates/shared/src/constants.rs` -- lifecycle `WM_APP_*` IDs

## Tasks & Acceptance

**Execution:**
- [x] `crates/daemon/src/ring.rs` -- implement 16-slot SPSC + unit/stress tests -- AC-2.1-004
- [x] `crates/daemon/src/hook.rs` -- Hook Thread, callback, refresh, lifecycle -- AC-2.1-001/002/003/006/007
- [x] `crates/daemon/src/worker.rs` -- drain on `WM_APP_COMMAND_READY` -- AC-2.1-005
- [x] `crates/daemon/src/main.rs` -- drop dummy hook path -- AC-2.1-001
- [x] `crates/daemon/src/tray.rs` -- integrate hook start/stop + tray messages -- AC-2.1-005/007
- [x] `crates/daemon/src/health.rs` -- `PostThreadMessageW` to hook -- AC-2.1-007
- [x] `crates/shared/src/constants.rs` -- hook lifecycle message constants -- coordination
- [ ] Verification gates -- fmt/check/clippy/test/build -- `cargo test -p daemon` PASS 17/17 elevated; manual runtime gates still open

**Acceptance Criteria:**
- Given daemon startup, when keyboard subsystem initializes, then a dedicated Hook Thread owns `WH_KEYBOARD_LL` with 5×1s fatal policy (AC-2.1-001).
- Given configured shortcuts, when physical key-down matches exactly, then one throttled `Cycle` may enqueue; extra modifiers pass through (AC-2.1-002).
- Given hook active, when callback runs, then no forbidden work and release builds omit debug timing (AC-2.1-003).
- Given ring operations, when 17th push while full, then fail without overwrite (AC-2.1-004).
- Given published command, when Worker receives wake, then ring drained via `Command::from_u8` (AC-2.1-005).
- Given repeated shortcut, when interval <50ms then drop; =50ms accept; swallow policy holds (AC-2.1-006).
- Given Story 1.5 health path, when ownership migrates, then heartbeat/Tier-3/recovery/shutdown preserved (AC-2.1-007).

## Spec Change Log

## Auto Run Result

Status: blocked  
Blocking condition: implementation verification failed — `cargo test -p daemon` and elevated runtime gates not executed (os error 740; requires Administrator session).

Implementation code for Story 2.1 is present on working tree vs baseline `3ce68f4`. Handover `002-coding-to-review.md` created with STATUS INCOMPLETE for Antigravity after elevated verification is recorded.

Step-04 adversarial review not executed (workflow constraint: BMAD review only via Antigravity `/bmad-code-review`).

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: clean
- `cargo check --workspace` -- expected: clean
- `cargo clippy --workspace --all-targets` -- expected: clean
- `cargo test -p shared` -- expected: pass
- `cargo test -p daemon` -- expected: pass (elevated on Windows)
- `cargo build --release -p daemon` -- expected: binary < 500KB

**Manual checks (if no CLI):**
- Elevated runtime: shortcut match, latency, heartbeat Tier-3, clean shutdown
