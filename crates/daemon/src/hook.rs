//! Dedicated Hook Thread: `WH_KEYBOARD_LL`, callback, refresh, lifecycle.
//! Threading: the hook callback runs only on this thread's message loop.

#[cfg(debug_assertions)]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use shared::config_path;
use shared::constants::{
    ANTI_MACRO_THROTTLE_MS, CAPTURE_LEASE_NONE, CAPTURE_LEASE_OBSERVE, CAPTURE_LEASE_RECORD,
    HOOK_CHECK_FAIL_THRESHOLD, HOOK_RETRY_DELAY_SECS, HOOK_RETRY_MAX, WM_APP_COMMAND_READY,
    WM_APP_CONFIG_SNAPSHOT, WM_APP_HOOK_CHECK, WM_APP_HOOK_DEAD, WM_APP_HOOK_INIT_FAILED,
    WM_APP_HOOK_LEASE, WM_APP_HOOK_READY, WM_APP_HOOK_REFRESH_OK, WM_APP_HOOK_SHUTDOWN,
    WM_APP_RECORDED_CHORD,
};
#[cfg(debug_assertions)]
use shared::constants::{
    WM_APP_DEBUG_DUMP_HOOK_LATENCY, WM_APP_DEBUG_SIMULATE_SHORTCUT, WM_APP_DEBUG_TOGGLE_HOOK_FAIL,
};
use shared::{Command, Config, Shortcut, SwitcherConfig};

use windows_sys::Win32::Foundation::{CloseHandle, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, GetExitCodeProcess, OpenProcess, SetThreadPriority,
    PROCESS_QUERY_LIMITED_INFORMATION, THREAD_PRIORITY_TIME_CRITICAL,
};

/// `STILL_ACTIVE` (`winbase.h`) — not re-exported by `windows-sys` 0.52 under
/// `Win32::System::Threading`, so it is named here rather than left as a bare
/// magic number at the call site.
const STILL_ACTIVE: u32 = 259;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PeekMessageW, PostMessageW, PostQuitMessage,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

// Measurement-only seam; gating the import keeps the release build free of
// unused-import noise, following the same shape as `tray.rs`.
#[cfg(debug_assertions)]
use shared::constants::WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED;

use crate::context::vm_bypass::HookIdentityCollector;
use crate::context::BypassPolicy;
use crate::ring;
use crate::util::debug_log;

#[cfg(debug_assertions)]
static DEBUG_FORCE_HOOK_FAIL: AtomicBool = AtomicBool::new(false);

#[cfg(debug_assertions)]
static QPC_FREQUENCY: AtomicU64 = AtomicU64::new(0);

#[cfg(debug_assertions)]
static QPC_MAX_NS: AtomicU64 = AtomicU64::new(0);

#[cfg(debug_assertions)]
static QPC_SAMPLES: AtomicU64 = AtomicU64::new(0);

#[cfg(debug_assertions)]
fn qpc_frequency() -> u64 {
    let cached = QPC_FREQUENCY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let mut freq = 0i64;
    // SAFETY: `&mut freq` is a unique pointer to a live, initialised local of the width the
    // API writes. The return value is ignored because `freq` starts at zero and callers
    // already treat a zero frequency as "no measurement", so a failure degrades to a
    // skipped sample rather than a division by zero.
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut freq);
    }
    let f = freq.max(0) as u64;
    QPC_FREQUENCY.store(f, Ordering::Relaxed);
    f
}

#[cfg(debug_assertions)]
fn qpc_now() -> i64 {
    let mut counter = 0i64;
    // SAFETY: `&mut counter` is a unique pointer to a live, initialised local of the width
    // the API writes, and the pointer is not retained past the call. Safe to call from the
    // hook callback: no allocation, no lock, no blocking.
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceCounter(&mut counter);
    }
    counter
}

#[cfg(debug_assertions)]
fn record_key_path_duration(start: i64) {
    let end = qpc_now();
    let elapsed = (end - start).max(0) as u64;
    let freq = qpc_frequency();
    if freq == 0 {
        return;
    }
    let ns = elapsed.saturating_mul(1_000_000_000) / freq;
    QPC_SAMPLES.fetch_add(1, Ordering::Relaxed);
    let mut cur = QPC_MAX_NS.load(Ordering::Relaxed);
    while ns > cur {
        match QPC_MAX_NS.compare_exchange_weak(cur, ns, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(v) => cur = v,
        }
    }
}

#[cfg(debug_assertions)]
fn reset_qpc_stats() {
    QPC_MAX_NS.store(0, Ordering::Relaxed);
    QPC_SAMPLES.store(0, Ordering::Relaxed);
}

#[cfg(debug_assertions)]
fn dump_qpc_stats() {
    let max_ns = QPC_MAX_NS.load(Ordering::Relaxed);
    let samples = QPC_SAMPLES.load(Ordering::Relaxed);
    let max_us = max_ns / 1_000;
    crate::util::append_debug_trace(&format!("HOOK_LATENCY: max_us={max_us} samples={samples}"));
}

#[cfg(not(debug_assertions))]
#[allow(dead_code)]
pub fn debug_force_hook_fail() -> bool {
    false
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub fn debug_force_hook_fail() -> bool {
    DEBUG_FORCE_HOOK_FAIL.load(Ordering::Relaxed)
}

#[cfg(debug_assertions)]
pub fn debug_toggle_hook_fail() {
    let was = DEBUG_FORCE_HOOK_FAIL.fetch_xor(true, Ordering::Relaxed);
    let msg = format!("DEBUG_TOGGLE: force_hook_fail {was} → {}", !was);
    debug_log(&format!("Wira Desk {msg}"));
    crate::util::append_debug_trace(&msg);
}

/// Set only by the measurement harness, via
/// `WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED`. Never set during normal operation.
#[cfg(debug_assertions)]
static DEBUG_ACCEPT_INJECTED: AtomicBool = AtomicBool::new(false);

#[cfg(debug_assertions)]
pub fn debug_toggle_accept_injected() {
    let was = DEBUG_ACCEPT_INJECTED.fetch_xor(true, Ordering::Relaxed);
    let msg = format!("DEBUG_TOGGLE: accept_injected {was} → {}", !was);
    debug_log(&format!("Wira Desk {msg}"));
    crate::util::append_debug_trace(&msg);
}

/// Whether the callback should process synthetic (`LLKHF_INJECTED`) input.
/// Constant `false` in release, so the branch below folds away and the shipped
/// callback is unchanged. Kept as a function rather than an inline `cfg!` so the
/// release build cannot accidentally read an atomic on the keystroke path.
#[cfg(not(debug_assertions))]
#[inline(always)]
fn accept_injected() -> bool {
    false
}

/// One relaxed atomic load. No allocation, no lock, no logging — 's bounded
/// callback contract is unaffected.
#[cfg(debug_assertions)]
#[inline]
fn accept_injected() -> bool {
    DEBUG_ACCEPT_INJECTED.load(Ordering::Relaxed)
}

const LLKHF_INJECTED: u32 = 0x10;

const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;
const VK_LSHIFT: u32 = 0xA0;
const VK_RSHIFT: u32 = 0xA1;
#[cfg(any(debug_assertions, test))]
const VK_BACKTICK: u32 = 0xC0;

/// Settings' hidden receiver window (`SETTINGS_HOOK_WINDOW_CLASS` /
/// `_TITLE`), resolved once by `tray.rs` when a lease arms — never per
/// keystroke. Read directly from the Hook thread's callback via a relaxed
/// atomic load, which is what keeps reporting a chord as cheap as the
/// existing `PostMessageW` wake-up to the Worker: no `FindWindowW`, no
/// allocation, no lock, on the callback path.
static REPORT_TARGET_HWND: AtomicIsize = AtomicIsize::new(0);

/// Set by `tray.rs` off the callback path: once when a lease arms (the
/// resolved Settings receiver window), and back to 0 on every disarm path.
pub fn set_report_target(hwnd: HWND) {
    REPORT_TARGET_HWND.store(hwnd, Ordering::Relaxed);
}

/// Post the chord the hook just observed back to Settings, while the observe
/// or record lease is armed. A no-op if no target has been resolved (lease is
/// `none`, or armed but the receiver window was never found).
fn report_recorded_chord(vk: u32, mods: ModifierState) {
    let target = REPORT_TARGET_HWND.load(Ordering::Relaxed) as HWND;
    if target == 0 {
        return;
    }
    let packed: u32 = u32::from(mods.ctrl)
        | (u32::from(mods.win) << 1)
        | (u32::from(mods.alt) << 2)
        | (u32::from(mods.shift) << 3);
    // SAFETY: `target` is read from `REPORT_TARGET_HWND`, which only ever
    // holds a value `tray.rs` obtained from a live `FindWindowW` or 0.
    // `PostMessageW` compares the handle rather than dereferencing it, so a
    // target that has since become stale (Settings closed) makes the call
    // fail harmlessly instead of faulting; `let _ =` matches that. `wParam`
    // and `lParam` carry only plain integers — no pointer crosses the
    // process boundary.
    unsafe {
        let _ = PostMessageW(target, WM_APP_RECORDED_CHORD, vk as usize, packed as isize);
    }
}

/// What the observe/record lease decides for the current keystroke, or `None`
/// when the lease does not apply (level `none`, or Settings does not hold the
/// foreground — fail closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeaseAction {
    /// Report and suppress, but let the chord reach Windows.
    Observe,
    /// Report, suppress, and swallow.
    Record,
}

