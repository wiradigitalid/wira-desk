//! System Tray icon, hidden host window, and main daemon message loop.
//! The loop listens for the `TaskbarCreated` broadcast and re-registers the icon.
//! The window-owning thread holds tray state exclusively; other threads
//! (hook, health) only `PostMessage`. Icon state machine: Normal, Warning,
//! Critical.
//! NOTE: the window MUST be top-level (not message-only) because broadcast
//! `TaskbarCreated` is delivered only to top-level windows. This window is never
//! shown (`WS_VISIBLE` is not set; `WS_EX_TOOLWINDOW` prevents Alt-Tab even if
//! some code path accidentally shows it).

use core::ffi::c_void;
use std::mem::{size_of, zeroed};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_ERROR,
    NIIF_LARGE_ICON, NIIF_USER, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION, NOTIFYICONDATAW,
    NOTIFYICON_VERSION_4,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ChangeWindowMessageFilterEx, CreateWindowExW, DefWindowProcW, DestroyIcon, DispatchMessageW,
    FindWindowW, GetMessageW, GetWindowLongPtrW, PostQuitMessage, PostThreadMessageW,
    RegisterClassW, RegisterWindowMessageW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW,
    GWLP_USERDATA, HICON, MSG, MSGFLT_ALLOW, WM_CONTEXTMENU, WM_CREATE, WM_DESTROY, WM_RBUTTONUP,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};

use shared::constants::{
    CAPTURE_LEASE_NONE, DAEMON_WINDOW_CLASS, DAEMON_WINDOW_TITLE, HOOK_RETRY_MAX,
    SETTINGS_HOOK_WINDOW_CLASS, SETTINGS_HOOK_WINDOW_TITLE, WM_APP_CAPTURE_LEASE,
    WM_APP_COMMAND_READY, WM_APP_HOOK_DEAD, WM_APP_HOOK_INIT_FAILED, WM_APP_HOOK_LEASE,
    WM_APP_HOOK_READY, WM_APP_HOOK_REFRESH_OK, WM_APP_HOOK_SHUTDOWN, WM_APP_LOG_WARNING,
    WM_APP_RELOAD_CONFIG,
};

// Debug verification seams exist only in debug builds; gating the import keeps
// the release build free of unused-import noise.
#[cfg(debug_assertions)]
use shared::constants::{
    WM_APP_DEBUG_CYCLE_BURST, WM_APP_DEBUG_DUMP_CYCLE_METRICS, WM_APP_DEBUG_DUMP_HOOK_LATENCY,
    WM_APP_DEBUG_HOOK_CHECK, WM_APP_DEBUG_RESET_CYCLE_METRICS, WM_APP_DEBUG_RUN_COMMAND,
    WM_APP_DEBUG_SIMULATE_SHORTCUT, WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED,
    WM_APP_DEBUG_TOGGLE_HOOK_FAIL, WM_APP_DEBUG_TRIGGER_WARN, WM_APP_HOOK_CHECK,
};

use crate::hook;
use crate::icon;
use crate::menu;
use crate::util::{debug_log, fill_wide_buf, wide};
use crate::worker;

/// Unique tray icon id owned by the daemon.
const TRAY_UID: u32 = 1;

/// Tray callback message (click/hover). Chosen as `WM_APP + 10` to avoid colliding
/// with `WM_APP_RELOAD_CONFIG`/`WM_APP_COMMAND_READY`/etc.
pub const WM_TRAYICON: u32 = shared::constants::WM_APP + 10;

// ── Debug-only seam to simulate a "dead hook" ───────────────────
// The entire block is compiled out in release builds (`cfg(debug_assertions)`)
// so there is zero trace in production binaries. Purpose: enable Tier-3 escalation
// and recovery paths deterministically during runtime verification, without needing
// real AV/GPO blocking `SetWindowsHookExW`. Driven from outside the process
// via `PostMessageW` to the hidden window, located with `FindWindowW` by class name;
// `verify-hook-runtime.ps1` is the harness that does it. UIPI is what keeps this from
// being a hole in release: the message filter is opened only for `TaskbarCreated`
// (see `run_message_loop`), so a non-elevated process cannot reach any `WM_APP`
// message even if a debug build is running.

