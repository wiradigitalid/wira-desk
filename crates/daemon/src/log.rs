//! Tier 2 (Warning) — append-only logging to `wiradesk.log` plus a red-dot
//! visual trigger (no pop-up).
//! `warn` has no call site in this module yet (this task only required the
//! module and function to exist; real Tier-2 triggers follow in a later story
//! with non-fatal conditions to report) — `#[allow(dead_code)]` is intentional,
//! not a sign of permanently dead code.
#![allow(dead_code)]

use std::io::Write;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;

use shared::constants::WM_APP_LOG_WARNING;

use crate::util::debug_log;

/// Write one timestamped log line to `shared::log_path`, then notify
/// `wndproc_impl` to set `Warning` state via `PostMessageW` — only the
/// thread that owns `TrayData` may change tray state, not `warn` directly.
pub fn warn(hwnd: HWND, msg: &str) {
    write_line(msg);
    // SAFETY: `PostMessageW` inspects `hwnd` without dereferencing it — a stale or invalid
    // handle makes the call fail and return zero rather than fault, which is why no
    // validity proof is needed here and why the return value can be ignored. Both `wParam`
    // and `lParam` are zero, so this message carries no pointer and transfers no
    // ownership: a post that never arrives loses a red-dot notification and nothing else.
    // That is the distinction from `WM_APP_CONFIG_SNAPSHOT`, which does carry a leaked
    // `Box` and therefore must reclaim it when the post fails.
    unsafe {
        PostMessageW(hwnd, WM_APP_LOG_WARNING, 0, 0);
    }
}

/// Open-write-close per line (not a persistent file handle) so it is safe across
/// processes — `menu::view_logs` opens the same file from this process too.
fn write_line(msg: &str) {
    let path = shared::log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format!("[{}] {msg}\n", timestamp());
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(mut f) => {
            if f.write_all(line.as_bytes()).is_err() {
                debug_log("Wira Desk: log::warn — failed to write log line");
            }
        }
        Err(_) => debug_log("Wira Desk: log::warn — failed to open wiradesk.log"),
    }
}

/// Local timestamp `YYYY-MM-DD HH:MM:SS` via `GetLocalTime` — no `chrono`/`time`
/// dependency, consistent with this repo's windows-sys-only convention.
fn timestamp() -> String {
    // SAFETY: the inferred type is `SYSTEMTIME`, which is eight `u16` fields and nothing
    // else — no references, no `NonZero`, no enum discriminants — so the all-zero bit
    // pattern is a valid inhabitant and `zeroed` cannot produce an invalid value here.
    let mut st = unsafe { std::mem::zeroed() };
    // SAFETY: `&mut st` is a unique pointer to a live, already-initialised local, and
    // `GetLocalTime` overwrites every field it owns without reading the prior contents or
    // retaining the pointer past the call.
    unsafe {
        GetLocalTime(&mut st);
    }
    format_timestamp(&st)
}

fn format_timestamp(st: &windows_sys::Win32::Foundation::SYSTEMTIME) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Foundation::SYSTEMTIME;

    #[test]
    fn format_timestamp_pads_single_digit_fields() {
        let st = SYSTEMTIME {
            wYear: 2026,
            wMonth: 7,
            wDayOfWeek: 0,
            wDay: 4,
            wHour: 9,
            wMinute: 5,
            wSecond: 3,
            wMilliseconds: 0,
        };
        assert_eq!(format_timestamp(&st), "2026-07-04 09:05:03");
    }
}