/// Pure decision, given the runtime's stored lease and a supplied foreground
/// process id. Takes the foreground lookup as a lazy closure — exactly the
/// shape `DEF-1` established for the VM/RDP bypass check — so a unit test can
/// supply a fixed pid instead of asking the live desktop, which is what makes
/// this branch reachable from a test at all. `DEF-3` is this branch never
/// having been reachable, one level above where `DEF-1` fixed the same shape.
fn lease_action<G>(rt: &mut HookRuntime, foreground_pid: G) -> Option<LeaseAction>
where
    G: FnOnce(&mut HookRuntime) -> u32,
{
    // Copied out before the closure runs, so nothing here holds a borrow of
    // `rt` while `foreground_pid` needs its own `&mut` — the same shape as
    // `eval_bypass` below, one level up.
    let level = rt.capture_lease_level;
    let pid = rt.capture_lease_pid;
    if level == CAPTURE_LEASE_NONE || pid == 0 {
        return None;
    }
    // Fail closed: a lease that is armed but Settings does not currently hold
    // the foreground window does nothing, on every keystroke this check
    // reaches — never only on the keystroke that happens to matter.
    if foreground_pid(rt) != pid {
        return None;
    }
    if level >= CAPTURE_LEASE_RECORD {
        Some(LeaseAction::Record)
    } else if level >= CAPTURE_LEASE_OBSERVE {
        Some(LeaseAction::Observe)
    } else {
        None
    }
}

/// Whether the process a lease is addressed to is still alive. `OQ-17` records
/// that a pid can be recycled; this narrows the window rather than closing it.
/// Called only from the heartbeat, never from the callback.
///
/// SAFETY: `OpenProcess` takes a plain `u32` pid and returns a handle or null;
/// null is checked before use. `GetExitCodeProcess` writes through `&mut
/// code`, a live local of the exact width it expects. The handle is closed on
/// every path once queried.
fn lease_holder_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // SAFETY: `OpenProcess` takes a plain `u32` pid and returns a handle or
    // null; null is checked before use. `GetExitCodeProcess` writes through
    // `&mut code`, a live local of the exact width it expects, and the
    // handle is closed on every path once queried.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == 0 {
            return false;
        }
        let mut code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut code) != 0;
        CloseHandle(handle);
        ok && code == STILL_ACTIVE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyHandleResult {
    PassToNext,
    Swallow,
}

struct KeyHandleOutcome {
    disposition: KeyHandleResult,
    #[allow(dead_code)]
    enqueued: bool,
}

#[cfg(debug_assertions)]
fn is_win_vk(vk: u32) -> bool {
    matches!(vk, VK_LWIN | VK_RWIN)
}

fn handle_key_event(rt: &mut HookRuntime, vk: u32, key_down: bool) -> KeyHandleOutcome {
    handle_key_event_with_bypass(
        rt,
        vk,
        key_down,
        |rt| {
            crate::context::vm_bypass::evaluate_foreground(&rt.bypass_policy, &mut rt.identity)
                .is_passthrough()
        },
        |rt| rt.identity.foreground_pid(),
    )
}

fn handle_key_event_with_bypass<F, G>(
    rt: &mut HookRuntime,
    vk: u32,
    key_down: bool,
    eval_bypass: F,
    lease_foreground_pid: G,
) -> KeyHandleOutcome
where
    F: FnOnce(&mut HookRuntime) -> bool,
    G: FnOnce(&mut HookRuntime) -> u32,
{
    rt.mods.apply_vk(vk, key_down);

    // The chord is over once every modifier is up; only then may the bypass
    // latch clear. Clearing it earlier would let the tail of a passthrough
    // chord fall back into Wira Desk handling.
    if rt.bypass_latched && !rt.mods.any() {
        rt.bypass_latched = false;
    }

    // A latched chord passes through in its entirety — including releases —
    // without touching swallow state, throttle, or the ring.
    if rt.bypass_latched {
        return KeyHandleOutcome {
            disposition: KeyHandleResult::PassToNext,
            enqueued: false,
        };
    }

    if !key_down {
        if rt.swallow_release_vk != 0 && vk as u16 == rt.swallow_release_vk {
            rt.swallow_release_vk = 0;
            return KeyHandleOutcome {
                disposition: KeyHandleResult::Swallow,
                enqueued: false,
            };
        }
        return KeyHandleOutcome {
            disposition: KeyHandleResult::PassToNext,
            enqueued: false,
        };
    }

    if ModifierState::is_modifier_vk(vk) {
        return KeyHandleOutcome {
            disposition: KeyHandleResult::PassToNext,
            enqueued: false,
        };
    }

    // DEC-004's observe/record lease, checked *before* `match_shortcut` so an
    // unconfigured chord (e.g. `Win+1`) reaches it too — `DEF-3`'s reported
    // symptom is exactly a chord that is already one of the six configured
    // shortcuts never reaching this far under the old ordering. Bounded to a
    // non-modifier key-down carrying at least one modifier, which is already
    // guaranteed by this point in the function (modifier-only presses and key
    // releases both returned above).
    if rt.mods.any() {
        if let Some(action) = lease_action(rt, lease_foreground_pid) {
            report_recorded_chord(vk, rt.mods);
            // Deliberately does NOT set `bypass_latched`. That latch exists
            // to protect a *swallowed* chord's tail (its own key-up, and the
            // modifier releases) from falling back into normal handling —
            // Observe swallows nothing, and Record's swallowed key-down has
            // no matching `swallow_release_vk` to protect either, since we
            // return before that assignment runs. Latching here instead
            // blocked every *other* key pressed while the same modifiers
            // stayed held: hold Ctrl+Alt, tap Left (latches), tap Right
            // without releasing Ctrl+Alt — the latch's top-of-function guard
            // passed Right through blind, before this check or
            // `match_shortcut` ever ran, so it was neither reported nor
            // executed until every modifier came up. That is the reported
            // symptom of "sometimes not detected while testing."
            return KeyHandleOutcome {
                disposition: match action {
                    LeaseAction::Observe => KeyHandleResult::PassToNext,
                    LeaseAction::Record => KeyHandleResult::Swallow,
                },
                enqueued: false,
            };
        }
    }

    let Some(cmd) = match_shortcut(&rt.chords, rt.mods, vk as u16) else {
        return KeyHandleOutcome {
            disposition: KeyHandleResult::PassToNext,
            enqueued: false,
        };
    };

    // The shortcut matched. Before claiming it, ask whether the foreground
    // belongs to a VM or Remote Desktop guest. This runs only on a
    // matched chord — rare — and uses bounded non-blocking metadata queries
    // against reusable buffers, so the callback stays within its budget.
    if eval_bypass(rt) {
        rt.bypass_latched = true;
        // No ring publication, no Worker wake, no throttle advancement, and no
        // swallow state — the chord is not ours.
        return KeyHandleOutcome {
            disposition: KeyHandleResult::PassToNext,
            enqueued: false,
        };
    }

    let mut enqueued = false;
    let now = tick_ms();
    // Throttle and capacity rejections are counted separately so the
    // reconciliation can tell an intentional drop from a failure. Atomic
    // increments only — no allocation, no lock, no logging in the callback.
    if !throttle_allows(rt.last_throttle_ms, now) {
        #[cfg(debug_assertions)]
        crate::metrics::THROTTLED.fetch_add(1, Ordering::Relaxed);
    } else if !ring::push(cmd) {
        #[cfg(debug_assertions)]
        crate::metrics::DROPPED_FULL.fetch_add(1, Ordering::Relaxed);
    } else {
        enqueued = true;
        rt.last_throttle_ms = now;
        #[cfg(debug_assertions)]
        crate::metrics::ACCEPTED.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `PostMessageW` compares `worker_hwnd` rather than dereferencing it, so a
        // stale handle makes the call fail instead of faulting — hence `let _ =`. Both
        // `wParam` and `lParam` are zero, so no pointer and no ownership crosses the thread
        // boundary; the command itself is already in the ring, and this message is only a
        // wake-up. A lost post therefore delays a command until the next wake rather than
        // losing or leaking it.
        //
        // `PostMessageW`, never `SendMessageW`, and that choice is what keeps this call
        // legal here at all: posting returns immediately, while sending would block the
        // input-processing path until the Worker finished a full cycle — unbounded work
        // inside the hook callback, which the bounded-callback contract forbids.
        unsafe {
            let _ = PostMessageW(rt.worker_hwnd, WM_APP_COMMAND_READY, 0, 0);
        }
    }

    rt.swallow_release_vk = vk as u16;
    // NOTE: an earlier attempt injected an unassigned key here (`SendInput`)
    // so the shell would not see a lone Win press. It was removed: calling
    // `SendInput` from inside the low-level hook callback races the activation
    // the Worker is about to perform, and cycling stopped moving focus at all.
    // The Win key-up is deliberately NOT swallowed. Swallowing it leaves the
    // focused application believing Win is still held, which turns every later
    // keystroke into Win+key — the sticky-modifier bug. A Start Menu that
    // occasionally opens is the lesser fault, and the correct fix belongs
    // outside the callback.
    KeyHandleOutcome {
        disposition: KeyHandleResult::Swallow,
        enqueued,
    }
}

