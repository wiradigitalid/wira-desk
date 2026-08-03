//! Transitional migration shim from WinTick — M-02 (Scheduled Task) and M-03 (legacy daemon).
// LEGACY: remove in v0.3.0

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HWND};
use windows_sys::Win32::System::Threading::{
    OpenMutexW, CREATE_NO_WINDOW, SYNCHRONIZATION_SYNCHRONIZE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_CLOSE};

use shared::constants::TASK_NAME;

use crate::autostart;
use crate::error;
use crate::util::{debug_log, wide};

const LEGACY_TASK_NAME: &str = "WinTick";
const LEGACY_MUTEX: &str = "Global\\WinTickSingleInstanceMutex";
const LEGACY_WINDOW_CLASS: &str = "WinTickDaemonHiddenWindow";
const LEGACY_WINDOW_TITLE: &str = "WinTickDaemon";

const LEGACY_STOP_TIMEOUT: Duration = Duration::from_secs(3);
const LEGACY_POLL: Duration = Duration::from_millis(50);

fn run_schtasks(args: &[&str]) -> bool {
    Command::new("schtasks.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn task_exists(name: &str) -> bool {
    run_schtasks(&["/Query", "/TN", name])
}

/// M-02: create a `WiraDesk` task from the current executable, then delete the `WinTick` task.
pub fn migrate_scheduled_task() {
    if task_exists(TASK_NAME) {
        return;
    }
    if !task_exists(LEGACY_TASK_NAME) {
        return;
    }

    if !autostart::enable() {
        debug_log("Wira Desk: legacy task migration — create WiraDesk task failed");
        return;
    }

    if !run_schtasks(&["/Delete", "/TN", LEGACY_TASK_NAME, "/F"]) {
        debug_log("Wira Desk: legacy task migration — delete WinTick task failed");
    }
}

fn legacy_mutex_held() -> bool {
    let name = wide(LEGACY_MUTEX);
    // SAFETY: `name` comes from `wide`, so it is NUL-terminated, and it is a local that
    // outlives the call. `OpenMutexW` returns 0 when the object does not exist — which is the
    // answer this function wants — so no validity has to be established beforehand. A non-zero
    // return is an owned handle, closed exactly once on the only path that produces it; the
    // early return happens precisely when there is nothing to close. Note this only *probes*
    // for the mutex and never waits on it, so it cannot block: `SYNCHRONIZATION_SYNCHRONIZE`
    // is requested because `OpenMutexW` requires some access right, not because we acquire it.
    unsafe {
        let handle = OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, FALSE, name.as_ptr());
        if handle == 0 {
            return false;
        }
        CloseHandle(handle);
        true
    }
}

fn find_legacy_daemon_window() -> HWND {
    let class = wide(LEGACY_WINDOW_CLASS);
    let title = wide(LEGACY_WINDOW_TITLE);
    // SAFETY: both buffers are NUL-terminated (`wide`) locals that outlive the call, and
    // `FindWindowW` only reads them to match against the window list — it retains neither. A
    // zero return means no such window, which the caller checks.
    unsafe { FindWindowW(class.as_ptr(), title.as_ptr()) }
}

fn request_legacy_shutdown(hwnd: HWND) {
    // SAFETY: `PostMessageW` compares the handle rather than dereferencing it, so the race
    // this call inherently has — the legacy daemon may exit between `FindWindowW` and here —
    // resolves to a failed post, not a fault. Both `wParam` and `lParam` are zero, so nothing
    // is leaked when that happens, and the caller does not depend on the result: it polls the
    // legacy mutex to find out whether the process actually went away.
    unsafe {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}

/// M-03: stop a still-running WinTick daemon so two hooks are not active at once.
pub fn stop_legacy_daemon() {
    if !legacy_mutex_held() {
        return;
    }

    let hwnd = find_legacy_daemon_window();
    if hwnd != 0 {
        request_legacy_shutdown(hwnd);
        let deadline = Instant::now() + LEGACY_STOP_TIMEOUT;
        while Instant::now() < deadline {
            if !legacy_mutex_held() {
                debug_log("Wira Desk: MIGRATE: legacy daemon stopped");
                return;
            }
            thread::sleep(LEGACY_POLL);
        }
    }

    if legacy_mutex_held() {
        error::fatal(
            "A previous version (WinTick) is still running.\n\nClose WinTick from its system-tray icon, then start Wira Desk again.\n\nRunning both at once would leave two global keyboard hooks competing for the same shortcut.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_constants_are_stable() {
        assert_eq!(LEGACY_TASK_NAME, "WinTick");
        assert_eq!(LEGACY_MUTEX, "Global\\WinTickSingleInstanceMutex");
        assert_eq!(LEGACY_WINDOW_CLASS, "WinTickDaemonHiddenWindow");
    }

    #[test]
    fn migrate_task_noop_when_new_task_exists() {
        // Exercises the early-return branch without touching schtasks when
        // the current machine already has WiraDesk registered.
        if task_exists(TASK_NAME) {
            migrate_scheduled_task();
        }
    }
}
