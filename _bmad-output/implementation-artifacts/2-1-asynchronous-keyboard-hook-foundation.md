---
baseline_commit: 3ce68f4
workflow_id: story-2-1-asynchronous-keyboard-hook
---

# Story 2.1: Asynchronous Keyboard Hook Foundation

Status: done

## Story

As a WinTick user,
I want keyboard shortcuts intercepted by a dedicated, time-critical hook thread,
so that Windows never drops the hook because window-processing work delayed the callback.

## Acceptance Criteria

### AC-2.1-001 — Dedicated hook ownership and startup

- **Given** the elevated daemon is starting
- **When** the keyboard subsystem initializes
- **Then** a dedicated Hook Thread creates its Win32 message queue, sets itself to `THREAD_PRIORITY_TIME_CRITICAL`, installs the global `WH_KEYBOARD_LL` hook, and owns the `HHOOK` for its entire lifetime
- **And** the existing startup policy remains: at most 5 installation attempts with a 1-second delay, followed by exactly one fatal `MessageBox` and process exit if initialization still fails
**And** only one production hook remains active after initialization or refresh.

### AC-2.1-002 — Exact shortcut matching

- **Given** the configured primary and fallback switcher shortcuts have been parsed before the callback starts
- **When** the user presses a physical key-down event
- **Then** an exact primary or fallback match produces exactly one `Command::Cycle` decision and publishes it only when the throttle and ring-capacity gates permit
- **And** any missing or additional modifier makes the event a non-match
- **And** unmatched, injected, or `code < 0` events are passed immediately to `CallNextHookEx`
**And** the callback does not call `GetAsyncKeyState`; modifier state is derived from the low-level key events owned by the Hook Thread.

### AC-2.1-003 — Bounded callback path

- **Given** the hook is active
- **When** `LowLevelKeyboardProc` handles an event
- **Then** the production callback performs no heap allocation, file or console I/O, logging, lock acquisition, sleep, config parsing, window enumeration, focus work, or blocking cross-process API call
- **And** the callback path through return or `CallNextHookEx` measures below 10ms in an elevated runtime verification
**And** any debug-only timing instrumentation records outside the user-facing logging path and is compiled out of release builds.

### AC-2.1-004 — Effective 16-slot SPSC ring

- **Given** the static Hook-to-Worker queue is empty
- **When** commands are pushed and popped
- **Then** it stores 16 usable FIFO entries as primitive `u8` values without heap allocation
- **And** the 17th push while full returns failure immediately, does not overwrite unread data, does not advance the producer cursor, and never blocks
**And** the consumer observes every successfully published value in order.

### AC-2.1-005 — Wake-only Worker notification

- **Given** a matched command passed the throttle gate and was published to the ring
- **When** the Hook Thread notifies the existing hidden-window/main thread
- **Then** it posts `WM_APP_COMMAND_READY` with `wParam = 0` and `lParam = 0` as a wake signal only
- **And** the Worker actor drains the ring until empty and converts each raw value with `Command::from_u8`
**And** this story performs placeholder dispatch only; window enumeration and focus changes remain Story 2.2 work.

### AC-2.1-006 — Anti-macro throttle and swallowing

- **Given** an exact configured shortcut is pressed repeatedly
- **When** the interval from the last shortcut accepted by the throttle is less than 50ms
- **Then** the new command is dropped before the ring push without blocking
- **And** an interval of exactly 50ms is accepted
- **And** unmatched events do not advance the throttle timestamp
- **And** matched events that are throttled or dropped because the ring is full remain swallowed so the WinTick shortcut does not leak into the foreground application
**And** the corresponding matched main-key release is swallowed, while modifier events continue through the hook chain.

### AC-2.1-007 — Health, recovery, and shutdown preservation

