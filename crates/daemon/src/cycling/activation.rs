//! Bounded window activation without restore/maximize side effects.
//!
//! One attempt per target. This module deliberately performs **no**
//! responsiveness probe, restore, maximize, or retry loop, so a hung window
//! receives exactly the same treatment as a responsive one.
//!
//! It is bounded, not instantaneous. `SetForegroundWindow` is applied
//! asynchronously by Windows, so each attempt confirms the result by polling for
//! up to ~20 ms, and the fallback path attempts twice — roughly 40 ms worst
//! case. Every wait is on the OS applying a focus change, never on the target
//! application answering a message, which is what keeps hung windows harmless.

use std::ptr;

use windows_sys::Win32::Foundation::{FALSE, HWND, TRUE};
// `AttachThreadInput` is exported under `System::Threading` in windows-sys
// 0.52, not under `UI::Input::KeyboardAndMouse` where the Win32 docs group it.
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
};

use super::{ActivationOutcome, Activator, WindowId};

/// Production activator driving the real desktop.
pub struct Win32Activator;

impl Activator for Win32Activator {
    fn activate(&mut self, target: WindowId) -> ActivationOutcome {
        let hwnd = target.0 as HWND;

        // Validity, not responsiveness. `IsWindow` answers "does this handle
        // still exist" without talking to the owning thread, so a hung window
        // passes this check exactly like a healthy one.
        // SAFETY: `IsWindow` is handle-tolerant — it looks the handle up rather than
        // dereferencing it — so any `isize`, including a stale or never-valid one, is a legal
        // argument and simply answers false. That tolerance is what lets this be the
        // validity check instead of requiring one from the caller.
        if unsafe { IsWindow(hwnd) } == FALSE {
            return ActivationOutcome::InvalidTarget;
        }

        // SAFETY: `focus_attempt`'s documented contract is that it needs no proof the window
        // still exists, because every Win32 call it makes reports failure rather than
        // misbehaving on a stale handle. It must also run on a thread with a message queue
        // that may take the foreground, which holds: activation is only ever reached from
        // the Worker's command drain.
        if unsafe { focus_attempt(hwnd) } {
            ActivationOutcome::Activated
        } else {
            ActivationOutcome::Failed
        }
    }
}

/// A single bounded focus attempt.
/// `SetForegroundWindow` is refused by Windows unless the caller owns the
/// foreground or is otherwise privileged, so the documented remedy is to
/// briefly attach to the foreground thread's input queue and retry **once**.
///
/// This path *does* wait, up to roughly 40 ms in the worst case: two
/// [`try_set_foreground`] calls, each polling for at most ~20 ms. The wait is on
/// the operating system applying a focus change, never on the target application
/// responding, so a hung window is still treated exactly like a healthy one —
/// which is the property this module actually guarantees. Do not describe this
/// function as non-blocking; an earlier revision did, directly above the sleep
/// loop, which is worth avoiding twice.
///
/// # Safety
/// `hwnd` is used only with handle-tolerant Win32 calls, all of which report
/// failure rather than misbehave on a stale or invalid handle, so the caller need
/// not prove the window still exists.
unsafe fn focus_attempt(hwnd: HWND) -> bool {
    if try_set_foreground(hwnd) {
        return true;
    }

    let foreground = GetForegroundWindow();
    if foreground == 0 {
        return false;
    }

    // The thread pair matters, and getting it wrong is silent. Windows grants
    // `SetForegroundWindow` to the thread that owns the foreground input queue,
    // so **this** thread must attach to the foreground thread in order to
    // inherit that right. Attaching the foreground thread to the *target*
    // thread — the mistake this code originally made — leaves the caller with
    // no rights at all, so the fallback never worked and every second cycle
    // silently failed to move focus.
    let current_thread = GetCurrentThreadId();
    let foreground_thread = GetWindowThreadProcessId(foreground, ptr::null_mut());
    if foreground_thread == 0 || foreground_thread == current_thread {
        return false;
    }

    let attached = AttachThreadInput(current_thread, foreground_thread, TRUE) != FALSE;
    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!(
        "FOCUS_ATTEMPT: target={hwnd} fg={foreground} attached={}",
        u8::from(attached)
    ));
    if !attached {
        return false;
    }

    // NOTE: this used to call `BringWindowToTop` first. That raises the window
    // without focusing it, so when the foreground request then failed the user
    // was left staring at a raised-but-unfocused window — the "lost focus"
    // step reported during cycling. Raising is not needed: a successful
    // `SetForegroundWindow` raises the window itself.
    let activated = try_set_foreground(hwnd);
    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!(
        "FOCUS_RESULT: target={hwnd} now={} ok={}",
        GetForegroundWindow(),
        u8::from(activated)
    ));
    if activated {
        // Best-effort keyboard focus; failure does not undo activation.
        SetFocus(hwnd);
    }

    AttachThreadInput(current_thread, foreground_thread, FALSE);
    activated
}

/// Ask for the foreground, then **confirm** we got it.
///
/// Two Win32 behaviours have to be handled here, and each one caused a
/// user-visible bug when it was not:
///
/// 1. `SetForegroundWindow` returns `TRUE` even when Windows declines and
///    merely flashes the taskbar button. Trusting the return value made the
///    daemon log `activated=<hwnd>` while focus had not moved — and because
///    `Activated` ends the pass, the next candidate was never tried.
/// 2. The change is applied **asynchronously**. Reading `GetForegroundWindow`
///    immediately catches the transition, when no window owns the foreground
///    yet, so a successful activation was reported as a failure and the driver
///    moved on to another window. That is the focus flicker seen while cycling.
///
/// So the confirmation polls briefly. The wait is on the operating system
/// applying a focus change, not on the target application responding, so hung
/// windows are treated exactly like healthy ones.
unsafe fn try_set_foreground(hwnd: HWND) -> bool {
    SetForegroundWindow(hwnd);

    // Bounded: ~20 ms worst case, and it exits on the first confirmation.
    for _ in 0..10 {
        if GetForegroundWindow() == hwnd {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    GetForegroundWindow() == hwnd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_handle_is_an_invalid_target() {
        // Desktop-free: `IsWindow(0)` is false in any session.
        assert_eq!(
            Win32Activator.activate(WindowId(0)),
            ActivationOutcome::InvalidTarget
        );
    }

    #[test]
    fn obviously_bogus_handle_is_an_invalid_target() {
        assert_eq!(
            Win32Activator.activate(WindowId(-1)),
            ActivationOutcome::InvalidTarget
        );
    }

    #[test]
    fn invalid_target_never_reports_activated() {
        // Guards the failure branch: a vanished window must let the driver
        // continue rather than falsely claim success.
        for handle in [0isize, -1, 1] {
            assert_ne!(
                Win32Activator.activate(WindowId(handle)),
                ActivationOutcome::Activated,
                "bogus handle {handle} reported as activated"
            );
        }
    }
}