#[cfg(debug_assertions)]
fn handle_key_event_timed(rt: &mut HookRuntime, vk: u32, key_down: bool) -> KeyHandleOutcome {
    let start = qpc_now();
    let outcome = handle_key_event(rt, vk, key_down);
    record_key_path_duration(start);
    outcome
}

#[cfg(debug_assertions)]
fn debug_simulate_shortcut(rt: &mut HookRuntime, scenario: usize) {
    rt.mods = ModifierState::default();
    rt.last_throttle_ms = 0;
    rt.swallow_release_vk = 0;
    while ring::pop().is_some() {}

    match scenario {
        0 => {
            let mut enqueued = false;
            let mut swallow_main = false;
            let mut win_up_passed = false;
            for (vk, down) in [
                (VK_LWIN, true),
                (VK_BACKTICK, true),
                (VK_BACKTICK, false),
                (VK_LWIN, false),
            ] {
                let o = handle_key_event_timed(rt, vk, down);
                if down && vk == VK_BACKTICK && o.disposition == KeyHandleResult::Swallow {
                    swallow_main = true;
                }
                if !down && is_win_vk(vk) && o.disposition == KeyHandleResult::PassToNext {
                    win_up_passed = true;
                }
                if o.enqueued {
                    enqueued = true;
                }
            }
            crate::util::append_debug_trace(&format!(
                "SIM_SHORTCUT: scenario=primary enqueued={} swallow_main_down={} win_up_passed={}",
                u8::from(enqueued),
                u8::from(swallow_main),
                u8::from(win_up_passed)
            ));
        }
        1 => {
            let _ = handle_key_event_timed(rt, VK_LWIN, true);
            let _ = handle_key_event_timed(rt, VK_LCONTROL, true);
            let o = handle_key_event_timed(rt, VK_BACKTICK, true);
            let passed = o.disposition == KeyHandleResult::PassToNext && !o.enqueued;
            crate::util::append_debug_trace(&format!(
                "SIM_SHORTCUT: scenario=extra_mod pass={}",
                u8::from(passed)
            ));
            let _ = handle_key_event_timed(rt, VK_BACKTICK, false);
            let _ = handle_key_event_timed(rt, VK_LCONTROL, false);
            let _ = handle_key_event_timed(rt, VK_LWIN, false);
        }
        _ => {
            crate::util::append_debug_trace("SIM_SHORTCUT: scenario=unknown pass=0");
        }
    }
}

/// Collapsed modifier state derived from hook events (never `GetAsyncKeyState`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierState {
    pub win: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl ModifierState {
    fn apply_vk(&mut self, vk: u32, key_down: bool) {
        match vk {
            VK_LWIN | VK_RWIN => self.win = key_down,
            VK_LCONTROL | VK_RCONTROL => self.ctrl = key_down,
            VK_LMENU | VK_RMENU => self.alt = key_down,
            VK_LSHIFT | VK_RSHIFT => self.shift = key_down,
            _ => {}
        }
    }

    /// True while any modifier is still held — i.e. a chord is in progress.
    fn any(&self) -> bool {
        self.win || self.ctrl || self.alt || self.shift
    }

    fn is_modifier_vk(vk: u32) -> bool {
        matches!(
            vk,
            VK_LWIN
                | VK_RWIN
                | VK_LCONTROL
                | VK_RCONTROL
                | VK_LMENU
                | VK_RMENU
                | VK_LSHIFT
                | VK_RSHIFT
        )
    }
}

/// Every chord the Hook matches against, as one value.
///
/// `None` means **unbound**: the action has no chord that reaches it, and no other action
/// fires in its place. Representing that as an *absent* chord rather than as a sentinel
/// `Shortcut` is what makes "matches nothing" true by construction — there is no value an
/// unbound field could accidentally equal, so the guarantee cannot be lost to a later edit.
///
/// One struct rather than one parameter per chord, because the previous shape carried the
/// same six values through a tuple return, six positional parameters, six runtime fields,
/// and six assignment statements. Six parallel lists is five chances for a new action to be
/// added to some of them and missed in the rest, and that mistake is silent: the chord
/// simply never fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chords {
    pub primary: Option<Shortcut>,
    pub fallback: Option<Shortcut>,
    pub snap_left: Option<Shortcut>,
    pub snap_right: Option<Shortcut>,
    pub snap_maximize: Option<Shortcut>,
    pub stack: Option<Shortcut>,
}

/// One entry of the declared sequence: a chord and the command it resolves to.
pub struct ChordSlot {
    pub chord: Option<Shortcut>,
    pub command: u8,
}

impl Chords {
    /// The **declared sequence**: the one order that decides both which action a chord
    /// resolves to and, when two actions carry the same chord, which of them keeps it.
    ///
    /// It exists exactly once on purpose. The order is also the Settings pane's draw order
    /// and its keyboard focus order; three independently maintained copies of it is how the
    /// visible order and the precedence order start disagreeing with nothing detecting it.
    pub fn in_declared_order(&self) -> [ChordSlot; 6] {
        [
            ChordSlot {
                chord: self.primary,
                command: Command::Cycle.as_u8(),
            },
            ChordSlot {
                chord: self.fallback,
                command: Command::Cycle.as_u8(),
            },
            ChordSlot {
                chord: self.snap_left,
                command: Command::SnapLeft.as_u8(),
            },
            ChordSlot {
                chord: self.snap_right,
                command: Command::SnapRight.as_u8(),
            },
            ChordSlot {
                chord: self.snap_maximize,
                command: Command::SnapMaximize.as_u8(),
            },
            ChordSlot {
                chord: self.stack,
                command: Command::OverlappingStack.as_u8(),
            },
        ]
    }
}