- **Given** Story 1.5 already monitors hook health every 10 seconds
- **When** hook ownership moves to the dedicated Hook Thread
- **Then** heartbeat checks, install-before-unhook refresh, keep-old-on-refresh-failure behavior, three-consecutive-failure Tier-3 escalation, one toast per failure episode, warning precedence, and recovery reset still work
- **And** the Hook Thread processes refresh and unhook operations on its own message-loop thread
- **And** shutdown cannot install a replacement hook after the final unhook
**And** the Hook Thread exits and is joined without leaving a live hook.

## Tasks / Subtasks

- [x] Task 1: Implement the effective-capacity SPSC command ring (AC: AC-2.1-004)
  - [x] Add `crates/daemon/src/ring.rs` with a static single-producer/single-consumer queue backed by `[AtomicU8; RING_BUFFER_CAPACITY]` and monotonic atomic producer/consumer cursors.
  - [x] Preserve all 16 usable slots; do not use an empty-sentinel cursor design that silently reduces capacity to 15.
  - [x] Publish slot data before the producer cursor with Release ordering; acquire the opposing cursor before reading/reusing a slot. Keep the hot path lock-free and allocation-free.
  - [x] Unit-test empty pop, FIFO order, wrap-around, 16 successful pushes, rejected 17th push, no overwrite, and reuse after pop.

- [x] Task 2: Move the complete hook lifecycle into a dedicated Hook Thread (AC: AC-2.1-001, AC: AC-2.1-007)
  - [x] Add `crates/daemon/src/hook.rs`; it owns Hook Thread state, parsed shortcuts, modifier state, throttle time, runtime refresh failure count, debug seam state, `HHOOK`, and the Win32 message loop.
  - [x] Force creation of the thread message queue before publishing the thread ID or starting heartbeat delivery.
  - [x] Set `THREAD_PRIORITY_TIME_CRITICAL` on the Hook Thread and treat failure as startup initialization failure.
  - [x] Move the 5-attempt/1-second startup installation policy from `main.rs` into Hook Thread initialization, reusing `HOOK_RETRY_MAX` and `HOOK_RETRY_DELAY_SECS`.
  - [x] Explicitly handle custom thread messages (`MSG.hwnd == 0`) inside the Hook Thread loop before `DispatchMessageW`; thread messages are not delivered to a window procedure by `DispatchMessageW`.
  - [x] Report ready/fatal initialization to the existing hidden-window thread through explicit `WM_APP_*` lifecycle messages; delay `add_icon`, heartbeat startup, and the normal operational state until the ready message.
  - [x] Remove `dummy_hook_proc` and all production `HHOOK` ownership/refresh/unhook logic from `main.rs` and `tray.rs`.

