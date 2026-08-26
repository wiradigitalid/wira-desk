//! Auto-start via Windows Task Scheduler (`schtasks.exe`).
//! The task is created with trigger `ONLOGON`, `/RL HIGHEST`, and `/RU <username>`
//! — it runs as the **active user** (not `SYSTEM`) so `%APPDATA%` stays aligned
//! between daemon and settings, while still elevated without a UAC prompt at boot.
//! The action (`/TR`) uses an **absolute path** to the executable and does **not**
//! set a working directory, mitigating DLL hijacking (`schtasks` CLI has no
//! "Start in" flag; the real mitigation is an absolute path with no working dir).
//! The authoritative source for the Auto-Start menu checkmark is `schtasks /Query`
//! (`is_registered`), **not** `config.auto_start` — avoid two sources of truth.
//! Two things follow from that stored absolute path, and both are handled here
//! rather than left to the reader of `SECURITY.md`: the path can go **stale** when
//! the executable moves (`refresh_registered_path`), and the path can point
//! somewhere a non-administrator is able to overwrite, which turns an unprompted
//! elevated logon task into a privilege-escalation route
//! (`warn_if_location_replaceable`).
//! Pure `std` implementation (`std::process::Command` + `creation_flags`), no
//! FFI: `CREATE_NO_WINDOW` prevents console window flicker because the daemon
//! uses `#![windows_subsystem = "windows"]`.

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

use shared::constants::TASK_NAME;

use crate::util::debug_log;

/// Arguments for `schtasks /Create` to register the auto-start task.
/// `/TR` is wrapped in **explicit** quotes (`"<path>"`) so Task Scheduler
/// stores the action as one complete command; without that, a path with spaces (e.g.
/// `%ProgramFiles%\WiraDesk\wiradesk.exe`) is split by the Task Scheduler parser into
/// executable + arguments. No working directory is set (DLL hijacking mitigation).
fn create_args(exe_path: &str, username: &str) -> Vec<String> {
    vec![
        "/Create".into(),
        "/TN".into(),
        TASK_NAME.into(),
        "/TR".into(),
        format!("\"{exe_path}\""),
        "/SC".into(),
        "ONLOGON".into(),
        "/RL".into(),
        "HIGHEST".into(),
        "/RU".into(),
        username.into(),
        "/F".into(),
    ]
}

/// Arguments for `schtasks /Query` to check whether the task exists.
fn query_args() -> Vec<String> {
    vec!["/Query".into(), "/TN".into(), TASK_NAME.into()]
}

/// Arguments for `schtasks /Delete` to remove the task.
fn delete_args() -> Vec<String> {
    vec![
        "/Delete".into(),
        "/TN".into(),
        TASK_NAME.into(),
        "/F".into(),
    ]
}

