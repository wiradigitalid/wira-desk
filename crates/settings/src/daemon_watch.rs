//! Settings' lifetime is bound to the daemon's: it never opens without a
//! running daemon, and it closes itself when that daemon goes away.
//!
//! Everything Settings does is a change to something only the daemon acts on —
//! shortcuts, arrangement, auto-start, and the live key check, which reads what
//! the daemon's hook reports (`DEC-004`). Without the daemon, every one of those
//! is either inert or actively misleading: the Shortcuts pane would show fields
//! that record nothing, and Key Check would report "not running" for every
//! chord. A Settings window left behind after the daemon exits is therefore not
//! a degraded window, it is a lying one.
//!
//! Liveness is decided by whether the daemon's hidden window
//! (`DAEMON_WINDOW_CLASS` / `DAEMON_WINDOW_TITLE`) still exists, polled from the
//! UI thread. `FindWindowW` is the probe rather than a process handle on
//! purpose:
//!
//! * It covers every way the daemon can stop with one mechanism — tray Exit,
//!   `ExitProcess`, a crash, or `taskkill` from Task Manager. Windows destroys
//!   a dead process's windows whether or not it shut down gracefully, so there
//!   is no separate "unexpected exit" path to get right.
//! * It needs no handle to the daemon. The daemon runs elevated; Settings
//!   normally inherits that elevation from it, but not always (it carries no
//!   elevation manifest of its own and can be started at medium integrity).
//!   `OpenProcess` across integrity levels is exactly the kind of access this
//!   watch must not be able to fail on, while finding a window is a read that
//!   works either way.
//! * It is the same probe Settings already uses to tell the Key Check pane
//!   whether the daemon is running, so one answer cannot contradict the other.
//!
//! The cost is up to [`POLL_INTERVAL`] of latency between the daemon exiting and
//! the window closing, which is not a cost that matters here.

use std::time::Duration;

use shared::constants::{DAEMON_WINDOW_CLASS, DAEMON_WINDOW_TITLE};
use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;

/// Environment escape hatch for harnesses that drive the Settings window on its
/// own, with no daemon to attach to — `scripts/verify-settings-runtime.ps1`
/// inspects the real window through UI Automation and deliberately does not
/// require Administrator, so it cannot start a daemon to satisfy the check.
///
/// Same shape as `WIRADESK_SKIP_MANIFEST` in the daemon's build: an opt-out a
/// test harness sets deliberately, never something a user's environment carries
/// by accident.
pub const ALLOW_NO_DAEMON_ENV: &str = "WIRADESK_SETTINGS_ALLOW_NO_DAEMON";

/// How often the watch asks whether the daemon is still there.
///
/// Fast enough that the window disappears with the daemon rather than
/// noticeably after it, slow enough that the poll is free next to the 20ms
/// chord-drain timer already running on the same thread.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Whether the daemon's hidden window currently exists.
pub fn daemon_is_running() -> bool {
    let class = wide(DAEMON_WINDOW_CLASS);
    let title = wide(DAEMON_WINDOW_TITLE);
    // SAFETY: `class` and `title` are NUL-terminated wide strings held in locals
    // that outlive the call, which is all `FindWindowW` reads. It returns a
    // window handle or 0 and never fails in a way that has to be released — the
    // handle is only compared against 0 here, never used.
    unsafe { FindWindowW(class.as_ptr(), title.as_ptr()) != 0 }
}

/// Whether the daemon-required rule is waived for this process.
///
/// Split from the environment read so the rule itself is testable: any value at
/// all waives it, including the empty string, because a harness that went to the
/// trouble of setting the variable meant to set it.
pub fn allow_no_daemon_from(value: Option<&str>) -> bool {
    value.is_some()
}

/// [`allow_no_daemon_from`] against the real process environment.
pub fn allow_no_daemon() -> bool {
    allow_no_daemon_from(std::env::var(ALLOW_NO_DAEMON_ENV).ok().as_deref())
}

/// What `main` should do about the daemon before it builds a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Startup {
    /// A daemon is running, or the rule is waived. Open normally.
    Open,
    /// No daemon and no waiver. Tell the user why and exit without opening.
    RefuseNoDaemon,
}

/// The startup rule, as a pure decision over its two inputs.
pub fn startup_decision(daemon_present: bool, allow_no_daemon: bool) -> Startup {
    if daemon_present || allow_no_daemon {
        Startup::Open
    } else {
        Startup::RefuseNoDaemon
    }
}

/// Message shown when Settings is opened with no daemon running. Names the way
/// back in — the tray icon — rather than only reporting the refusal.
pub const NO_DAEMON_MESSAGE: &str = "Wira Desk is not running, so there is nothing for these \
    settings to change.\n\nStart Wira Desk first, then open Settings from its tray icon.";

/// The running watch: polls a liveness probe and reports the single tick on
/// which the window must close.
///
/// Generic over the probe so the decision is testable without a daemon, a
/// window, or a message loop.
pub struct DaemonWatch<P> {
    probe: P,
    fired: bool,
}

impl<P: Fn() -> bool> DaemonWatch<P> {
    pub fn new(probe: P) -> Self {
        Self {
            probe,
            fired: false,
        }
    }

    /// One tick. Returns `true` exactly once — on the first tick that finds the
    /// daemon gone.
    ///
    /// Firing once matters: closing the Slint window does not stop this timer
    /// synchronously, so a watch that answered "close" on every later tick would
    /// keep asking the event loop to quit after it already had.
    pub fn tick(&mut self) -> bool {
        if self.fired || (self.probe)() {
            return false;
        }
        self.fired = true;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn startup_opens_when_the_daemon_is_running() {
        assert_eq!(startup_decision(true, false), Startup::Open);
    }

    #[test]
    fn startup_refuses_when_no_daemon_and_no_waiver() {
        assert_eq!(startup_decision(false, false), Startup::RefuseNoDaemon);
    }

    #[test]
    fn startup_waiver_opens_without_a_daemon() {
        assert_eq!(startup_decision(false, true), Startup::Open);
    }

    #[test]
    fn any_value_waives_the_rule_including_empty() {
        assert!(allow_no_daemon_from(Some("1")));
        assert!(allow_no_daemon_from(Some("")));
        assert!(!allow_no_daemon_from(None));
    }

    #[test]
    fn watch_stays_quiet_while_the_daemon_is_alive() {
        let mut watch = DaemonWatch::new(|| true);
        for _ in 0..5 {
            assert!(!watch.tick());
        }
    }

    #[test]
    fn watch_fires_on_the_tick_the_daemon_disappears() {
        let alive = Cell::new(true);
        let mut watch = DaemonWatch::new(|| alive.get());

        assert!(!watch.tick());
        alive.set(false);
        assert!(watch.tick(), "the tick that first misses the daemon closes");
    }

    #[test]
    fn watch_fires_only_once() {
        let mut watch = DaemonWatch::new(|| false);
        assert!(watch.tick());
        for _ in 0..5 {
            assert!(
                !watch.tick(),
                "later ticks must not ask the event loop to quit again"
            );
        }
    }

    /// A daemon that comes back after Settings already decided to close does not
    /// un-close it. Settings is bound to the daemon it was opened with, not to
    /// whichever daemon happens to be running later.
    #[test]
    fn a_returning_daemon_does_not_reopen_the_decision() {
        let alive = Cell::new(false);
        let mut watch = DaemonWatch::new(|| alive.get());

        assert!(watch.tick());
        alive.set(true);
        assert!(!watch.tick());
    }
}