- [x] Task 3: Implement the bounded low-level callback and exact matching (AC: AC-2.1-002, AC-2.1-003)
  - [x] Load `Config::load_or_default(config_path())` and parse the primary/fallback `shared::Shortcut` values before installing the hook; the callback receives only compact `Copy` state.
  - [x] If a syntactically valid config contains an invalid shortcut string, use the corresponding `SwitcherConfig::default()` value and report a Tier-2 warning outside the callback; do not leave the hook without a valid binding.
  - [x] Track left/right Win, Ctrl, Alt, and Shift transitions from `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, and `WM_SYSKEYUP`; compare collapsed modifier groups exactly.
  - [x] Do not call `GetAsyncKeyState` from `LowLevelKeyboardProc`, because Windows invokes the callback before asynchronous key state is updated.
  - [x] Pass injected events through without enqueueing or changing physical modifier state.
  - [x] Return `CallNextHookEx` immediately for `code < 0`, unsupported messages, modifier-only events, and non-matches.
  - [x] Keep matching/throttle/ring logic in pure helper functions where practical so it can be unit-tested without installing a global hook.

- [x] Task 4: Apply throttle, enqueue, wake, and key swallowing (AC: AC-2.1-005, AC-2.1-006)
  - [x] Use a monotonic millisecond source for the 50ms production throttle and a pure helper with injected timestamps for deterministic boundary tests.
  - [x] Update the throttle timestamp only after an exact shortcut passes the throttle gate; an unmatched event must not affect it.
  - [x] On successful push, call `PostMessageW(worker_hwnd, WM_APP_COMMAND_READY, 0, 0)`; never duplicate the command in `WPARAM` or `LPARAM`.
  - [x] On full-buffer drop, return immediately without I/O or retry. Preserve the matched-key swallowing contract so the shortcut does not leak to the foreground app.
  - [x] Apply the same throttle policy to OS typematic key-down repeats; no separate one-command-per-physical-press rule is introduced. Repeats at `< 50ms` drop and repeats at `>= 50ms` may enqueue.
  - [x] Swallow the matching main-key release and allow modifier key events to continue through the chain.

- [x] Task 5: Add the Worker-side drain boundary without implementing cycling (AC: AC-2.1-005)
  - [x] Add `crates/daemon/src/worker.rs` as the command-drain/dispatch boundary executed by the existing hidden-window/main thread; do not spawn a redundant third command-processing thread.
  - [x] Handle `WM_APP_COMMAND_READY` in `tray::wndproc_impl`, drain until `pop()` returns `None`, convert through existing `shared::Command`, and ignore `Command::Nop`.
  - [x] Keep `Command::Cycle` dispatch as an explicit placeholder for Story 2.2. Do not add `EnumWindows`, executable matching, filtering, or focus calls here.
  - [x] Multiple wake messages after one drain must be harmless no-ops.
  - [x] Resolve Story 1.5 RF-1 while editing `wndproc_impl`: do not retain a broad `&mut TrayData` borrow across `menu::show` or another nested modal message pump; re-fetch or scope the pointer per message arm.

- [x] Task 6: Re-home heartbeat refresh, recovery, debug seam, and shutdown (AC: AC-2.1-007)
  - [x] Keep `health.rs` timer-only, but post `WM_APP_HOOK_CHECK` to the Hook Thread queue rather than asking the tray thread to touch `HHOOK`; break the timer loop when posting fails after Hook Thread shutdown.
  - [x] Preserve Story 1.5 install-before-unhook refresh on the Hook Thread: install the replacement first, retain the old hook on failure, and unhook the old handle only after replacement succeeds.
  - [x] Add explicit lifecycle messages for hook ready, initialization failure, Tier-3 dead, and recovery as needed; the tray thread remains the sole owner of visual state, warning latch, and toast guard.
  - [x] Move or forward the existing debug-only forced-refresh-failure seam so it controls Hook Thread-owned refresh state without introducing production shared mutable state.
  - [x] On `WM_DESTROY`, stop heartbeat delivery and post `WM_QUIT` (or an equivalent explicit stop message) to the Hook Thread. The Hook Thread must handle the message directly, unhook after all earlier queue entries, exit, and be joined after the tray message loop exits.
  - [x] Preserve Story 1.5 runtime evidence paths and update them only where ownership changed.

- [x] Task 7: Verification and regression gates (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace`
  - [x] `cargo clippy --workspace --all-targets`
  - [x] `cargo test -p shared`
  - [x] `cargo test -p daemon` in an elevated shell; all tests must pass.
  - [x] `cargo build --release -p daemon`; release binary remains below the 500KB hard limit.
  - [x] Elevated runtime: primary and fallback shortcuts match exactly; an extra modifier passes through; injected events pass through; accepted matches are swallowed.
  - [x] Elevated runtime: debug-only QPC-based or equivalent high-resolution evidence shows callback duration below 10ms without logging inside the callback.
  - [x] Elevated runtime: verify the Hook Thread's effective priority with `GetThreadPriority` or equivalent evidence rather than assuming `SetThreadPriority` succeeded.
  - [x] Elevated runtime: heartbeat refresh/recovery, three-failure Tier-3 escalation, one toast per episode, and clean exit remain valid after ownership migration.
  - [x] Confirm release build contains no debug timing or forced-failure seam.

## Dev Notes

### Current Implementation and Required Delta