/// Trace state transitions to a file (`wiradesk-debug-trace.log`, beside
/// `wiradesk.log`) so automated runtime verification can read them —
/// `util::debug_log` (`OutputDebugStringW`) requires a debugger. Debug-only.
#[cfg(debug_assertions)]
fn debug_trace(msg: &str) {
    crate::util::append_debug_trace(msg);
}

/// Visual tray icon state (Normal / Warning / Critical).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Normal,
    Warning,
    Critical,
}

/// State owned by the message loop (one instance, one thread). The pointer is stored in
/// the window's `GWLP_USERDATA` so `WndProc` can access it.
struct TrayData {
    hwnd: HWND,
    icon_normal: HICON,
    icon_warning: HICON,
    icon_critical: HICON,
    state: TrayState,
    /// Dynamic message id from `RegisterWindowMessageW("TaskbarCreated")`.
    /// Zero only when registration fails — that case is guarded in `wndproc`
    /// so it does not match `WM_NULL`.
    taskbar_created: u32,
    /// Hook Thread id (set on `WM_APP_HOOK_READY`).
    hook_thread_id: u32,
    hook_join: Option<std::thread::JoinHandle<()>>,
    health_shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    hook_dead_toast_sent: bool,
    /// Tier-2 (Warning) latch separate from `state`: `state`
    /// holds the DISPLAYED icon (precedence Critical > Warning > Normal),
    /// while this flag records that a warning is still "active" so that
    /// leaving Critical restores Warning (not Normal) instead of
    /// silently clearing the Tier-2 signal.
    warning_latched: bool,
}

impl TrayData {
    fn current_icon(&self) -> HICON {
        match self.state {
            TrayState::Normal => self.icon_normal,
            TrayState::Warning => self.icon_warning,
            TrayState::Critical => self.icon_critical,
        }
    }

    /// Idempotent teardown: safe to call from `WM_DESTROY` or the
    /// `GetMessageW == -1` error path. Zeroes handles after release so a second
    /// call is a no-op.
    unsafe fn cleanup(&mut self) {
        if self.hwnd != 0 {
            delete_icon(self);
        }
        if self.icon_normal != 0 {
            DestroyIcon(self.icon_normal);
            self.icon_normal = 0;
        }
        if self.icon_warning != 0 {
            DestroyIcon(self.icon_warning);
            self.icon_warning = 0;
        }
        if self.icon_critical != 0 {
            DestroyIcon(self.icon_critical);
            self.icon_critical = 0;
        }
    }

    unsafe fn shutdown_hook_thread(&mut self) {
        self.health_shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if self.hook_thread_id != 0 {
            let _ = PostThreadMessageW(self.hook_thread_id, WM_APP_HOOK_SHUTDOWN, 0, 0);
        }
        if let Some(join) = self.hook_join.take() {
            let _ = join.join();
        }
        self.hook_thread_id = 0;
    }
}

/// Build `NOTIFYICONDATAW` from the current state.
fn notify_data(data: &TrayData) -> NOTIFYICONDATAW {
    // SAFETY: `NOTIFYICONDATAW` is a plain C struct — integers, fixed `[u16; N]` arrays,
    // and a union of two `u32`s. In windows-sys `HWND`/`HICON` are raw integer types rather
    // than `NonNull`, so zero is a valid inhabitant of every field and `zeroed` cannot
    // construct an invalid value. Zeroing is the required starting point rather than a
    // shortcut: the shell reads only the fields named by `uFlags`, and any byte left
    // uninitialised would be interpreted as meaningful for a flag set later.
    let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = data.hwnd;
    nid.uID = TRAY_UID;
    // `NIF_SHOWTIP` is required under `NOTIFYICON_VERSION_4` — without it Windows
    // suppresses the standard tooltip even when `szTip` is filled.
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = data.current_icon();
    fill_wide_buf(&mut nid.szTip, "Wira Desk");
    nid
}