/// Pure shortcut equality match → command byte.
///
/// Resolves against the declared sequence and returns the **first** match, which is the
/// same first-wins behaviour the previous `if / else if` chain had. Stating it as a walk
/// over the declared order rather than as a chain is what makes the precedence order and
/// the match order literally the same code instead of two things that happen to agree.
pub fn match_shortcut(chords: &Chords, mods: ModifierState, vk: u16) -> Option<u8> {
    let current = Shortcut {
        win: mods.win,
        ctrl: mods.ctrl,
        alt: mods.alt,
        shift: mods.shift,
        vk,
    };
    chords
        .in_declared_order()
        .iter()
        .find(|slot| slot.chord == Some(current))
        .map(|slot| slot.command)
}

/// Returns `true` when `now_ms` may accept a new throttled shortcut.
pub fn throttle_allows(last_accept_ms: u64, now_ms: u64) -> bool {
    last_accept_ms == 0 || now_ms.saturating_sub(last_accept_ms) >= ANTI_MACRO_THROTTLE_MS
}

/// Pure hook-check counter , testable without Win32.
pub fn next_hook_check_state(current_fail_count: u32, install_succeeded: bool) -> (u32, bool) {
    if install_succeeded {
        return (0, false);
    }
    let count = current_fail_count + 1;
    (count, count >= HOOK_CHECK_FAIL_THRESHOLD)
}

struct HookRuntime {
    worker_hwnd: HWND,
    h_mod: HINSTANCE,
    chords: Chords,
    mods: ModifierState,
    last_throttle_ms: u64,
    swallow_release_vk: u16,
    hook_handle: HHOOK,
    hook_check_fail_count: u32,
    /// Immutable VM/RDP policy, normalized off the callback path.
    bypass_policy: BypassPolicy,
    /// Reusable fixed buffers for foreground identity. Never reallocated.
    identity: HookIdentityCollector,
    /// Latched once a chord begins inside a bypass context.
    /// Without this, a focus change mid-chord could make the key-down pass
    /// through while the matching key-up got swallowed, leaving the guest
    /// session with a stuck modifier.
    bypass_latched: bool,
    /// Level of the temporary shortcut capture lease: `CAPTURE_LEASE_NONE`,
    /// `_OBSERVE`, or `_RECORD`. See [`lease_action`].
    capture_lease_level: usize,
    /// PID of the Settings process holding the lease (0 when the level is
    /// `CAPTURE_LEASE_NONE`).
    capture_lease_pid: u32,
}

/// Address of the Hook thread's [`HookRuntime`], which lives on that thread's
/// stack for as long as the message loop runs.
///
/// This is an `AtomicPtr` rather than a `static mut` for two reasons, the second
/// far more important than the first. `static mut` references became a hard
/// error in edition 2024, so the old form was on borrowed time. But the real
/// hazard is aliasing: once the address is published here, this pointer is the
/// **only** path by which the runtime may be reached. The message loop MUST NOT
/// borrow the local directly again, because a fresh `&mut` to the local
/// invalidates the pointer stored here, and every hook callback afterwards would
/// be dereferencing dangling provenance.
static RUNTIME: AtomicPtr<HookRuntime> = AtomicPtr::new(std::ptr::null_mut());

/// Configuration snapshot the Worker has handed over but the Hook thread has not
/// collected yet.
///
/// The snapshot used to travel *inside* the message, as a `Box::into_raw` pointer in
/// `lParam`, and the handler reconstructed it with `Box::from_raw`. That worked, but its
/// safety rested entirely on nobody else being able to post the message: the handler had
/// no way to tell a pointer the Worker had staged from an arbitrary integer, so a
/// `WM_APP_CONFIG_SNAPSHOT` from anywhere else would have been a free of an
/// attacker-chosen address. UIPI does block that for an elevated process, and the message
/// filter is never widened beyond `TaskbarCreated` — but that is mitigation by
/// unreachability, one refactor away from not holding.
///
/// Keeping the pointer here instead removes the class of bug rather than guarding it.
/// `WM_APP_CONFIG_SNAPSHOT` now carries zero in both `wParam` and `lParam`, exactly like
/// every other message the daemon posts, so it is a wake-up and nothing more. A spurious
/// one causes the Hook thread to collect whatever the Worker legitimately staged, or
/// nothing at all.
///
/// Ownership rule: whoever swaps a non-null pointer *out* of this slot owns it and frees
/// it. That is what keeps a superseded snapshot from leaking and from being freed twice.
static PENDING_SNAPSHOT: AtomicPtr<crate::config::HookSnapshot> =
    AtomicPtr::new(std::ptr::null_mut());

// The slot moves ownership of a `HookSnapshot` from the Worker thread to the Hook thread,
// which is only sound if the type may cross threads at all. Asserted rather than assumed,
// so adding a non-`Send` field to `HookSnapshot` becomes a compile error here instead of a
// data race somewhere else.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<crate::config::HookSnapshot>();
};

/// Worker side: stage `snapshot` for the Hook thread and return its raw pointer so the
/// caller can undo this if the wake-up cannot be posted.
///
/// An earlier snapshot still sitting in the slot is freed here. It was superseded before
/// the Hook thread ever saw it, and its wake-up message — if one is still queued — will
/// simply find whatever is current, or an empty slot.
pub fn stage_snapshot(snapshot: crate::config::HookSnapshot) -> *mut crate::config::HookSnapshot {
    let raw = Box::into_raw(Box::new(snapshot));
    let previous = PENDING_SNAPSHOT.swap(raw, Ordering::AcqRel);
    if !previous.is_null() {
        // SAFETY: `previous` came from `Box::into_raw` in an earlier call to this
        // function, and swapping it out of the slot is what transfers ownership here —
        // the Hook thread can no longer reach it, so this is the only free.
        drop(unsafe { Box::from_raw(previous) });
    }
    raw
}

/// Worker side: reclaim a staged snapshot after a failed wake-up post.
///
/// Conditional on purpose. Between staging and this call the Hook thread may already have
/// collected the snapshot, or a newer stage may have replaced it; in both cases the
/// pointer belongs to someone else and must not be freed here. The compare-exchange is
/// what distinguishes those from the case this function is for.
pub fn unstage_snapshot(raw: *mut crate::config::HookSnapshot) {
    if PENDING_SNAPSHOT
        .compare_exchange(
            raw,
            std::ptr::null_mut(),
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
    {
        // SAFETY: the exchange succeeded, so `raw` was still the slot's value and no
        // other party has taken ownership of it. It came from `Box::into_raw` in
        // `stage_snapshot`.
        drop(unsafe { Box::from_raw(raw) });
    }
}

/// Hook side: take the staged snapshot, if there is one.
fn take_staged_snapshot() -> Option<Box<crate::config::HookSnapshot>> {
    let raw = PENDING_SNAPSHOT.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if raw.is_null() {
        None
    } else {
        // SAFETY: a non-null value was placed here by `stage_snapshot` via
        // `Box::into_raw` on this exact type, and the swap leaves the slot empty, so this
        // is the only reconstruction of that pointer.
        Some(unsafe { Box::from_raw(raw) })
    }
}

/// Borrow the Hook thread's runtime, if one has been published.
///
/// # Safety
/// The caller MUST be running on the Hook thread and MUST NOT already hold a
/// borrow of the runtime. Both hold for the two callers — the `WH_KEYBOARD_LL`
/// callback and the message loop — because Windows never runs them concurrently
/// on a single thread, and neither keeps a borrow alive across a
/// message-retrieval call, which is the only point at which the callback can be
/// delivered.
unsafe fn runtime_mut() -> Option<&'static mut HookRuntime> {
    let rt = RUNTIME.load(Ordering::Acquire);
    if rt.is_null() {
        None
    } else {
        // SAFETY: a non-null value means `hook_thread_main` published the address
        // of its live stack local and has not yet retracted it, so the target is
        // allocated and initialized. The caller's contract supplies uniqueness.
        Some(&mut *rt)
    }
}