The current Story 1.5 implementation installs `WH_KEYBOARD_LL` in `main.rs`, transfers the handle into `TrayData`, refreshes it from `tray::refresh_hook`, and unhooks it from `TrayData::cleanup`. That design was correct for Story 1.5 but does not satisfy Story 2.1's dedicated Hook Thread invariant.

Story 2.1 must migrate the lifecycle as one atomic architectural change:

1. The existing hidden top-level window/main thread remains the Worker actor and tray owner.
2. A new dedicated Hook Thread owns installation, callback execution, refresh, failure counting, and final unhook.
3. `health.rs` remains a timer actor and sends a thread message to the Hook Thread.
4. Hook-to-Worker commands travel only through the static ring; `WM_APP_COMMAND_READY` is wake-only.
5. The Worker boundary is introduced now, but actual window cycling begins in Story 2.2.

Do not leave the old `tray::refresh_hook` path active in parallel. Two lifecycle owners would create duplicate hooks, break shutdown ordering, and violate AD-1.

### Locked Thread Topology and Lifecycle

- **Hook Thread:** owns `HHOOK`, hook message queue, physical modifier state, parsed primary/fallback shortcuts, throttle state, ring producer, refresh fail count, and debug-only hook seam.
- **Existing hidden-window/main thread:** acts as Worker consumer and owns tray/menu/error visual state. It drains commands in response to `WM_APP_COMMAND_READY`.
- **Health timer thread:** sleeps for `HOOK_HEARTBEAT_SECS` and posts `WM_APP_HOOK_CHECK` to the Hook Thread queue. It never calls hook APIs.
- **No additional Worker thread:** the architecture diagram's Worker role maps to the existing hidden-window/main thread for this codebase.

The Hook Thread must call a User/GDI queue-creating API such as `PeekMessageW(..., PM_NOREMOVE)` before its thread ID is used with `PostThreadMessageW`. Normal heartbeat starts only after a hook-ready signal. Shutdown posts quit to that queue; the Hook Thread performs its final unhook after all earlier queue entries, exits, and is joined outside `WndProc`.

Messages posted with `PostThreadMessageW` have `MSG.hwnd == 0`; `DispatchMessageW` does not route them to a window procedure. The Hook Thread loop must match and process hook-check, debug, and stop messages directly, then dispatch only window-associated messages when applicable.

### Hook Callback Rules

- Handle only `HC_ACTION` key messages. For `code < 0`, delegate immediately.
- Use `KBDLLHOOKSTRUCT.vkCode`, message kind, and flags. Ignore injected events (`LLKHF_INJECTED`) for command generation.
- `GetAsyncKeyState` is prohibited inside the callback: Microsoft documents that asynchronous key state has not yet been updated when `LowLevelKeyboardProc` runs.
- Keep physical modifier state in Hook Thread-owned fixed-size state. Account for left/right variants and collapse them only for exact `Shortcut` comparison.
- Primary and fallback both map to existing `Command::Cycle`; do not hardcode their text or virtual-key values locally.
- A matched shortcut is swallowed even when throttle/full-buffer policy drops its command. Unmatched input always proceeds through `CallNextHookEx`.
- OS typematic key-down repeats use the same exact-match and 50ms throttle policy. A separate "once until key-up" enqueue latch is not part of this story; only the matched key-up swallow state is latched.
- No `debug_log`, `log::warn`, TOML access, formatting, allocation, mutex/RwLock, window enumeration, focus change, sleep, or retry loop is allowed in the callback.

### Ring Buffer Contract

Reuse `shared::constants::RING_BUFFER_CAPACITY` and existing `shared::Command`; do not duplicate either.

The ring is SPSC: Hook Thread is the only producer and hidden-window Worker is the only consumer. Use monotonic producer/consumer counters so all 16 array slots are usable. A sentinel-slot layout with `next_head == tail` is not acceptable because it exposes only 15 entries.

Required publication relation:

