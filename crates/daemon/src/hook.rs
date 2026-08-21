//! Dedicated Hook Thread: `WH_KEYBOARD_LL`, callback, refresh, lifecycle.
//! Threading: the hook callback runs only on this thread's message loop.

#[cfg(debug_assertions)]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use shared::config_path;
use shared::constants::{
    ANTI_MACRO_THROTTLE_MS, HOOK_CHECK_FAIL_THRESHOLD, HOOK_RETRY_DELAY_SECS, HOOK_RETRY_MAX,
    WM_APP_COMMAND_READY, WM_APP_CONFIG_SNAPSHOT, WM_APP_HOOK_CHECK, WM_APP_HOOK_DEAD,
    WM_APP_HOOK_INIT_FAILED, WM_APP_HOOK_READY, WM_APP_HOOK_REFRESH_OK, WM_APP_HOOK_SHUTDOWN,
};
#[cfg(debug_assertions)]
use shared::constants::{
    WM_APP_DEBUG_DUMP_HOOK_LATENCY, WM_APP_DEBUG_SIMULATE_SHORTCUT, WM_APP_DEBUG_TOGGLE_HOOK_FAIL,
};
use shared::{Command, Config, Shortcut, SwitcherConfig};

use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
};
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
    handle_key_event_with_bypass(rt, vk, key_down, |rt| {
        crate::context::vm_bypass::evaluate_foreground(&rt.bypass_policy, &mut rt.identity)
            .is_passthrough()
    })
}

fn handle_key_event_with_bypass<F>(
    rt: &mut HookRuntime,
    vk: u32,
    key_down: bool,
    eval_bypass: F,
) -> KeyHandleOutcome
where
    F: FnOnce(&mut HookRuntime) -> bool,
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

    let Some(cmd) = match_shortcut(rt.primary, rt.fallback, rt.mods, vk as u16) else {
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

/// Pure shortcut equality match → command byte.
pub fn match_shortcut(
    primary: Shortcut,
    fallback: Shortcut,
    mods: ModifierState,
    vk: u16,
) -> Option<u8> {
    let current = Shortcut {
        win: mods.win,
        ctrl: mods.ctrl,
        alt: mods.alt,
        shift: mods.shift,
        vk,
    };
    if current == primary || current == fallback {
        Some(Command::Cycle.as_u8())
    } else {
        None
    }
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
    primary: Shortcut,
    fallback: Shortcut,
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

fn load_shortcuts(worker_hwnd: HWND) -> (Shortcut, Shortcut) {
    let cfg = Config::load_or_default(&config_path());
    let defaults = SwitcherConfig::default();
    let primary = match Shortcut::parse(&cfg.switcher.shortcut) {
        Some(s) => s,
        None => {
            crate::log::warn(
                worker_hwnd,
                "Invalid primary switcher shortcut; using default win+backtick",
            );
            resolve_shortcut(&defaults.shortcut, "win+backtick")
        }
    };
    let fallback = match Shortcut::parse(&cfg.switcher.fallback_shortcut) {
        Some(s) => s,
        None => {
            crate::log::warn(
                worker_hwnd,
                "Invalid fallback switcher shortcut; using default alt+backtick",
            );
            resolve_shortcut(&defaults.fallback_shortcut, "alt+backtick")
        }
    };
    (primary, fallback)
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
                rt.primary = snapshot.primary;
                rt.fallback = snapshot.fallback;
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

        let (primary, fallback) = load_shortcuts(worker_hwnd);

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
            primary,
            fallback,
            mods: ModifierState::default(),
            last_throttle_ms: 0,
            swallow_release_vk: 0,
            hook_handle,
            hook_check_fail_count: 0,
            bypass_policy,
            identity: HookIdentityCollector::new(),
            bypass_latched: false,
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

    #[test]
    fn exact_primary_match() {
        let primary = Shortcut::parse("win+backtick").unwrap();
        let fallback = Shortcut::parse("alt+backtick").unwrap();
        let mods = ModifierState {
            win: true,
            ..Default::default()
        };
        assert_eq!(
            match_shortcut(primary, fallback, mods, 0xC0),
            Some(Command::Cycle.as_u8())
        );
    }

    #[test]
    fn extra_modifier_is_non_match() {
        let primary = Shortcut::parse("win+backtick").unwrap();
        let fallback = Shortcut::parse("alt+backtick").unwrap();
        let mods = ModifierState {
            win: true,
            ctrl: true,
            ..Default::default()
        };
        assert!(match_shortcut(primary, fallback, mods, 0xC0).is_none());
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
            primary,
            fallback,
            mods: ModifierState::default(),
            last_throttle_ms: 0,
            swallow_release_vk: 0,
            hook_handle: 0,
            hook_check_fail_count: 0,
            bypass_policy: BypassPolicy::default(),
            identity: HookIdentityCollector::new(),
            bypass_latched: false,
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
            handle_key_event_with_bypass(&mut rt, VK_LWIN, true, |_| false).disposition,
            KeyHandleResult::PassToNext
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, true, |_| false).disposition,
            KeyHandleResult::Swallow
        );
        // The main key-up is still swallowed: the application never saw the
        // key-down, so delivering only its release would be incoherent.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, false, |_| false).disposition,
            KeyHandleResult::Swallow
        );
        // The modifier release, however, must always get through.
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LWIN, false, |_| false).disposition,
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
            let _ = handle_key_event(&mut rt, VK_LWIN, true);
            let _ = handle_key_event(&mut rt, VK_BACKTICK, true);
            let _ = handle_key_event(&mut rt, VK_BACKTICK, false);
            assert_eq!(
                handle_key_event(&mut rt, modifier, false).disposition,
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
            handle_key_event_with_bypass(&mut rt, VK_LMENU, true, |_| false).disposition,
            KeyHandleResult::PassToNext
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_BACKTICK, true, |_| false).disposition,
            KeyHandleResult::Swallow
        );
        assert_eq!(
            handle_key_event_with_bypass(&mut rt, VK_LWIN, false, |_| false).disposition,
            KeyHandleResult::PassToNext
        );
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
            primary: Shortcut::parse(primary).unwrap(),
            fallback: Shortcut::parse("alt+backtick").unwrap(),
            bypass: BypassPolicy::default(),
        }
    }

    #[test]
    fn staged_snapshot_is_collected_exactly_once() {
        let _guard = TEST_LOCK.lock().unwrap();
        drain_slot();

        stage_snapshot(snapshot("win+backtick"));
        let taken = take_staged_snapshot().expect("a staged snapshot must be collectable");
        assert_eq!(taken.primary, Shortcut::parse("win+backtick").unwrap());

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
        assert_eq!(taken.primary, Shortcut::parse("ctrl+alt+tab").unwrap());
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
