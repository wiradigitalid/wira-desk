//! Auto-start via Windows Task Scheduler (`schtasks.exe`).
//!
//! # Two registration paths, and why the shorter one is not enough
//! Registration prefers `/XML` and falls back to flags. Three settings this task needs
//! are wrong by default and `schtasks.exe` exposes no flag for any of them — the daemon
//! does not start on battery, is *terminated* when a charger is unplugged, and is killed
//! after 72 hours of running. [`task_xml`] carries the evidence for each. The fallback
//! keeps auto-start working if the XML path is unavailable, with those defaults intact,
//! because that is what shipped before and it is a better floor than nothing.
//! `refresh_registered_path` runs `enable` on every start, so a task written by an older
//! version is corrected without migration code.
//!
//! # The rest holds for both paths
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

/// Run `schtasks.exe` with the given arguments without a console window.
/// Returns `None` if the process fails to spawn.
///
/// **The output is captured and logged on failure, and that is the point.** This
/// previously discarded both streams, on the reasoning that only the exit code
/// mattered. The cost showed up the first time auto-start misbehaved on a real
/// machine: the log said `schtasks /Create failed` and nothing else, so the reason had
/// to be reconstructed by running `schtasks /Query` by hand. A failure that destroys
/// its own explanation is the expensive kind.
fn run_schtasks(args: &[String]) -> Option<std::process::ExitStatus> {
    let out = Command::new("schtasks.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .output()
        .ok()?;

    if !out.status.success() {
        // Both streams, because `schtasks` reports errors on stdout about as often as on
        // stderr. Collapsed to one line and truncated by characters rather than bytes, so
        // a localised message cannot split a UTF-8 sequence and panic the logger.
        let report = |label: &str, bytes: &[u8]| {
            let text = String::from_utf8_lossy(bytes)
                .replace(['\r', '\n'], " ")
                .trim()
                .to_owned();
            if !text.is_empty() {
                let clipped: String = text.chars().take(300).collect();
                debug_log(&format!("Wira Desk: schtasks {label}: {clipped}"));
            }
        };
        report("stderr", &out.stderr);
        report("stdout", &out.stdout);
    }

    Some(out.status)
}

/// Escape the five characters XML reserves, so an install path cannot break the
/// document or inject elements into it.
fn xml_escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// The task definition as Task Scheduler XML.
///
/// **Why XML at all, when `/Create` with flags is shorter.** Three of this task's
/// settings are wrong by default and `schtasks.exe` has no flag for any of them —
/// verified against `schtasks /Create /?`, which offers nothing for power management or
/// execution limits. All three were found by reading `schtasks /Query /V` output from a
/// real install, and two of them were then reproduced deliberately:
///
/// * `DisallowStartIfOnBatteries` defaults **true**, so on a laptop running on battery
///   the daemon does not start at all. Confirmed: plugged in, it starts; on battery, it
///   does not.
/// * `StopIfGoingOnBatteries` defaults **true**, so unplugging the charger *terminates*
///   a running daemon. A tray utility disappearing when a cable is pulled.
/// * `ExecutionTimeLimit` is absent from the XML Windows emits, and absent means the
///   72-hour default — a kill switch on day three for a process meant to run forever.
///
/// The element order mirrors what Windows itself emitted for a task created the old
/// way, taken from `schtasks /Query /XML`, rather than from a reading of the schema
/// sequence: the two disagree about where `MultipleInstancesPolicy` belongs, and output
/// Windows produces is output Windows accepts.
///
/// `<Command>` is deliberately **not** quoted. The old CLI form needed `"..."` because
/// Task Scheduler would otherwise split a spaced path into executable plus arguments;
/// in XML the element boundary is the delimiter, so quotes would become part of the
/// path and only work by being stripped again.
fn task_xml(exe_path: &str, user: &str) -> String {
    let exe = xml_escape(exe_path);
    let who = xml_escape(user);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>{who}</Author>
    <Description>Starts Wira Desk when {who} signs in.</Description>
  </RegistrationInfo>
  <Principals>
    <Principal id="Author">
      <UserId>{who}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
  </Triggers>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
    </Exec>
  </Actions>
</Task>
"#
    )
}

/// Fully-qualified account for the XML principal, `DOMAIN\user` where a domain is
/// known. Windows normalises it to a SID on storage; either form is accepted on input.
fn qualified_username() -> String {
    let user = current_username();
    match std::env::var("USERDOMAIN") {
        Ok(domain) if !domain.is_empty() && !user.is_empty() => format!("{domain}\\{user}"),
        _ => user,
    }
}

/// Where the XML may be staged, or `None` when nowhere safe is available.
///
/// **This is a privilege boundary, not a scratch path.** `schtasks` reads the file back
/// as an elevated process, and whatever it reads becomes a task that runs elevated at
/// every logon with no prompt. If the file lives anywhere a non-administrator can
/// write, another process running as the same user can replace its contents between our
/// write and that read, and register a task of its own choosing. Deleting the file
/// afterwards does not help: the window is before the delete, not after it.
///
/// `%TEMP%` and `%APPDATA%` both fail this test — they sit in the user profile at
/// normal user permissions, which `PRIVACY.md` already says out loud. The install
/// directory passes, and rather than assume so, `acl` is asked. Anything short of a
/// clear `AdminOnly` verdict returns `None`, which sends the caller down the CLI path:
/// the battery defaults stay wrong, and that is strictly better than a privilege
/// escalation. An unreadable DACL is treated as unsafe for the same reason.
fn xml_staging_dir() -> Option<std::path::PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    match crate::acl::replaceable_by_non_admin(&dir) {
        crate::acl::Verdict::AdminOnly => Some(dir),
        crate::acl::Verdict::NonAdminWritable => {
            debug_log(
                "Wira Desk: autostart::xml_staging_dir — install directory is non-admin \
                 writable; refusing to stage task XML there",
            );
            None
        }
        crate::acl::Verdict::Unknown => {
            debug_log(
                "Wira Desk: autostart::xml_staging_dir — install directory DACL unreadable; \
                 treating as unsafe",
            );
            None
        }
    }
}