/// Add the tray icon (`NIM_ADD`) and set version 4. Called at start and on
/// `TaskbarCreated`. Returns `false` when `NIM_ADD` fails — the caller logs
/// and continues; formal Tier-2/log handling follows later.
fn add_icon(data: &TrayData) -> bool {
    // SAFETY: `nid` is a live local whose `cbSize` matches the struct actually passed, which
    // is how the shell decides which fields exist; `Shell_NotifyIconW` copies out of it and
    // retains no pointer, so the local may die at the end of this block. `nid.hIcon` is the
    // one field whose validity must outlast the call, because the shell keeps displaying
    // that icon: it belongs to `TrayData` and is destroyed only by `TrayData::cleanup`,
    // which issues `NIM_DELETE` before any `DestroyIcon`. Writing `Anonymous.uVersion` is
    // sound for the same reason zeroing is — both union arms are `u32`.
    unsafe {
        let mut nid = notify_data(data);
        if Shell_NotifyIconW(NIM_ADD, &nid) == 0 {
            debug_log("Wira Desk: Shell_NotifyIconW(NIM_ADD) failed");
            return false;
        }
        nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        if Shell_NotifyIconW(NIM_SETVERSION, &nid) == 0 {
            debug_log(
                "Wira Desk: Shell_NotifyIconW(NIM_SETVERSION) failed — v4 click semantics inactive",
            );
        }
        true
    }
}

