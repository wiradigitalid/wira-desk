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
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{msg}");
    }
    debug_log(msg);
}

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