fn resolve_shortcut(configured: &str, default: &str) -> Shortcut {
    Shortcut::parse(configured)
        .or_else(|| Shortcut::parse(default))
        .unwrap_or_else(|| Shortcut::parse("win+backtick").expect("default shortcut"))
}

fn load_shortcuts(worker_hwnd: HWND) -> Chords {
    let cfg = Config::load_or_default(&config_path());
    let defaults = SwitcherConfig::default();
    let snap_defaults = shared::config::SnappingConfig::default();
    let layout_defaults = shared::config::LayoutConfig::default();

    // One row per chord, and the row is the only place a chord's config path, its
    // shipped default, and its diagnostic name appear together. The six hand-written
    // blocks this replaces each repeated that triple, and a new chord meant writing a
    // seventh block correctly rather than adding a row.
    let rows: [(&str, &str, &str); 6] = [
        (
            "switcher.shortcut",
            &cfg.switcher.shortcut,
            &defaults.shortcut,
        ),
        (
            "switcher.fallback_shortcut",
            &cfg.switcher.fallback_shortcut,
            &defaults.fallback_shortcut,
        ),
        (
            "snapping.snap_half_left",
            &cfg.snapping.snap_half_left,
            &snap_defaults.snap_half_left,
        ),
        (
            "snapping.snap_half_right",
            &cfg.snapping.snap_half_right,
            &snap_defaults.snap_half_right,
        ),
        (
            "snapping.snap_maximize",
            &cfg.snapping.snap_maximize,
            &snap_defaults.snap_maximize,
        ),
        (
            "layout.stack_shortcut",
            &cfg.layout.stack_shortcut,
            &layout_defaults.stack_shortcut,
        ),
    ];

    let mut resolved: [Option<Shortcut>; 6] = [None; 6];
    for (i, (field, configured, default)) in rows.iter().enumerate() {
        resolved[i] = Some(resolve_one(worker_hwnd, field, configured, default));
    }

    Chords {
        primary: resolved[0],
        fallback: resolved[1],
        snap_left: resolved[2],
        snap_right: resolved[3],
        snap_maximize: resolved[4],
        stack: resolved[5],
    }
}

/// Resolve one configured chord, falling back to its shipped default on either failure and
/// warning once about which. Startup must always produce a daemon, so neither an
/// unparseable string nor a chord Windows owns is fatal here — both degrade to the default
/// and say so. `config::validate` is where the same conditions are *refused*, because a
/// reload has a last-known-good configuration to keep and startup does not.
fn resolve_one(worker_hwnd: HWND, field: &str, configured: &str, default: &str) -> Shortcut {
    match Shortcut::parse(configured) {
        Some(parsed) => {
            if shared::shortcut::reservation(&parsed).is_some() {
                crate::log::warn(
                    worker_hwnd,
                    &format!(
                        "Reserved shortcut configured for {field}; falling back to default {default}"
                    ),
                );
                resolve_shortcut(default, default)
            } else {
                parsed
            }
        }
        None => {
            crate::log::warn(
                worker_hwnd,
                &format!("Invalid {field} shortcut; using default {default}"),
            );
            resolve_shortcut(default, default)
        }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: Windows delivers WH_KEYBOARD_LL on the thread that installed the
    // hook, which is the Hook thread, and no other borrow of the runtime is live
    // at this point. Taking the borrow exactly once for the whole callback is
    // what makes that true rather than merely likely — there is no second `&mut`
    // in this function to alias with.
    let Some(rt) = runtime_mut() else {
        // Nothing published yet, or already torn down. A null HHOOK is a valid
        // first argument to CallNextHookEx, so the event still propagates.
        return CallNextHookEx(0, code, wparam, lparam);
    };
    let hook = rt.hook_handle;
    if code < 0 {
        return CallNextHookEx(hook, code, wparam, lparam);
    }
    if code != HC_ACTION as i32 {
        return CallNextHookEx(hook, code, wparam, lparam);
    }
    // SAFETY: for HC_ACTION, Windows documents lparam as a pointer to a
    // KBDLLHOOKSTRUCT owned by the OS and valid for the duration of the call.
    let info = &*(lparam as *const KBDLLHOOKSTRUCT);
    let vk = info.vkCode;
    // Synthetic input is ignored, and must be: Wira Desk injects `VK_NONAME`
    // itself to suppress the Start Menu, so processing injected events would
    // let the hook consume its own injection. The measurement harness opens
    // this gate deliberately (debug builds only) because posting commands
    // straight to the Worker bypasses the hook, and a daemon that never
    // received an input event is denied foreground rights by Windows — which
    // is why every previously recorded sample came from a cycle that
    // failed to move focus.
    if (info.flags & LLKHF_INJECTED) != 0 && !accept_injected() {
        return CallNextHookEx(hook, code, wparam, lparam);
    }

    let msg = wparam as u32;
    let key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    let key_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
    if !(key_down || key_up) {
        return CallNextHookEx(hook, code, wparam, lparam);
    }

    #[cfg(debug_assertions)]
    let qpc_start = qpc_now();
    let outcome = handle_key_event(rt, vk, key_down);
    #[cfg(debug_assertions)]
    record_key_path_duration(qpc_start);

    match outcome.disposition {
        KeyHandleResult::Swallow => 1,
        KeyHandleResult::PassToNext => CallNextHookEx(hook, code, wparam, lparam),
    }
}

fn tick_ms() -> u64 {
    // SAFETY: `GetTickCount64` takes no arguments, touches no memory we own, and cannot
    // fail; it is `unsafe` only because it is an FFI declaration. It is also callable from
    // the hook callback, which is where the throttle reads it — no allocation, no lock, no
    // blocking, and no dependence on thread or apartment state.
    unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() }
}

unsafe fn install_hook(h_mod: HINSTANCE) -> HHOOK {
    SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), h_mod, 0)
}

unsafe fn refresh_hook_on_hook_thread(rt: &mut HookRuntime) {
    // Reap a lease left armed against a process that has since exited — on
    // the heartbeat, never inside the callback (`DEC-004`'s bound). This
    // narrows the `OQ-17` pid-reuse window rather than closing it: between an
    // exit and the next heartbeat tick, a recycled pid could still match.
    if rt.capture_lease_level != CAPTURE_LEASE_NONE && !lease_holder_alive(rt.capture_lease_pid) {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!(
            "HOOK_LEASE: reaped dead holder pid={}",
            rt.capture_lease_pid
        ));
        rt.capture_lease_level = CAPTURE_LEASE_NONE;
        rt.capture_lease_pid = 0;
        set_report_target(0);
    }

    let new_hook = install_hook(rt.h_mod);

    #[cfg(debug_assertions)]
    let new_hook = {
        let mut h = new_hook;
        if debug_force_hook_fail() && h != 0 {
            UnhookWindowsHookEx(h);
            h = 0;
        }
        h
    };

    let install_succeeded = new_hook != 0;
    let (fail_count, should_escalate) =
        next_hook_check_state(rt.hook_check_fail_count, install_succeeded);
    rt.hook_check_fail_count = fail_count;

    if install_succeeded {
        if rt.hook_handle != 0 {
            UnhookWindowsHookEx(rt.hook_handle);
        }
        rt.hook_handle = new_hook;
        let _ = PostMessageW(rt.worker_hwnd, WM_APP_HOOK_REFRESH_OK, 0, 0);
    } else {
        debug_log(
            "Wira Desk: hook heartbeat refresh — SetWindowsHookExW failed; keeping prior hook",
        );
    }

    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!(
        "HOOK_CHECK: install_succeeded={install_succeeded} fail_count={fail_count} escalate={should_escalate}"
    ));

    if should_escalate {
        let _ = PostMessageW(rt.worker_hwnd, WM_APP_HOOK_DEAD, 0, 0);
    }
}