/// Update the icon (`NIM_MODIFY`) — used by the state machine.
fn modify_icon(data: &TrayData) {
    // SAFETY: as in `add_icon` — `nid` is a live local with a matching `cbSize`, nothing is
    // retained past the call, and the `hIcon` handed over stays valid because `TrayData`
    // owns it until `cleanup` removes the icon first and destroys it second.
    unsafe {
        let nid = notify_data(data);
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

/// Remove the tray icon (`NIM_DELETE`) — prevents a ghost icon on exit.
fn delete_icon(data: &TrayData) {
    // SAFETY: `NIM_DELETE` is identified purely by the `hWnd` and `uID` pair, which
    // `notify_data` copies from the same `TrayData` that registered the icon; every other
    // field is ignored, so no handle validity is required here. `TrayData::cleanup` calls
    // this only while `hwnd != 0`, and a stale `hWnd` would make the call fail rather than
    // fault, since the shell compares the handle instead of dereferencing it.
    unsafe {
        let nid = notify_data(data);
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

/// Change icon state then re-render. Called from `WndProc` on Tier 2/3 messages.
fn set_state(data: &mut TrayData, state: TrayState) {
    if data.state != state {
        data.state = state;
        modify_icon(data);
    }
}

/// Show a balloon/toast (`NIF_INFO`) — reuse `notify_data` as the
/// base instead of building a fresh `NOTIFYICONDATAW`
/// from scratch; `notify_data` itself is not modified and remains used as-is
/// by `add_icon`/`modify_icon`/`delete_icon`. `icon_flag` is one of the
/// `NIIF_*` constants so the shell renders the icon that matches what the
/// message actually says — a Tier 3 hook-dead toast passes `NIIF_ERROR` for
/// the standard red error glyph. `hBalloonIcon` is always set to Wira Desk's
/// own tray icon, so a caller that passes `NIIF_USER | NIIF_LARGE_ICON`
/// gets that instead of Windows' generic stock "information" icon — the one
/// every other unbranded balloon on the system also uses, and the reason an
/// informational toast otherwise looks like a system dialog rather than
/// something from this product.
fn show_toast(data: &TrayData, title: &str, msg: &str, icon_flag: u32) {
    // SAFETY: same contract as `modify_icon`. The two extra fields are fixed-size arrays
    // inside `nid`, and `fill_wide_buf` is bounded by their `N` and always NUL-terminates,
    // so neither write can run past the struct nor hand the shell an unterminated string.
    // `hBalloonIcon` is set to `data.icon_normal`, a handle owned by `TrayData` for the
    // process lifetime — Shell_NotifyIconW borrows it for the toast and does not take
    // ownership, so nothing here transfers or frees it.
    unsafe {
        let mut nid = notify_data(data);
        nid.uFlags |= NIF_INFO;
        fill_wide_buf(&mut nid.szInfoTitle, title);
        fill_wide_buf(&mut nid.szInfo, msg);
        nid.dwInfoFlags = icon_flag;
        nid.hBalloonIcon = data.icon_normal;
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

/// Watch-out #3 (reset per-episode) + severity precedence:
/// icon shown after LEAVING `Critical` when hook refresh SUCCEEDS.
fn state_after_recovery(warning_latched: bool) -> TrayState {
    if warning_latched {
        TrayState::Warning
    } else {
        TrayState::Normal
    }
}

/// `WNDPROC` wrapper. A panic crossing an FFI `extern "system"` boundary
/// is UB on debug profiles (`panic="unwind"`); wrap with
/// `catch_unwind` then fall back to `DefWindowProcW`.
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        // SAFETY: `wndproc_impl` requires being called on the thread that owns the window,
        // with the arguments Windows supplied verbatim. Both hold by construction: the
        // system invokes a `WNDPROC` only on the owning thread, and this wrapper forwards
        // its parameters untouched. `AssertUnwindSafe` is the honest part of this claim —
        // a panic partway through a `&mut TrayData` mutation could leave a field updated
        // and a dependent one not. That is tolerable here because every field is a plain
        // handle, id, or flag with no cross-field invariant that a torn write could break,
        // and the recovery path only logs and defers to `DefWindowProcW`.
        wndproc_impl(hwnd, msg, wparam, lparam)
    }));
    match result {
        Ok(r) => r,
        Err(_) => {
            debug_log("Wira Desk: wndproc panicked — falling back to DefWindowProcW");
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

unsafe fn wndproc_impl(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Store the TrayData pointer during window creation.
    if msg == WM_CREATE {
        let cs = lparam as *const CREATESTRUCTW;
        let data_ptr = (*cs).lpCreateParams as isize;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, data_ptr);
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayData;
    if data_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let data = &mut *data_ptr;

    // Auto-recovery when Explorer restarts: dynamic message id, checked at runtime.
    // Guard `taskbar_created != 0` so a failed `RegisterWindowMessageW`
    // (returns 0) does not match `WM_NULL` and trigger a spurious re-add.
    if data.taskbar_created != 0 && msg == data.taskbar_created {
        if !add_icon(data) {
            debug_log("Wira Desk: TaskbarCreated recovery failed to re-add tray icon");
        }
        return 0;
    }

    match msg {
        // Tier 2 (Warning): red dot, no pop-up. Triggered by `log::warn` via
        // PostMessage (no production call site in this module yet; see
        // Watch-out #4). Separate warning latch and respect precedence: Critical
        // (Tier 3) must NOT be downgraded to Warning.
        m if m == WM_APP_LOG_WARNING => {
            data.warning_latched = true;
            if data.state != TrayState::Critical {
                set_state(data, TrayState::Warning);
            }
            0
        }
        m if m == WM_APP_COMMAND_READY => {
            worker::drain_commands();
            0
        }
        m if m == WM_APP_HOOK_READY => {
            data.hook_thread_id = wparam as u32;
            crate::health::spawn(
                data.hook_thread_id,
                std::sync::Arc::clone(&data.health_shutdown),
            );
            if !add_icon(data) {
                debug_log(
                    "Wira Desk: initial NIM_ADD failed — tray icon not visible; will retry on TaskbarCreated",
                );
            }
            // Every start, not only the first: this is the one signal that
            // Wira Desk is actually alive and its hook is installed, without
            // opening Settings or checking Task Manager to find out. Placed
            // after `add_icon` so the toast has an icon to anchor to rather
            // than racing a tray icon that is not there yet.
            show_toast(
                data,
                "Wira Desk",
                "Wira Desk is now running and listening for shortcuts.",
                NIIF_USER | NIIF_LARGE_ICON,
            );
            // First run. Launched here rather than in `main` so the
            // tray icon already exists — otherwise a user who closes the
            // tutorial would be left with no visible sign Wira Desk is running.
            if !shared::config_path().exists() {
                debug_log("Wira Desk: no configuration — launching onboarding");
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace("FIRST_RUN: onboarding=1");
                crate::menu::launch_onboarding();
            }
            0
        }
        m if m == WM_APP_HOOK_INIT_FAILED => {
            // The attempt count comes from the constant the retry loop actually uses, so
            // the message cannot start lying if the loop is retuned.
            crate::error::fatal(&format!(
                "Wira Desk could not install the global keyboard hook after {HOOK_RETRY_MAX} attempts.\n\nAnother program may already be holding it, or a security policy may not permit it.\n\nThe application will now close."
            ));
        }
        m if m == WM_APP_HOOK_REFRESH_OK => {
            if data.state == TrayState::Critical {
                let restored = state_after_recovery(data.warning_latched);
                set_state(data, restored);
                #[cfg(debug_assertions)]
                debug_trace(&format!(
                    "RECOVERY: refresh OK → Critical→{restored:?}, toast guard reset"
                ));
            }
            data.hook_dead_toast_sent = false;
            0
        }
        m if m == WM_APP_HOOK_DEAD => {
            set_state(data, TrayState::Critical);
            if !data.hook_dead_toast_sent {
                show_toast(
                    data,
                    "Wira Desk",
                    "The keyboard hook stopped responding and could not be recovered automatically.\n\nRestart Wira Desk to bring window switching back.",
                    NIIF_ERROR,
                );
                data.hook_dead_toast_sent = true;
                #[cfg(debug_assertions)]
                debug_trace("TIER3: state→Critical, TOAST sent");
            } else {
                #[cfg(debug_assertions)]
                debug_trace("TIER3: state→Critical, toast suppressed (guard already set)");
            }
            0
        }
        // Settings writes config.toml to completion, then
        // posts this. Until now there was no arm here at all: the message fell
        // through to DefWindowProcW while Settings reported "saved and applied",
        // and a changed shortcut only took effect after a daemon restart.
        m if m == WM_APP_RELOAD_CONFIG => {
            let outcome = crate::config::handle_reload_message(hwnd, data.hook_thread_id);
            #[cfg(debug_assertions)]
            debug_trace(&format!("RELOAD_CONFIG: {outcome:?}"));
            let _ = outcome;
            0
        }
        // Settings requests a capture lease (or disarms it). Forwarded to the
        // Hook Thread unchanged: `wParam` is the lease level, `lParam` is
        // Settings' own process id — never a window handle. `DEF-3` was this
        // message being read as an HWND on this side while Settings sent a
        // PID; the fix is to stop converting it, not to convert it correctly.
        m if m == WM_APP_CAPTURE_LEASE => {
            if data.hook_thread_id != 0 {
                let level = wparam;
                if level != CAPTURE_LEASE_NONE {
                    // Resolve Settings' hidden receiver window once, here,
                    // off the Hook thread's callback path — never per
                    // keystroke — so the Hook thread can post
                    // `WM_APP_RECORDED_CHORD` back with a cheap
                    // `PostMessageW` against an already-known handle.
                    let class = wide(SETTINGS_HOOK_WINDOW_CLASS);
                    let title = wide(SETTINGS_HOOK_WINDOW_TITLE);
                    // SAFETY: `class` and `title` are NUL-terminated wide
                    // string locals that outlive this call. `FindWindowW`
                    // returns 0 if Settings' receiver window does not exist
                    // (not running, or not yet created), handled as "no
                    // report target" rather than an error.
                    let report_hwnd = unsafe { FindWindowW(class.as_ptr(), title.as_ptr()) };
                    crate::hook::set_report_target(report_hwnd);
                } else {
                    crate::hook::set_report_target(0);
                }
                let _ = PostThreadMessageW(data.hook_thread_id, WM_APP_HOOK_LEASE, wparam, lparam);
            }
            0
        }
        // Debug-only (Task 0): posted externally during runtime verification.
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_TOGGLE_HOOK_FAIL => {
            if data.hook_thread_id != 0 {
                let _ =
                    PostThreadMessageW(data.hook_thread_id, WM_APP_DEBUG_TOGGLE_HOOK_FAIL, 0, 0);
            }
            0
        }
        // Measurement-only: lets the harness drive the real shortcut through the
        // hook via `SendInput` instead of posting past it (see the constant's
        // docs for why every earlier sample missed the activation path).
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED => {
            if data.hook_thread_id != 0 {
                let _ = PostThreadMessageW(
                    data.hook_thread_id,
                    WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED,
                    0,
                    0,
                );
            }
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_TRIGGER_WARN => {
            crate::log::warn(hwnd, "debug: simulated Tier-2 warning");
            debug_trace("DEBUG_TRIGGER_WARN: log::warn called → expect red dot");
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_HOOK_CHECK => {
            if data.hook_thread_id != 0 {
                let _ = PostThreadMessageW(data.hook_thread_id, WM_APP_HOOK_CHECK, 0, 0);
            }
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_DUMP_HOOK_LATENCY => {
            if data.hook_thread_id != 0 {
                let _ = PostThreadMessageW(
                    data.hook_thread_id,
                    WM_APP_DEBUG_DUMP_HOOK_LATENCY,
                    wparam,
                    0,
                );
            }
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_SIMULATE_SHORTCUT => {
            if data.hook_thread_id != 0 {
                let _ = PostThreadMessageW(
                    data.hook_thread_id,
                    WM_APP_DEBUG_SIMULATE_SHORTCUT,
                    wparam,
                    0,
                );
            }
            0
        }
        // Cycle metrics live on the Worker (this thread), so handled
        // directly — not forwarded to the Hook Thread like HOOK_LATENCY.
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_DUMP_CYCLE_METRICS => {
            crate::metrics::dump();
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_RESET_CYCLE_METRICS => {
            crate::metrics::reset();
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_RUN_COMMAND => {
            // One command, real foreground. This is the seam that exercises the
            // success path: candidate accepted, focus moved, window placed.
            let raw = wparam as u8;
            if crate::ring::push(raw) {
                worker::drain_commands();
                crate::util::append_debug_trace(&format!("RUN_COMMAND: cmd={raw} dispatched=1"));
            } else {
                crate::util::append_debug_trace(&format!("RUN_COMMAND: cmd={raw} push_failed=1"));
            }
            0
        }
        #[cfg(debug_assertions)]
        m if m == WM_APP_DEBUG_CYCLE_BURST => {
            // Each iteration publishes one command and drains it immediately,
            // so the ring never backs up and no command is discarded. This
            // exercises the Worker path measures; it does not claim to
            // exercise the Hook→Worker transport.
            let iterations = wparam as u64;
            for _ in 0..iterations {
                if !crate::ring::push(shared::Command::Cycle.as_u8()) {
                    crate::util::append_debug_trace("CYCLE_BURST: push_failed=1");
                    break;
                }
                worker::drain_commands();
            }
            crate::util::append_debug_trace(&format!("CYCLE_BURST: requested={iterations}"));
            0
        }
        // Right-click or keyboard menu button on the tray icon → context menu.
        // `NOTIFYICON_VERSION_4` callback: event = LOWORD(lParam); screen anchor
        // X = LOWORD(wParam), Y = HIWORD(wParam). `windows-sys` lacks
        // GET_X_LPARAM → extract bits manually; cast via i16 so negative coordinates
        // (secondary monitor left/above primary) sign-extend correctly.
        m if m == WM_TRAYICON => {
            let event = (lparam as u32) & 0xFFFF;
            if event == WM_CONTEXTMENU || event == WM_RBUTTONUP {
                let x = ((wparam & 0xFFFF) as u16) as i16 as i32;
                let y = (((wparam >> 16) & 0xFFFF) as u16) as i16 as i32;
                menu::show(hwnd, x, y);
            }
            0
        }
        WM_DESTROY => {
            data.shutdown_hook_thread();
            data.cleanup();
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Register the window class, create the hidden window, install the tray icon, and run
/// the message loop until `WM_QUIT`. Returns the exit code.
pub fn run_message_loop() -> i32 {
    // SAFETY: this block owns the window's whole lifetime, so the obligations are stated
    // once here rather than per call.
    //
    // Handles and strings. `GetModuleHandleW(null)` returns the current image's handle,
    // which is always valid and must not be freed. `class_name` and `title` are locals that
    // outlive every call taking them; the `wide("TaskbarCreated")` temporary lives to the
    // end of its own statement, which covers the `RegisterWindowMessageW` call it feeds.
    // Windows copies the class name into its atom table during `RegisterClassW`, so
    // `class_name` does not need to outlive the window itself. `zeroed` is valid for
    // `WNDCLASSW` and `MSG` because both are plain C structs whose fields are integers or
    // nullable pointers — including `lpfnWndProc`, an `Option<unsafe extern fn>` for which
    // all-zero is exactly `None`, and which is overwritten with `Some(wndproc)` regardless.
    //
    // `TrayData` ownership. The state is heap-allocated and leaked with `Box::into_raw`, so
    // there is exactly one owning pointer and it does not live on this frame — a stack local
    // would dangle the moment `run_message_loop` returned while the window still referenced
    // it. `GWLP_USERDATA` receives a *copy* of that pointer, not ownership, so `wndproc` may
    // dereference but never free. The single owner is reclaimed on exactly one of two
    // mutually exclusive paths: at the `hwnd == 0` failure below, or by the `Box::from_raw`
    // after the loop. `cleanup` being idempotent is what makes this safe when `WM_DESTROY`
    // has already released the Win32 handles — the later drop then frees heap memory only.
    //
    // Aliasing. `(*data_ptr)` is dereferenced here while `wndproc` also forms
    // `&mut *data_ptr`, so the two must never overlap. They cannot: `wndproc` runs only
    // when this thread dispatches, i.e. inside `CreateWindowExW` (for `WM_NCCREATE`/
    // `WM_CREATE`, before the first dereference below), or inside `GetMessageW` and
    // `DispatchMessageW`. Every dereference on this frame happens between those calls and
    // holds its reference no longer than the statement, so no `&mut` is ever live across a
    // point where `wndproc` could be entered.
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = wide(DAEMON_WINDOW_CLASS);

        let mut wc: WNDCLASSW = zeroed();
        wc.lpfnWndProc = Some(wndproc);
        wc.hInstance = hinstance;
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassW(&wc);

        let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
        if taskbar_created == 0 {
            debug_log(
                "Wira Desk: RegisterWindowMessageW(TaskbarCreated) returned 0 — auto-recovery inactive",
            );
        }

        let health_shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let data = Box::new(TrayData {
            hwnd: 0,
            icon_normal: icon::base(),
            icon_warning: icon::with_warning(),
            icon_critical: icon::with_critical(),
            state: TrayState::Normal,
            taskbar_created,
            hook_thread_id: 0,
            hook_join: None,
            health_shutdown,
            hook_dead_toast_sent: false,
            warning_latched: false,
        });
        let data_ptr = Box::into_raw(data);

        let title = wide(DAEMON_WINDOW_TITLE);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW, // Prevent Alt-Tab if a future code path shows the window.
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            0,
            0,
            hinstance,
            data_ptr as *const c_void,
        );

        if hwnd == 0 {
            // Window creation failed: reclaim box, clean up icons, and unhook for symmetric cleanup.
            let mut d = Box::from_raw(data_ptr);
            d.cleanup();
            return 1;
        }

        (*data_ptr).hwnd = hwnd;

        let (join_handle, health_shutdown_from_hook) = hook::spawn(hwnd, hinstance);
        (*data_ptr).hook_join = Some(join_handle);
        (*data_ptr).health_shutdown = health_shutdown_from_hook;

        // Hardening: elevated daemon, medium-integrity Explorer.
        // UIPI can block the `TaskbarCreated` broadcast. Modern Windows generally
        // auto-allows shell messages, but this explicit filter is defensive for
        // tightened UAC configurations.
        if taskbar_created != 0
            && ChangeWindowMessageFilterEx(
                hwnd,
                taskbar_created,
                MSGFLT_ALLOW,
                std::ptr::null_mut(),
            ) == 0
        {
            debug_log(
                "Wira Desk: ChangeWindowMessageFilterEx(TaskbarCreated) failed — recovery may be blocked on hardened systems",
            );
        }

        // Message loop. Distinguish `0` (WM_QUIT, normal cleanup via WM_DESTROY)
        // from `-1` (GetMessageW error) so cleanup still runs when the pump breaks.
        let mut msg: MSG = zeroed();
        let exit_code: i32;
        loop {
            let r = GetMessageW(&mut msg, 0, 0, 0);
            if r == 0 {
                exit_code = msg.wParam as i32;
                break;
            }
            if r == -1 {
                debug_log("Wira Desk: GetMessageW returned -1 — forcing cleanup and exit");
                (*data_ptr).shutdown_hook_thread();
                (*data_ptr).cleanup();
                exit_code = 1;
                break;
            }
            // SAFETY (`-1` path, stated here because it is the one asymmetry): unlike the
            // `WM_QUIT` path this breaks out without the window having been destroyed, so
            // after the `Box::from_raw` below its `GWLP_USERDATA` still holds the freed
            // pointer. Nothing can dereference it: this function returns immediately, and
            // `main` does nothing but `ExitProcess`, so the window is torn down by process
            // exit without another message ever being dispatched to it. Destroying the
            // window here instead would re-enter `wndproc` for `WM_DESTROY` on a path
            // already reached because the pump itself failed, which is the worse trade.
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Reclaim heap state. `cleanup` is already idempotent — WM_DESTROY /
        // the -1 path release Win32 handles; drop only frees the heap Box.
        drop(Box::from_raw(data_ptr));
        exit_code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook;

    #[test]
    fn next_hook_check_state_resets_on_success() {
        assert_eq!(hook::next_hook_check_state(2, true), (0, false));
    }

    #[test]
    fn next_hook_check_state_escalates_after_threshold() {
        assert_eq!(hook::next_hook_check_state(0, false), (1, false));
        assert_eq!(hook::next_hook_check_state(1, false), (2, false));
        assert_eq!(hook::next_hook_check_state(2, false), (3, true));
    }

    #[test]
    fn next_hook_check_state_keeps_escalating_while_still_failing() {
        assert_eq!(hook::next_hook_check_state(3, false), (4, true));
    }

    #[test]
    fn state_after_recovery_restores_latched_warning() {
        // Watch-out #3 + precedence: leaving Critical → Warning when warning
        // is still latched, otherwise Normal (do not drop the Tier-2 signal).
        assert_eq!(state_after_recovery(true), TrayState::Warning);
        assert_eq!(state_after_recovery(false), TrayState::Normal);
    }
}
