//! Tier 2 (Warning) — append-only logging to `wiradesk.log` plus a red-dot
//! visual trigger (no pop-up).
//! `warn` has no call site in this module yet (this task only required the
//! module and function to exist; real Tier-2 triggers follow in a later story
//! with non-fatal conditions to report) — `#[allow(dead_code)]` is intentional,
//! not a sign of permanently dead code.
#![allow(dead_code)]

use std::io::Write;
use std::path::Path;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;

use shared::constants::WM_APP_LOG_WARNING;

use crate::util::debug_log;

/// Size ceiling for `wiradesk.log`. Tier 2 events are rare by design (a
/// config reload rejection, a reserved shortcut warning — not a per-keystroke
/// or per-heartbeat write), so a byte cap is simpler to implement and test
/// correctly than pruning by parsed line age, for the same slow growth this
/// log actually sees. No backup file is kept: once the cap is reached, the
/// file is truncated and restarted rather than rotated to a `.old` — old
/// Tier 2 warnings are not worth preserving past the point they'd force this
/// file to keep growing forever.
const LOG_MAX_BYTES: u64 = 1_000_000; // 1 MB

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
    write_line_to(&shared::log_path(), msg);
}

/// `write_line`'s actual logic, over an explicit path — the seam that makes
/// the size-cap decision testable without touching the real
/// `%APPDATA%\WiraDesk\wiradesk.log`.
fn write_line_to(path: &Path, msg: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format!("[{}] {msg}\n", timestamp());
    // A file that does not exist yet is not "at the cap" — `unwrap_or(false)`
    // covers exactly that, and any other metadata failure the same way:
    // append is always the safe default when the size cannot be determined.
    let at_cap = std::fs::metadata(path)
        .map(|m| m.len() >= LOG_MAX_BYTES)
        .unwrap_or(false);
    match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(!at_cap)
        .truncate(at_cap)
        .open(path)
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

    fn temp_log_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "wiradesk-log-test-{}-{name}.log",
            std::process::id()
        ));
        p
    }

    #[test]
    fn a_line_under_the_cap_is_appended() {
        let path = temp_log_path("append");
        let _ = std::fs::remove_file(&path);

        write_line_to(&path, "first");
        write_line_to(&path, "second");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.contains("first"),
            "earlier lines must survive: {contents:?}"
        );
        assert!(contents.contains("second"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_already_at_the_cap_is_truncated_not_grown_forever() {
        let path = temp_log_path("cap");
        let _ = std::fs::remove_file(&path);

        // Pre-fill past the cap directly, without going through `write_line_to` —
        // the point under test is what happens on the *next* write once the
        // file is already there, not how it got there.
        std::fs::write(&path, vec![b'x'; LOG_MAX_BYTES as usize]).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() >= LOG_MAX_BYTES);

        write_line_to(&path, "after the cap");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            !contents.contains('x'),
            "the oversized content must be gone, not merely appended past: {} bytes",
            contents.len()
        );
        assert!(contents.contains("after the cap"));
        assert!(
            (contents.len() as u64) < LOG_MAX_BYTES,
            "a reset file must not still be near the cap"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn no_backup_file_is_ever_created_on_truncation() {
        let path = temp_log_path("nobackup");
        let backup = path.with_extension("log.old");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);

        std::fs::write(&path, vec![b'x'; LOG_MAX_BYTES as usize]).unwrap();
        write_line_to(&path, "triggers truncation");

        assert!(
            !backup.exists(),
            "no .old (or any other backup) file must be written — old logs are discarded, not kept"
        );
        let _ = std::fs::remove_file(&path);
    }
}