1. Producer writes the `AtomicU8` slot.
2. Producer publishes the advanced cursor with Release ordering.
3. Consumer observes the producer cursor with Acquire ordering before reading the slot.
4. Consumer publishes its advanced cursor with Release ordering.
5. Producer observes the consumer cursor with Acquire ordering before reusing the slot.

Tests must prove FIFO, wrap-around, full/drop behavior, and reuse. Do not claim lock-free behavior only from code inspection.

Add an SPSC stress test with one producer thread and one consumer thread to validate ordering/publication across many wrap-arounds; pure single-thread tests alone do not exercise the Acquire/Release contract.

### Shortcut and Throttle Semantics

- Load config and parse `Shortcut` before hook installation; live reload is Story 4.2.
- Default primary is `Win+Backtick`; default fallback is `Alt+Backtick`, sourced from `shared`.
- A valid TOML document with an invalid shortcut string falls back to the corresponding shared default and reports a Tier-2 warning outside the callback.
- Exactness includes absence of extra modifiers.
- The first exact match is accepted. For subsequent exact matches, `< 50ms` is rejected and `>= 50ms` is accepted.
- Unmatched events do not change the throttle clock.
- Full queue drops the newest command without overwriting old commands or waiting.
- `GetTickCount64` is suitable for the 50ms policy but not for proving a `<10ms` callback because its normal resolution is roughly 10–16ms. Use QPC or equivalent high-resolution external evidence for the callback gate.
- If `PostMessageW` fails after a command was already published, do not roll the producer cursor back or block/retry inside the callback. The command remains queued and the next successful wake drains it; record this as a runtime diagnostic outside the callback when a later safe boundary is available.

### Story 1.5 Behavior That Must Be Preserved

- Startup hook retry: 5 attempts, 1-second spacing, exactly one fatal popup on terminal failure.
- Runtime heartbeat: one check every 10 seconds.
- Refresh ordering: install replacement before unhooking the old handle.
- Refresh failure: keep the old handle and increment consecutive failure count.
- Tier 3: after 3 consecutive failures, tray becomes Critical and emits one toast per episode.
- Recovery: successful refresh after a critical episode restores Warning when `warning_latched`, otherwise Normal, and resets the toast episode guard.
- Debug forced-failure seam remains debug-only.
- Existing review follow-ups RF-1/RF-2 must not worsen: do not hold `&mut TrayData` across a nested modal message pump, and do not allow a late heartbeat to reinstall after final cleanup.

### Existing Components to Reuse

- `shared::Command` and `Command::from_u8`
- `shared::Shortcut` and `Shortcut::parse`
- `shared::Config`, `config_path`, `SwitcherConfig::default`
- `RING_BUFFER_CAPACITY`, `ANTI_MACRO_THROTTLE_MS`, `HOOK_RETRY_MAX`, `HOOK_RETRY_DELAY_SECS`, `HOOK_HEARTBEAT_SECS`, `HOOK_CHECK_FAIL_THRESHOLD`
- `WM_APP_COMMAND_READY`, `WM_APP_HOOK_CHECK`, `WM_APP_HOOK_DEAD`, `WM_APP_LOG_WARNING`
- `log::warn` for non-callback Tier-2 diagnostics
- `tray` warning/critical precedence and one-toast-per-episode logic
- `util::debug_log` only outside the callback

The daemon currently pins `windows-sys = 0.52`. The Architecture Spine's `0.61.x` table is stale. Do not upgrade dependencies as part of this story; add only a required feature flag supported by the pinned version if compilation proves it necessary.

### File Impact