/// Register the task from XML. `false` on any failure, so the caller can fall back to
/// the flag-based path rather than leaving auto-start broken.
fn register_via_xml(exe: &str) -> bool {
    let Some(dir) = xml_staging_dir() else {
        return false;
    };
    let path = dir.join("autostart-task.xml");

    // UTF-16LE with a BOM: Task Scheduler's own output declares `encoding="UTF-16"`, and
    // handing it UTF-8 is a documented way to get an unhelpful parse error.
    let xml = task_xml(exe, &qualified_username());
    let mut bytes = vec![0xFF, 0xFE];
    for unit in xml.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    if let Err(e) = std::fs::write(&path, &bytes) {
        debug_log(&format!(
            "Wira Desk: autostart::register_via_xml — writing {} failed: {e}",
            path.display()
        ));
        return false;
    }

    let args = vec![
        "/Create".to_string(),
        "/TN".to_string(),
        TASK_NAME.to_string(),
        "/XML".to_string(),
        path.to_string_lossy().into_owned(),
        "/F".to_string(),
    ];
    let ok = run_schtasks(&args).map(|s| s.success()).unwrap_or(false);

    // Removed either way. It carries no secret, but a stale task definition sitting in
    // the install directory invites someone to edit it and wonder why nothing changed.
    if let Err(e) = std::fs::remove_file(&path) {
        debug_log(&format!(
            "Wira Desk: autostart::register_via_xml — could not remove {}: {e}",
            path.display()
        ));
    }

    if !ok {
        debug_log("Wira Desk: autostart::register_via_xml — schtasks /Create /XML failed");
    }
    ok
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

/// Register the auto-start task. Returns `true` when registration succeeds.
///
/// XML first, flags second, and the fallback is deliberate rather than defensive
/// boilerplate. The XML path is the only one that can correct the three settings
/// `schtasks` gets wrong by default (see [`task_xml`]), but it depends on a staging
/// directory that passes an ACL check and on a schema this code cannot validate
/// locally. If either gives way, auto-start still gets registered — with the battery
/// defaults intact, which is the behaviour that shipped before and is worth keeping as
/// a floor rather than replacing with nothing.
///
/// `refresh_registered_path` calls this on every daemon start, so a task created by an
/// older version is rewritten with the corrected settings without any migration code.
pub fn enable() -> bool {
    let exe = match current_exe_path() {
        Some(p) => p,
        None => {
            debug_log("Wira Desk: autostart::enable — current_exe() failed");
            return false;
        }
    };

    if register_via_xml(&exe) {
        return true;
    }
    debug_log("Wira Desk: autostart::enable — XML registration unavailable, using flags");

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

    /// The three settings this whole XML path exists for. Each was wrong by default,
    /// each is unreachable through a `schtasks` flag, and two were reproduced on a real
    /// laptop before being fixed — so each is asserted by value rather than trusted to
    /// survive an edit of the template.
    #[test]
    fn task_xml_corrects_the_three_schtasks_defaults() {
        let xml = task_xml(r"C:\Program Files\Wira Desk\wiradesk.exe", r"PC\bob");

        assert!(
            xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"),
            "on battery the daemon would not start at all"
        );
        assert!(
            xml.contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"),
            "unplugging the charger would terminate a running daemon"
        );
        assert!(
            xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"),
            "absent means the 72-hour default, which kills a tray daemon on day three"
        );
    }

    #[test]
    fn task_xml_carries_the_logon_and_elevation_shape() {
        let xml = task_xml(r"C:\x\wiradesk.exe", r"PC\bob");
        assert!(xml.contains("<LogonTrigger>"));
        assert!(xml.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
        assert!(xml.contains(r"<Command>C:\x\wiradesk.exe</Command>"));
    }

    /// `<Command>` must not be quoted here. The CLI form needs `"..."` so a spaced path
    /// is not split into executable plus arguments; in XML the element boundary does
    /// that job, and quotes would become part of the path.
    #[test]
    fn task_xml_does_not_quote_the_command_the_way_the_cli_must() {
        let xml = task_xml(r"C:\Program Files\Wira Desk\wiradesk.exe", "bob");
        assert!(xml.contains(r"<Command>C:\Program Files\Wira Desk\wiradesk.exe</Command>"));
        assert!(
            !xml.contains("<Command>\""),
            "a quoted path in XML is a path that begins with a quote character"
        );
    }

    /// An install directory is attacker-influenced in the sense that matters: it is a
    /// string that ends up inside a document which becomes an elevated logon task.
    #[test]
    fn xml_escaping_closes_the_injection_route() {
        let nasty = r#"C:\a&b\<x>\"q"\'p'\wiradesk.exe"#;
        let xml = task_xml(nasty, "bob");

        assert!(
            !xml.contains("<x>"),
            "an element survived into the document"
        );
        assert!(xml.contains("&amp;") && xml.contains("&lt;x&gt;"));
        assert!(xml.contains("&quot;") && xml.contains("&apos;"));

        // The closing tag must still be exactly where the parser expects it.
        assert!(xml.contains("</Command>"));
    }

    #[test]
    fn xml_escape_leaves_ordinary_text_alone() {
        assert_eq!(
            xml_escape(r"C:\Program Files\Wira Desk"),
            r"C:\Program Files\Wira Desk"
        );
        assert_eq!(xml_escape(""), "");
    }

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
