//! Auto-start via Windows Task Scheduler (`schtasks.exe`).
//! The task is created with trigger `ONLOGON`, `/RL HIGHEST`, and `/RU <username>`
//! — it runs as the **active user** (not `SYSTEM`) so `%APPDATA%` stays aligned
//! between daemon and settings, while still elevated without a UAC prompt at boot.
//! The action (`/TR`) uses an **absolute path** to the executable and does **not**
//! set a working directory, mitigating DLL hijacking (`schtasks` CLI has no
//! "Start in" flag; the real mitigation is an absolute path with no working dir).
//! The authoritative source for the Auto-Start menu checkmark is `schtasks /Query`
//! (`is_registered`), **not** `config.auto_start` — avoid two sources of truth.
//! Pure `std` implementation (`std::process::Command` + `creation_flags`), no
//! FFI: `CREATE_NO_WINDOW` prevents console window flicker because the daemon
//! uses `#![windows_subsystem = "windows"]`.

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

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