- **NEW:** `crates/daemon/src/ring.rs`
- **NEW:** `crates/daemon/src/hook.rs`
- **NEW:** `crates/daemon/src/worker.rs`
- **MODIFY:** `crates/daemon/src/main.rs` — remove direct hook install/retry and `dummy_hook_proc`; start the tray/worker host.
- **MODIFY:** `crates/daemon/src/tray.rs` — remove `HHOOK` ownership/refresh; start and coordinate Hook Thread; drain Worker commands; preserve tray state.
- **MODIFY:** `crates/daemon/src/health.rs` — post heartbeat to the Hook Thread queue.
- **MODIFY:** `crates/shared/src/constants.rs` — add/clarify lifecycle message IDs without colliding with tray/debug IDs.
- **MODIFY IF REQUIRED:** `crates/daemon/Cargo.toml` — only for Win32 feature gates required by the pinned `windows-sys 0.52`.

### Scope and Non-Goals

In scope:

- Dedicated hook thread and message queue
- Exact primary/fallback shortcut matching
- Static SPSC ring and wake-only notification
- Throttle and swallowing policy
- Existing hidden-window Worker drain boundary
- Migration of health, recovery, debug seam, and shutdown ownership
- Unit, build, and elevated runtime verification

Explicitly out of scope:

- `EnumWindows`, executable/class matching, and focus activation — Story 2.2
- Minimized/ghost/hung-window filtering — Story 2.3
- Physical monitor and virtual-desktop filtering — Story 3.1
- VM/RDP bypass evaluation — Story 3.2
- Snapping command matching/dispatch — Story 3.3
- Overlapping stack layout — Story 3.4
- Shortcut UI and live config reload — Story 4.2
- Any visible overlay, animation, or switcher UI
- Dependency upgrades or Architecture Spine cleanup

### Testing Requirements

Pure unit tests must cover:

- Ring empty/full/FIFO/wrap-around/effective capacity/no overwrite
- Exact modifier comparison, including extra modifiers
- Primary/fallback match mapping to one `Command::Cycle`
- Injected and unsupported event pass-through decisions
- First press, 49ms reject, 50ms accept, and unmatched event not advancing throttle
- Matched throttled/full events retain swallow decision
- Lifecycle state transitions: ready, refresh success/failure threshold, recovery, shutdown

Elevated integration/runtime verification must cover:

- Dedicated Hook Thread identity and requested priority
- One live hook, not tray-owned duplicate hooks
- Callback latency below 10ms using high-resolution evidence without callback logging
- Worker drain after wake-only notification
- Startup fatal policy
- Existing heartbeat/Tier-3/recovery behavior
- Clean shutdown and no post-cleanup reinstall

### Previous Story Intelligence

Story 1.5 established the health timer, install-before-unhook refresh, failure threshold, warning latch, toast episode guard, and deterministic debug seam. Preserve these behaviors while moving ownership; do not recreate the tray state machine.

The Story 1.5 review also identified:

- `refresh_hook` re-entrancy risk when `TrayData` is borrowed across nested menu/message loops.
- A shutdown race when heartbeat work can arrive around cleanup.
- Existing constants for startup retry that `main.rs` currently does not reuse.
- `log::warn` is capability-only and ready for an appropriate non-fatal production trigger.

Story 2.1 is the correct place to resolve the hook ownership and shutdown items because it changes the lifecycle boundary.

### Git Intelligence

- Baseline: `3ce68f4` (`feat(daemon): verify Story 1.5 + review fixes`)
- `3ce68f4` owns the latest refresh ordering, tray severity precedence, recovery, and debug-seam behavior.
- `0c01ea3` introduced the Story 1.5 error/health/tray foundation.
- `eee0141` enriched Story 1.5 context and demonstrated the expected story-quality gate.

Implementation must diff against `3ce68f4`; the previous story baseline `720d155` is obsolete.

### Latest Technical Information

- Microsoft documents that `LowLevelKeyboardProc` executes on the thread that installed the hook and therefore requires that thread to run a message loop.
- Microsoft recommends a dedicated hook thread that hands work to a worker and returns immediately.
- Windows can silently remove a timed-out low-level hook, so the existing heartbeat policy remains required.
- `GetAsyncKeyState` cannot provide the new asynchronous state from inside `LowLevelKeyboardProc`; event-owned modifier tracking is required.
- `PostThreadMessageW` fails before the target thread has a message queue; create the queue before publishing readiness.
- `THREAD_PRIORITY_TIME_CRITICAL` is a very high scheduling priority. The Hook Thread must remain event-driven and perform only brief bounded work.

