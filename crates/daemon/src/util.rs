//! Small cross-module daemon utilities.

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MESSAGEBOX_STYLE};

/// Convert Rust `&str` to a null-terminated UTF-16 buffer for Win32 `*W` APIs.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copy `&str` into a fixed `[u16; N]` buffer (e.g. `szTip`, `szInfo`) with
/// null termination, truncating when too long. Safe for `N == 0`
/// (no-op) — without the guard, `N - 1` underflows to `usize::MAX`.
pub fn fill_wide_buf<const N: usize>(buf: &mut [u16; N], s: &str) {
    if N == 0 {
        return;
    }
    let mut i = 0;
    for u in s.encode_utf16() {
        if i >= N - 1 {
            break;
        }
        buf[i] = u;
        i += 1;
    }
    buf[i] = 0;
}

/// Emit a UTF-16 debug string via `OutputDebugStringW`. Visible only under a
/// debugger; no user-facing surface. Centralized here so `tray`, `menu`, and
/// `autostart` share the same diagnostic path. Kept separate from the Tier-2
/// logger (`log::warn`, `wiradesk.log` file): developer diagnostics
/// (`debug_log`) and user-facing warning logs (`log::warn`) are two distinct paths.
pub fn debug_log(msg: &str) {
    let w = wide(msg);
    // SAFETY: `wide` always appends a NUL, so `w` is a terminated UTF-16 string, and it is
    // a local that outlives this block — the buffer cannot be freed while the debugger
    // reads it. `OutputDebugStringW` only copies out of the buffer; it retains nothing.
    unsafe {
        OutputDebugStringW(w.as_ptr());
    }
}

/// Append-only trace for elevated runtime scripts (debug builds only).
#[cfg(debug_assertions)]
pub fn append_debug_trace(msg: &str) {
    use std::io::Write;
    let mut path = shared::log_path();
    path.set_file_name("wiradesk-debug-trace.log");
    crate::log::rotate_at_cap(&path, 1_000_000);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{msg}");
    }
    debug_log(msg);
}

/// In production release builds, debug trace is compiled out.
#[cfg(not(debug_assertions))]
#[allow(dead_code)]
pub fn append_debug_trace(_msg: &str) {}

/// Show a modal `MessageBoxW`, centralizing wide-string conversion so each
/// call site (startup error in `main`, About Check-for-Updates in `menu`) does not
/// repeat its own encoding. `hwnd` may be `0` for an ownerless box.
pub fn message_box(hwnd: HWND, text: &str, title: &str, flags: MESSAGEBOX_STYLE) -> i32 {
    let text_w = wide(text);
    let title_w = wide(title);
    // SAFETY: both buffers come from `wide`, so both are NUL-terminated, and both are
    // locals that outlive this block — which matters more than usual because `MessageBoxW`
    // is modal and reads them for as long as the dialog is on screen, not just for the
    // duration of a normal call. A zero `hwnd` is the documented request for an ownerless
    // box, so it needs no validity proof.
    unsafe { MessageBoxW(hwnd, text_w.as_ptr(), title_w.as_ptr(), flags) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_wide_buf_zero_length_is_safe() {
        let mut buf = [0u16; 0];
        fill_wide_buf(&mut buf, "test");
    }

    #[test]
    fn fill_wide_buf_truncates_and_null_terminates() {
        let mut buf = [0u16; 4];
        fill_wide_buf(&mut buf, "hello");
        assert_eq!(buf[3], 0);
        let s = String::from_utf16_lossy(&buf[..3]);
        assert_eq!(s, "hel");
    }

    #[test]
    fn debug_trace_rotates_at_cap_when_debug_assertions_active() {
        let mut path = std::env::temp_dir();
        path.push(format!("wiradesk-trace-test-{}.log", std::process::id()));
        let mut old = path.clone();
        old.set_file_name(format!(
            "wiradesk-trace-test-{}.log.old",
            std::process::id()
        ));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&old);

        std::fs::write(&path, vec![b'd'; 1_000_000]).unwrap();
        crate::log::rotate_at_cap(&path, 1_000_000);

        assert!(!path.exists());
        assert!(old.exists());
        assert_eq!(std::fs::metadata(&old).unwrap().len(), 1_000_000);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&old);
    }
}
