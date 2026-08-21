---
type: contract
component: window-management
lc: LC-hook-thread
direction: internal
created: '2026-08-21'
updated: '2026-08-21'
---

# Contract — Hook health heartbeat

## Source of truth

`crates/shared/src/constants.rs` — `HOOK_HEARTBEAT_SECS`, `HOOK_CHECK_FAIL_THRESHOLD`, `HOOK_RETRY_MAX`,
`WM_APP_HOOK_CHECK`, `WM_APP_HOOK_REFRESH_OK`, `WM_APP_HOOK_DEAD`, `WM_APP_HOOK_SHUTDOWN`.
Behaviour: `crates/daemon/src/health.rs`, `crates/daemon/src/hook.rs`, `crates/daemon/src/tray.rs`.

## Purpose

Detect a hook that the OS, an endpoint protection product, or a `LowLevelHooksTimeout` violation has
silently removed, and recover it without user action. A dead `WH_KEYBOARD_LL` hook is indistinguishable
from an idle one — nothing fails, no key arrives — so liveness has to be asked for rather than waited for.

## Operations

| Operation | Direction | Purpose | Realizes |
| --- | --- | --- | --- |
| `PostThreadMessageW(hook_tid, WM_APP_HOOK_CHECK, 0, 0)` | `health::heartbeat` → Hook Thread | Ask the hook thread to verify and, if needed, renew its registration. Every `HOOK_HEARTBEAT_SECS` (10 s) | AD-8 |
| `PostMessageW(worker_hwnd, WM_APP_HOOK_REFRESH_OK, 0, 0)` | Hook Thread → Worker | Registration is healthy again; clear Tier 3 and reset the toast latch | AD-7, AD-8 |
| `PostMessageW(worker_hwnd, WM_APP_HOOK_DEAD, 0, 0)` | Hook Thread → Worker | `HOOK_CHECK_FAIL_THRESHOLD` (3) consecutive renewals failed; escalate to Tier 3 | AD-7, AD-8 |
| `PostThreadMessageW(hook_tid, WM_APP_HOOK_SHUTDOWN, 0, 0)` | Main Thread → Hook Thread | Unhook and leave the message loop so the thread can be joined | AD-1 |

The renewal call itself (`SetWindowsHookExW`) MUST run **on the hook thread**. A hook is owned by the
thread that installed it, so re-registering it from the heartbeat thread would install a hook nobody
drains — the failure this contract is least likely to detect, because everything would still look alive.

## Error behaviour

| Condition | Response | Caller should |
| --- | --- | --- |
| Renewal fails, `fail_count < 3` | Increment `hook_check_fail_count`, **retain the prior `HHOOK`** | Nothing. The old handle may still be delivering; discarding it early converts a transient refusal into an outage |
| Renewal fails, `fail_count >= 3` | Post `WM_APP_HOOK_DEAD`; tray goes `Critical`; exactly one toast if `!hook_dead_toast_sent` | Nothing. The heartbeat continues — this is not a terminal state |
| Renewal later succeeds | Unhook the stale handle, reset `hook_check_fail_count = 0`, post `WM_APP_HOOK_REFRESH_OK` | Restore `Normal`, or `Warning` when `warning_latched`; reset `hook_dead_toast_sent = false` |
| Install fails at startup, all `HOOK_RETRY_MAX` (5) attempts | Post `WM_APP_HOOK_INIT_FAILED`; Tier 1 modal; process exits | Nothing — startup failure is fatal by AD-7, and distinct from runtime death |
| Hook thread id not yet known | No heartbeat runs; the thread is started only after `WM_APP_HOOK_READY` carries the id | Nothing |

## Compatibility

`HOOK_CHECK_FAIL_THRESHOLD` sets the worst-case detection latency at
`HOOK_HEARTBEAT_SECS × HOOK_CHECK_FAIL_THRESHOLD` = 30 s. Lowering the threshold to 1 would report
every transient refusal as a dead hook; raising the interval widens the window in which shortcuts are
silently dead. Either change moves a number the user feels, so both belong in the spine (AD-8), not here.

## Constraints

- Tier 3 fires **at most one toast per death**, latched by `hook_dead_toast_sent` and cleared only by a
  successful refresh (LBR-WM-5).
- The heartbeat MUST NOT terminate on reaching Tier 3. Recovery is restart-free precisely because the
  loop outlives the escalation.
- The hook callback itself does no work here: heartbeat handling happens in the hook thread's message
  loop, never inside the `HOOKPROC` (NFR-2, NFR-3).