### Project Structure Notes

The Architecture Spine already reserves `daemon/hook.rs` and `daemon/worker.rs`. Adding `ring.rs` is a focused implementation of AD-2/NFR5. The existing hidden window remains top-level rather than message-only because Story 1.3 requires `TaskbarCreated` broadcast reception.

No production documentation, UX, settings code, or window-cycling implementation belongs in this story.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.1]
- [Source: `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md` — FR-6, FR-7, §4.1]
- [Source: `_bmad-output/specs/spec-wintick/SPEC.md` — CAP-1 and Constraints]
- [Source: `_bmad-output/specs/spec-wintick/conventions.md` — Local Configuration]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-WinTick-2026-07-06/ARCHITECTURE-SPINE.md` — AD-1, AD-2, AD-7, AD-8, Daemon Internal Threading]
- [Source: `_bmad-output/implementation-artifacts/1-5-3-tier-error-protocol-and-tray-state-machine.md` — Completion Notes and Review Follow-ups]
- [Source: `crates/daemon/src/main.rs`, `tray.rs`, `health.rs`, `log.rs`]
- [Source: `crates/shared/src/commands.rs`, `shortcut.rs`, `config.rs`, `constants.rs`]
- [Microsoft: LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)
- [Microsoft: SetWindowsHookExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- [Microsoft: PostThreadMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew)
- [Microsoft: KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)
- [Microsoft: SetThreadPriority](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadpriority)
- [Microsoft: QueryPerformanceCounter](https://learn.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancecounter)

## Dev Agent Record

### Agent Model Used

- Cursor implementation handovers H5/H8 (production code)
- Antigravity `/bmad-code-review` acceptance through H9
- Codex ChatGPT `close-spek` final documentation and traceability close-out

### Debug Log References

- Elevated `verify-story-2-1-runtime.ps1` v2 verification (11/11 PASS, 2026-07-24 workspace evidence)
- Elevated `cargo test -p daemon` verification (19/19 PASS, 2026-07-24 workspace evidence)
- Debug-only callback timing evidence remained outside release artifacts and passed the `<10ms` gate (`max_us=9`)

### Completion Notes List

- Dedicated Hook Thread ownership, wake-only `WM_APP_COMMAND_READY`, and effective 16-slot SPSC ring were implemented and verified against all seven stable AC IDs.
- Accepted review patches remained included at close time: debug constants centralized in `shared::constants`, `CallNextHookEx` handle usage unified, and `swallow_win_release` added so swallowed `Win+Backtick` no longer leaks the Start Menu on key-up.
- Verification evidence at accepted review target: `cargo fmt -p daemon -p shared --check`, `cargo check --workspace`, `cargo clippy -p daemon -p shared --all-targets`, `cargo test -p shared --lib` (10/10), `cargo test -p daemon` (19/19, elevated), and `verify-story-2-1-runtime.ps1` v2 (11/11, elevated).
- Deferred items preserved explicitly in `deferred-work.md`: health-thread shutdown sleep latency and `ModifierState` desync risk on missed key-up events.
- Story 2.1 was closed through inter-agent workflow `story-2-1-asynchronous-keyboard-hook`; Codex finalized documentation and operational chain artifacts only.

### File List

- `crates/daemon/src/main.rs`
- `crates/daemon/src/tray.rs`
- `crates/daemon/src/health.rs`
- `crates/daemon/src/hook.rs`
- `crates/daemon/src/ring.rs`
- `crates/daemon/src/worker.rs`
- `crates/daemon/src/util.rs`
- `crates/shared/src/constants.rs`
- `crates/daemon/Cargo.toml`
- `verify-story-2-1-runtime.ps1`
- `_bmad-output/implementation-artifacts/deferred-work.md`