/// Run `schtasks.exe` with the given arguments without a console window and
/// without inheriting stdio (output is irrelevant; only the exit code is used).
/// Returns `None` if the process fails to spawn.
fn run_schtasks(args: &[String]) -> Option<std::process::ExitStatus> {
    Command::new("schtasks.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
}

/// Absolute path of the running daemon executable.
fn current_exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Active username from `%USERNAME%` (the user running the elevated daemon).
fn current_username() -> String {
    std::env::var("USERNAME").unwrap_or_default()
}

/// `true` when the auto-start task is registered (`schtasks /Query` exit code 0).
/// Authoritative source for the Auto-Start menu checkmark.
pub fn is_registered() -> bool {
    run_schtasks(&query_args())
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Register the auto-start task. Returns `true` when `schtasks /Create` succeeds.
pub fn enable() -> bool {
    let exe = match current_exe_path() {
        Some(p) => p,
        None => {
            debug_log("Wira Desk: autostart::enable — current_exe() failed");
            return false;
        }
    };
    let user = current_username();
    let ok = run_schtasks(&create_args(&exe, &user))
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        debug_log("Wira Desk: autostart::enable — schtasks /Create failed");
    }
    ok
}

/// Re-point the registered task at the executable that is running now.
///
/// `is_registered` answers *does the task exist*, and deliberately keeps answering
/// only that — it is the source of truth for the menu checkmark, and a checkmark
/// that went blank because a path had drifted would report the wrong thing. So the
/// path is not validated there; it is enforced here.
///
/// **Why it rewrites unconditionally instead of comparing first.** Reading the
/// stored path back means parsing `schtasks` output, and both of its formats are
/// traps: the `/FO LIST` labels are localised, so `Task To Run:` is a different
/// string on a Windows installed in Indonesian or German, and `/XML` emits UTF-16
/// through a pipe. A guard whose failure mode is "silently stops matching on a
/// translated Windows" is a guard that is not there. `create_args` already carries
/// `/F`, so re-creating is idempotent, costs one `schtasks` call at startup, and
/// cannot fail open.
///
/// The running executable wins because it is the one that just proved it is
/// elevated. An attacker able to run a *different* copy elevated already holds
/// administrator rights, which `SECURITY.md` puts out of scope; the case this
/// closes is the honest one, where the file moved and the task did not follow.
///
/// Does nothing when no task is registered — enabling auto-start is the user's
/// decision, and this must never make it for them.
pub fn refresh_registered_path() -> bool {
    if !is_registered() {
        return false;
    }
    if !enable() {
        debug_log("Wira Desk: autostart::refresh_registered_path — schtasks /Create failed");
        return false;
    }
    true
}

/// Raise a Tier-2 warning when auto-start would launch this executable from a
/// place a non-administrator could overwrite.
///
/// The task runs with `/RL HIGHEST` at every logon and shows no UAC prompt, so the
/// file's permissions are the only thing standing between a non-administrator and
/// unprompted elevated execution. `SECURITY.md` asks the reader to install
/// somewhere only administrators can write; this is that request, checked.
///
/// **Warn, never refuse.** Registration still succeeds and the toggle still turns
/// on. Refusing would be the stronger guard and it is a deliberate non-goal here:
/// running from `target\release` is exactly what building this project looks like,
/// and a guard that blocks the maintainer's own workflow gets switched off rather
/// than heeded. The judgement stays with the owner; the product's job is to make
/// sure they are not making it unknowingly.
///
/// Silent when the location is fine, and silent when `acl` could not read the DACL —
/// an unreadable permission is logged for a developer, not escalated to the user,
/// because a warning that might be about nothing teaches people to ignore warnings.
pub fn warn_if_location_replaceable(hwnd: HWND) {
    if !is_registered() {
        return;
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            debug_log("Wira Desk: autostart::warn_if_location_replaceable — current_exe() failed");
            return;
        }
    };
    match crate::acl::replaceable_by_non_admin(&exe) {
        crate::acl::Verdict::NonAdminWritable => crate::log::warn(
            hwnd,
            &format!(
                "Auto-Start is registered from a location a non-administrator can overwrite: {}. \
                 Windows runs that file elevated at every logon with no prompt, so anyone able to \
                 replace it gains administrator access. Move Wira Desk to a folder only \
                 administrators can write, such as %ProgramFiles%, or turn Auto-Start off.",
                exe.display()
            ),
        ),
        crate::acl::Verdict::Unknown => {
            debug_log("Wira Desk: autostart::warn_if_location_replaceable — DACL unreadable")
        }
        crate::acl::Verdict::AdminOnly => {}
    }
}

/// Remove the auto-start task. Returns `true` when `schtasks /Delete` succeeds.
pub fn disable() -> bool {
    let ok = run_schtasks(&delete_args())
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        debug_log("Wira Desk: autostart::disable — schtasks /Delete failed");
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_args_wraps_exe_path_in_quotes() {
        // Paths with spaces (%ProgramFiles%) MUST be quoted so Task Scheduler
        // does not treat the remainder after the space as a separate argument.
        let args = create_args(r"C:\Program Files\WiraDesk\wiradesk.exe", "alice");
        let tr = args.iter().position(|a| a == "/TR").expect("/TR present");
        assert_eq!(args[tr + 1], r#""C:\Program Files\WiraDesk\wiradesk.exe""#);
    }

    #[test]
    fn create_args_carries_logon_elevation_flags() {
        let args = create_args("x.exe", "bob");
        let pair = |k: &str| args.windows(2).find(|w| w[0] == k).map(|w| w[1].clone());
        assert_eq!(pair("/SC").as_deref(), Some("ONLOGON"));
        assert_eq!(pair("/RL").as_deref(), Some("HIGHEST"));
        assert_eq!(pair("/RU").as_deref(), Some("bob"));
        assert_eq!(pair("/TN").as_deref(), Some(TASK_NAME));
        assert!(args.contains(&"/Create".to_string()));
        assert!(args.contains(&"/F".to_string()));
    }

    #[test]
    fn query_and_delete_target_the_pinned_task_name() {
        assert_eq!(query_args(), vec!["/Query", "/TN", TASK_NAME]);
        assert_eq!(delete_args(), vec!["/Delete", "/TN", TASK_NAME, "/F"]);
    }
}