unsafe fn handle_thread_message(rt: &mut HookRuntime, msg: &MSG) -> bool {
    match msg.message {
        m if m == WM_APP_HOOK_CHECK => {
            refresh_hook_on_hook_thread(rt);
            true
        }
        // Config reload. The message is a wake-up only — see `PENDING_SNAPSHOT` for why
        // the snapshot no longer travels in `lParam`. Collecting it here is what keeps the
        // Hook's shortcut and bypass state written by the Hook thread alone, so "never
        // mutated concurrently by the Worker" is structural rather than a convention
        // someone has to remember. An empty slot means the wake-up was superseded or
        // spurious, and doing nothing is the right answer.
        m if m == WM_APP_CONFIG_SNAPSHOT => {
            if let Some(snapshot) = take_staged_snapshot() {
                rt.chords = snapshot.chords;
                rt.bypass_policy = snapshot.bypass;
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace("CONFIG_SNAPSHOT: hook state replaced");
            }
            true
        }
        m if m == WM_APP_HOOK_SHUTDOWN => {
            if rt.hook_handle != 0 {
                UnhookWindowsHookEx(rt.hook_handle);
                rt.hook_handle = 0;
            }
            PostQuitMessage(0);
            true
        }
        m if m == WM_APP_HOOK_LEASE => {
            rt.capture_lease_level = msg.wParam;
            rt.capture_lease_pid = if rt.capture_lease_level != CAPTURE_LEASE_NONE {
                msg.lParam as u32
            } else {
                0
            };
            #[cfg(debug_assertions)]
            crate::util::append_debug_trace(&format!(
                "HOOK_LEASE: level={} pid={}",
                rt.capture_lease_level, rt.capture_lease_pid
            ));
            true
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_TOGGLE_HOOK_FAIL => {
            debug_toggle_hook_fail();
            true
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED => {
            debug_toggle_accept_injected();
            true
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_DUMP_HOOK_LATENCY => {
            if msg.wParam != 0 {
                reset_qpc_stats();
            } else {
                dump_qpc_stats();
            }
            true
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_SIMULATE_SHORTCUT => {
            debug_simulate_shortcut(rt, msg.wParam);
            true
        }
        _ => false,
    }
}

fn hook_thread_main(worker_hwnd: HWND, h_mod: HINSTANCE) {
    // SAFETY: this function owns the hook's entire lifetime and the publication of
    // [`RUNTIME`], so the obligations are stated once here.
    //
    // Plain calls. `MSG` is a plain C struct of integers and a `POINT`, so `zeroed` yields a
    // valid value; both `MSG` locals are passed by unique pointer and live across their
    // calls. `GetCurrentThread` is a pseudo-handle that is always valid and must not be
    // closed. The opening `PeekMessageW` is not a stray probe — it forces the OS to create
    // this thread's message queue before `WM_APP_HOOK_READY` tells the Worker the thread id,
    // so a heartbeat or snapshot posted immediately afterwards cannot be dropped for want of
    // a queue.
    //
    // The hook. `low_level_keyboard_proc` is a `fn` item, so the pointer Windows retains is
    // valid for the whole program; `WH_KEYBOARD_LL` needs no DLL, and with a zero thread id
    // the callback is delivered on *this* thread, which is the premise every borrow below
    // depends on. `UnhookWindowsHookEx` must be called from the installing thread, and both
    // call sites are on it.
    //
    // Publishing `runtime`. The address is taken with `&raw mut`, which produces a pointer
    // *without* creating a `&mut` — the distinction is the whole point. Forming a reference
    // and casting it would give the pointer provenance derived from a borrow that ends
    // immediately, and every later callback would dereference dangling provenance even
    // though the storage is still alive. Between the `store` and the null `store`, `runtime`
    // MUST NOT be borrowed directly again; the only accesses are through `rt_ptr` or
    // `runtime_mut`, which loads the same pointer. The local outlives the loop, so the
    // published address stays valid for exactly that window, and the null store precedes the
    // direct `runtime.hook_handle` read below so no callback-derived borrow can overlap it.
    //
    // Why no lock is needed for that shared pointer: the callback and this loop both run on
    // this thread and never concurrently, because Windows delivers `WH_KEYBOARD_LL` only
    // from inside a message-retrieval call. No borrow here is held across `GetMessageW`,
    // `TranslateMessage`, or `DispatchMessageW`, which are the only points where the
    // callback can be entered.
    unsafe {
        let mut peek_msg: MSG = std::mem::zeroed();
        let _ = PeekMessageW(&mut peek_msg, 0, 0, 0, PM_NOREMOVE);

        if SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL) == 0 {
            let _ = PostMessageW(worker_hwnd, WM_APP_HOOK_INIT_FAILED, 0, 0);
            return;
        }

        let chords = load_shortcuts(worker_hwnd);

        // Normalize the VM/RDP policy here — before the hook is installed and
        // therefore off the callback path entirely. It stays
        // immutable for the lifetime of this Hook configuration.
        let bypass_policy =
            BypassPolicy::from_config(&Config::load_or_default(&config_path()).vm_bypass);

        let mut hook_handle = 0isize;
        let mut retries = HOOK_RETRY_MAX;
        while retries > 0 {
            hook_handle = install_hook(h_mod);
            if hook_handle != 0 {
                break;
            }
            retries -= 1;
            if retries > 0 {
                thread::sleep(Duration::from_secs(HOOK_RETRY_DELAY_SECS));
            }
        }

        if hook_handle == 0 {
            let _ = PostMessageW(worker_hwnd, WM_APP_HOOK_INIT_FAILED, 0, 0);
            return;
        }

        let mut runtime = HookRuntime {
            worker_hwnd,
            h_mod,
            chords,
            mods: ModifierState::default(),
            last_throttle_ms: 0,
            swallow_release_vk: 0,
            hook_handle,
            hook_check_fail_count: 0,
            bypass_policy,
            identity: HookIdentityCollector::new(),
            bypass_latched: false,
            capture_lease_level: CAPTURE_LEASE_NONE,
            capture_lease_pid: 0,
        };

        // Publish the runtime by address. From here until the null store below,
        // `runtime` MUST NOT be borrowed directly again — see the note on
        // `RUNTIME`. Every access goes through `rt_ptr`, or through
        // `runtime_mut`, which loads this same pointer.
        let rt_ptr: *mut HookRuntime = &raw mut runtime;
        RUNTIME.store(rt_ptr, Ordering::Release);
        let thread_id = GetCurrentThreadId();
        let _ = PostMessageW(worker_hwnd, WM_APP_HOOK_READY, thread_id as usize, 0);
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!("HOOK_READY: tid={thread_id}"));

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let r = GetMessageW(&mut msg, 0, 0, 0);
            if r == 0 || r == -1 {
                break;
            }
            // SAFETY: reborrowed through the published pointer rather than
            // borrowed from the local, which is what keeps `rt_ptr` valid for the
            // callback. The callback cannot observe this borrow: Windows delivers
            // WH_KEYBOARD_LL only from inside a message-retrieval call, and this
            // borrow ends before the loop returns to `GetMessageW`.
            if msg.hwnd == 0 && handle_thread_message(&mut *rt_ptr, &msg) {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Retract the pointer *before* touching `runtime` directly again, so no
        // callback can be holding a borrow derived from it once the local is
        // borrowed normally below.
        RUNTIME.store(std::ptr::null_mut(), Ordering::Release);
        if runtime.hook_handle != 0 {
            UnhookWindowsHookEx(runtime.hook_handle);
        }
    }
}

/// Start the Hook Thread.
pub fn spawn(worker_hwnd: HWND, h_mod: HINSTANCE) -> (JoinHandle<()>, Arc<AtomicBool>) {
    let shutdown = Arc::new(AtomicBool::new(false));
    let handle = thread::spawn(move || hook_thread_main(worker_hwnd, h_mod));
    (handle, shutdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The six shipped chords as they stood before `DEC-008` moved the family, so the
    /// tests below keep exercising the same values this refactor must not change.
    fn shipped_chords() -> Chords {
        Chords {
            primary: Shortcut::parse("win+backtick"),
            fallback: Shortcut::parse("alt+backtick"),
            snap_left: Shortcut::parse("ctrl+win+left"),
            snap_right: Shortcut::parse("ctrl+win+right"),
            snap_maximize: Shortcut::parse("ctrl+win+enter"),
            stack: Shortcut::parse("ctrl+win+down"),
        }
    }

    #[test]
    fn exact_primary_match() {
        let mods = ModifierState {
            win: true,
            ..Default::default()
        };
        assert_eq!(
            match_shortcut(&shipped_chords(), mods, 0xC0),
            Some(Command::Cycle.as_u8())
        );
    }

    #[test]
    fn exact_snap_left_match() {
        let mods = ModifierState {
            win: true,
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(
            match_shortcut(&shipped_chords(), mods, 0x25), // VK_LEFT
            Some(Command::SnapLeft.as_u8())
        );
    }

    #[test]
    fn extra_modifier_is_non_match() {
        let mods = ModifierState {
            win: true,
            ctrl: true,
            ..Default::default()
        };
        assert!(match_shortcut(&shipped_chords(), mods, 0xC0).is_none());
    }

    #[test]
    fn an_unbound_chord_matches_nothing() {
        // `None` is not "some chord nobody presses" — it must be unreachable. This is the
        // property `DEC-009`'s unbinding rests on, asserted here rather than assumed.
        let chords = Chords {
            snap_left: None,
            ..shipped_chords()
        };
        let mods = ModifierState {
            win: true,
            ctrl: true,
            ..Default::default()
        };
        assert!(match_shortcut(&chords, mods, 0x25).is_none());
        // Every other action still reaches its own chord.
        assert_eq!(
            match_shortcut(&chords, mods, 0x27), // VK_RIGHT
            Some(Command::SnapRight.as_u8())
        );
    }

    #[test]
    fn the_declared_order_is_the_precedence_order() {
        // Two actions on one chord: the earlier slot wins, and it wins because it is
        // earlier in `in_declared_order`, not because of where an `if` happens to sit.
        let clash = Shortcut::parse("ctrl+win+down").unwrap();
        let chords = Chords {
            snap_right: Some(clash),
            stack: Some(clash),
            ..shipped_chords()
        };
        let order = chords.in_declared_order();
        let first = order
            .iter()
            .position(|s| s.chord == Some(clash))
            .expect("the clashing chord is in the sequence");
        assert_eq!(order[first].command, Command::SnapRight.as_u8());
        let mods = ModifierState {
            win: true,
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(
            match_shortcut(&chords, mods, 0x28), // VK_DOWN
            Some(Command::SnapRight.as_u8())
        );
    }

    #[test]
    fn throttle_boundary() {
        assert!(throttle_allows(0, 100));
        assert!(!throttle_allows(100, 149));
        assert!(throttle_allows(100, 150));
    }

    fn test_runtime(primary: Shortcut, fallback: Shortcut) -> HookRuntime {
        HookRuntime {
            worker_hwnd: 0,
            h_mod: 0,
            chords: Chords {
                primary: Some(primary),
                fallback: Some(fallback),
                ..shipped_chords()
            },
            mods: ModifierState::default(),
            last_throttle_ms: 0,
            swallow_release_vk: 0,
            hook_handle: 0,
            hook_check_fail_count: 0,
            bypass_policy: BypassPolicy::default(),
            identity: HookIdentityCollector::new(),
            bypass_latched: false,
            capture_lease_level: CAPTURE_LEASE_NONE,
            capture_lease_pid: 0,
        }
    }

    #[test]
    fn win_up_passes_through_after_win_shortcut() {
        // Regression guard for the sticky-modifier bug. Wira Desk used to swallow
        // the Win key-up so the shell would not open Start. The focused
        // application then saw Win pressed and never released, so every later
        // keystroke behaved as Win+key. The key-up must always reach the system.
        let primary = Shortcut::parse("win+backtick").unwrap();
        let fallback = Shortcut::parse("alt+backtick").unwrap();
        let mut rt = test_runtime(primary, fallback);

        // Explicitly supply `|_| false` for the bypass check so this unit test
        // does not depend on the live foreground window state of the desktop.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LWIN, true, |_| false, |_| 0).disposition,
            KeyHandleResult::PassToNext
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, true, |_| false, |_| 0).disposition,
            KeyHandleResult::Swallow
        );
        // The main key-up is still swallowed: the application never saw the
        // key-down, so delivering only its release would be incoherent.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, false, |_| false, |_| 0).disposition,
            KeyHandleResult::Swallow
        );
        // The modifier release, however, must always get through.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LWIN, false, |_| false, |_| 0).disposition,
            KeyHandleResult::PassToNext
        );
    }

    #[test]
    fn no_modifier_release_is_ever_swallowed() {
        // Stronger form of the same guard, across every modifier.
        let primary = Shortcut::parse("win+backtick").unwrap();
        let fallback = Shortcut::parse("alt+backtick").unwrap();
        for modifier in [VK_LWIN, VK_RWIN, VK_LCONTROL, VK_LMENU, VK_LSHIFT] {
            let mut rt = test_runtime(primary, fallback);
            let _ = handle_key_event_with_bypass(&mut rt, VK_LWIN, true, |_| false, |_| 0);
            let _ = handle_key_event_with_bypass(&mut rt, VK_BACKTICK, true, |_| false, |_| 0);
            let _ = handle_key_event_with_bypass(&mut rt, VK_BACKTICK, false, |_| false, |_| 0);
            assert_eq!(
                handle_key_event_with_bypass(&mut rt, modifier, false, |_| false, |_| 0)
                    .disposition,
                KeyHandleResult::PassToNext,
                "modifier {modifier:#x} release was swallowed - it would stick"
            );
        }
    }

    #[test]
    fn win_up_not_suppressed_for_alt_fallback() {
        let primary = Shortcut::parse("win+backtick").unwrap();
        let fallback = Shortcut::parse("alt+backtick").unwrap();
        let mut rt = test_runtime(primary, fallback);

        // Explicitly supply `|_| false` for the bypass check so this unit test
        // does not depend on the live foreground window state of the desktop.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LMENU, true, |_| false, |_| 0).disposition,
            KeyHandleResult::PassToNext
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, true, |_| false, |_| 0).disposition,
            KeyHandleResult::Swallow
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LWIN, false, |_| false, |_| 0).disposition,
            KeyHandleResult::PassToNext
        );
    }

    // ── DEC-004 capture lease: pure decision seam ─────────────────────────

    #[test]
    fn lease_none_takes_no_action() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_NONE;
        rt.capture_lease_pid = 4321;
        assert_eq!(lease_action(&mut rt, |_| 4321), None);
    }

    #[test]
    fn lease_fails_closed_when_settings_is_not_foreground() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_RECORD;
        rt.capture_lease_pid = 4321;
        assert_eq!(
            lease_action(&mut rt, |_| 999),
            None,
            "a lease armed against a different foreground process must do nothing"
        );
    }

    #[test]
    fn lease_observe_action_is_reported_without_a_record_action() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_OBSERVE;
        rt.capture_lease_pid = 4321;
        assert_eq!(lease_action(&mut rt, |_| 4321), Some(LeaseAction::Observe));
    }

    #[test]
    fn lease_record_action() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_RECORD;
        rt.capture_lease_pid = 4321;
        assert_eq!(lease_action(&mut rt, |_| 4321), Some(LeaseAction::Record));
    }

    // ── DEC-004 / DEF-3 regression: the lease must be reachable by a chord
    // that is not one of the six configured shortcuts, and must be checked
    // before `match_shortcut` — this is the guard that never armed. ────────

    const VK_1: u32 = 0x31;

    #[test]
    fn record_lease_swallows_an_unconfigured_chord_before_match_shortcut() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_RECORD;
        rt.capture_lease_pid = 777;

        let _ = handle_key_event_with_bypass(&mut rt, VK_LWIN, true, |_| false, |_| 777);
        let outcome = handle_key_event_with_bypass(&mut rt, VK_1, true, |_| false, |_| 777);
        assert_eq!(
            outcome.disposition,
            KeyHandleResult::Swallow,
            "Win+1 must be swallowed while the record lease is armed, even though it \
             matches none of the six configured shortcuts"
        );
        assert!(
            !outcome.enqueued,
            "a recorded chord must never also be enqueued as a Wira Desk command"
        );
    }

    #[test]
    fn observe_lease_passes_an_unconfigured_chord_through_without_swallowing() {
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_OBSERVE;
        rt.capture_lease_pid = 777;

        let _ = handle_key_event_with_bypass(&mut rt, VK_LWIN, true, |_| false, |_| 777);
        let outcome = handle_key_event_with_bypass(&mut rt, VK_1, true, |_| false, |_| 777);
        assert_eq!(outcome.disposition, KeyHandleResult::PassToNext);
        assert!(!outcome.enqueued);
    }

    #[test]
    fn lease_never_fires_without_a_modifier_held() {
        // Bound from DEC-004: report/suppress/swallow apply only to a
        // non-modifier key-down carrying at least one modifier. A bare `1`
        // with no modifier held passes through as if no lease existed.
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_RECORD;
        rt.capture_lease_pid = 777;

        let outcome = handle_key_event_with_bypass(&mut rt, VK_1, true, |_| false, |_| 777);
        assert_eq!(outcome.disposition, KeyHandleResult::PassToNext);
    }

    #[test]
    fn observe_lease_reports_a_second_key_pressed_without_releasing_modifiers() {
        // Regression: firing the lease for one key must not latch bypass —
        // that used to swallow-through every *other* key pressed while the
        // same modifiers stayed held, until every modifier came up. A user
        // testing shortcuts by holding Ctrl+Alt and tapping through
        // Left, Right, Enter, Down without releasing Ctrl+Alt in between
        // must see every one of those taps handled independently.
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        rt.capture_lease_level = CAPTURE_LEASE_OBSERVE;
        rt.capture_lease_pid = 777;

        let _ = handle_key_event_with_bypass(&mut rt, VK_LCONTROL, true, |_| false, |_| 777);
        let _ = handle_key_event_with_bypass(&mut rt, VK_LMENU, true, |_| false, |_| 777);

        const VK_LEFT: u32 = 0x25;
        const VK_RIGHT: u32 = 0x27;

        let first = handle_key_event_with_bypass(&mut rt, VK_LEFT, true, |_| false, |_| 777);
        assert_eq!(first.disposition, KeyHandleResult::PassToNext);
        // The precise regression guard: Observe must never latch bypass at
        // all. `disposition` alone cannot tell a genuine lease pass-through
        // apart from the old bug's stale-latch pass-through — both read
        // `PassToNext` — so this checks the state the bug actually left
        // behind instead.
        assert!(
            !rt.bypass_latched,
            "observe swallows nothing, so there is no tail to protect with a latch; \
             a set latch is what silently swallowed-through every later key"
        );
        let _ = handle_key_event_with_bypass(&mut rt, VK_LEFT, false, |_| false, |_| 777);

        // Ctrl and Alt are still held here — this is the exact scenario the
        // old latch broke.
        assert!(rt.mods.ctrl && rt.mods.alt);
        let second = handle_key_event_with_bypass(&mut rt, VK_RIGHT, true, |_| false, |_| 777);
        assert_eq!(second.disposition, KeyHandleResult::PassToNext);
        assert!(!rt.bypass_latched);
    }

    #[test]
    fn hook_lease_message_stores_level_and_pid_verbatim() {
        // DEF-3: three places used to disagree about what `lParam` carried.
        // The Hook thread's own message handler must store exactly what
        // arrived — a process id, never a converted window handle.
        let mut rt = test_runtime(
            Shortcut::parse("win+backtick").unwrap(),
            Shortcut::parse("alt+backtick").unwrap(),
        );
        let msg = MSG {
            hwnd: 0,
            message: WM_APP_HOOK_LEASE,
            wParam: CAPTURE_LEASE_RECORD,
            lParam: 4242,
            time: 0,
            pt: windows_sys::Win32::Foundation::POINT { x: 0, y: 0 },
        };
        // SAFETY: `handle_thread_message` only touches `rt` (a valid unique
        // borrow) and reads plain integer fields off `msg`; it does not
        // require the caller to be on the Hook thread for the
        // `WM_APP_HOOK_LEASE` arm exercised here, which does no Win32 I/O.
        unsafe {
            handle_thread_message(&mut rt, &msg);
        }
        assert_eq!(rt.capture_lease_level, CAPTURE_LEASE_RECORD);
        assert_eq!(rt.capture_lease_pid, 4242);
    }

    // ── Configuration staging slot ───────────────────────────────────────────────
    //
    // `PENDING_SNAPSHOT` is a process-wide `static`, so these tests must not run
    // concurrently with each other — the same mistake that once made a metrics test
    // flaky by asserting on globals other test threads were writing. Serialised here
    // rather than left to chance.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn drain_slot() {
        let _ = take_staged_snapshot();
    }

    fn snapshot(primary: &str) -> crate::config::HookSnapshot {
        crate::config::HookSnapshot {
            chords: Chords {
                primary: Shortcut::parse(primary),
                ..shipped_chords()
            },
            bypass: BypassPolicy::default(),
        }
    }

    #[test]
    fn staged_snapshot_is_collected_exactly_once() {
        let _guard = TEST_LOCK.lock().unwrap();
        drain_slot();

        stage_snapshot(snapshot("win+backtick"));
        let taken = take_staged_snapshot().expect("a staged snapshot must be collectable");
        assert_eq!(taken.chords.primary, Shortcut::parse("win+backtick"));

        // A second wake-up for the same stage must find nothing, which is what makes a
        // superseded or spurious message harmless.
        assert!(take_staged_snapshot().is_none());
    }

    #[test]
    fn staging_again_supersedes_an_uncollected_snapshot() {
        let _guard = TEST_LOCK.lock().unwrap();
        drain_slot();

        // Two saves in quick succession, with the Hook thread never scheduled between
        // them. The second must win, and the first must not linger.
        stage_snapshot(snapshot("win+backtick"));
        stage_snapshot(snapshot("ctrl+alt+tab"));

        let taken = take_staged_snapshot().expect("the newer snapshot must be present");
        assert_eq!(taken.chords.primary, Shortcut::parse("ctrl+alt+tab"));
        assert!(take_staged_snapshot().is_none(), "only one may be staged");
    }

    #[test]
    fn unstaging_after_a_failed_post_leaves_nothing_to_collect() {
        let _guard = TEST_LOCK.lock().unwrap();
        drain_slot();

        let raw = stage_snapshot(snapshot("win+backtick"));
        unstage_snapshot(raw);
        assert!(
            take_staged_snapshot().is_none(),
            "a snapshot whose wake-up never posted must not stay staged"
        );
    }

    #[test]
    fn unstaging_is_a_no_op_once_the_hook_has_collected() {
        let _guard = TEST_LOCK.lock().unwrap();
        drain_slot();

        let raw = stage_snapshot(snapshot("win+backtick"));
        drop(take_staged_snapshot().expect("staged"));

        // The Worker losing a race to the Hook thread is the case this guards: `raw` now
        // points at freed memory, and `unstage_snapshot` must only ever *compare* it. If
        // it freed on a failed compare-exchange this would be a double free.
        unstage_snapshot(raw);
        assert!(take_staged_snapshot().is_none());
    }
}
