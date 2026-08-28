//! The daily update check.
//!
//! The daemon asks the question; `settings` answers the consequences. That division is the
//! point rather than an accident of where code lives: **the elevated process only reads.**
//! It performs one HTTPS GET and a version comparison, and never downloads, hashes, or
//! executes anything — those belong to `wiradesk-settings.exe`, which is not elevated by
//! design.
//!
//! Shaped like [`crate::health`]: a thread that sleeps, does its work, and posts a bare
//! message to the host window. The result itself travels through shared state rather than
//! through the message, because a version string does not fit in a `WPARAM` and packing one
//! into a heap pointer to be freed by the receiver is a leak waiting for a dropped message.

use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use shared::constants::WM_APP_UPDATE_STATE;
use shared::update::{decide, latest_json_url, Decision, DESCRIPTOR_LIMIT};

/// How long after start the first check runs.
///
/// Not immediately: a machine that just signed in is busy, and an update that has waited a
/// day can wait two more minutes. Not at 24 hours either — a user who shuts down nightly
/// would then never be checked at all, which is the failure mode a pure daily interval has.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(120);

/// The interval the owner asked for: once a day.
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Something worth telling the user, once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// A version not announced before is available.
    UpdateAvailable(String),
    /// A run of failed checks has just begun.
    CheckFailed(String),
}

/// What the tray needs to know, written by the checker thread and read by the UI thread.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateState {
    /// Version offered, when one is. `None` means nothing newer, or not yet checked.
    pub available: Option<String>,
    /// Waiting to be shown once, then taken.
    pub pending: Option<Announcement>,
}

fn state() -> &'static Mutex<UpdateState> {
    static STATE: OnceLock<Mutex<UpdateState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(UpdateState::default()))
}

/// Read the current state. Returns the default if the lock was poisoned, because a tray menu
/// that cannot be drawn is worse than one that briefly forgets an update is available.
pub fn snapshot() -> UpdateState {
    state().lock().map(|s| s.clone()).unwrap_or_default()
}

/// Take the pending announcement, leaving none behind. The caller shows it; nobody shows it
/// twice, however many times the message that woke them arrives.
pub fn take_announcement() -> Option<Announcement> {
    state().lock().ok().and_then(|mut s| s.pending.take())
}

/// What to announce, and the memory that keeps each thing to once.
///
/// **Two rules, both about not repeating yourself, and they are here rather than scattered
/// through the loop so that all of it is reachable from a test.** The checker itself cannot
/// be tested — it sleeps for a day and talks to the network — so everything that decides
/// anything lives in this struct instead.
///
/// *One notification per version.* A daily check that found 0.2.0 yesterday and finds it
/// again today has learned nothing new, and saying so again every morning is how a user
/// turns notifications off. A **different** version is new information and is announced.
///
/// *One notification per run of failures.* A machine offline for a week produces one toast
/// rather than seven. A success ends the run, so the next outage is announced again —
/// announcing only ever once would mean a user whose network broke in March is never told
/// about an outage in July.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Announcer {
    failing: bool,
    announced_version: Option<String>,
}

impl Announcer {
    /// Fold one check result in, and say what if anything the user should see.
    pub fn on_result(&mut self, outcome: &Result<Option<String>, String>) -> Option<Announcement> {
        match outcome {
            Ok(Some(version)) => {
                self.failing = false;
                if self.announced_version.as_deref() == Some(version.as_str()) {
                    return None;
                }
                self.announced_version = Some(version.clone());
                Some(Announcement::UpdateAvailable(version.clone()))
            }
            Ok(None) => {
                // Nothing newer. Clearing the memory matters after the user actually
                // updates: the version they were told about is now the one they run, and
                // holding on to it would be remembering an announcement about the present.
                self.failing = false;
                self.announced_version = None;
                None
            }
            Err(reason) => {
                if self.failing {
                    return None;
                }
                self.failing = true;
                Some(Announcement::CheckFailed(reason.clone()))
            }
        }
    }
}

/// Start the checker. Called once, after the tray window exists.
pub fn spawn(hwnd: isize) {
    thread::spawn(move || {
        let mut announcer = Announcer::default();
        let mut delay = FIRST_CHECK_DELAY;

        loop {
            thread::sleep(delay);
            delay = CHECK_INTERVAL;

            if !crate::config::update_check_enabled() {
                // The setting is read on every tick rather than captured at start, so turning
                // it off in Settings takes effect without restarting the daemon.
                continue;
            }

            let outcome = run_once();
            let announcement = announcer.on_result(&outcome);

            if let Ok(mut s) = state().lock() {
                s.available = outcome.ok().flatten();
                if announcement.is_some() {
                    s.pending = announcement;
                }
            }

            post_state_changed(hwnd);
        }
    });
}

/// One check. `Ok(Some(version))` when something newer exists, `Ok(None)` when current.
fn run_once() -> Result<Option<String>, String> {
    let body = shared::https::get_text(&latest_json_url(), DESCRIPTOR_LIMIT)
        .map_err(|e| format!("{e:?}"))?;

    match decide(env!("CARGO_PKG_VERSION"), &body) {
        Decision::UpToDate => Ok(None),
        Decision::Available(release) => Ok(Some(release.version)),
        // A refused descriptor is not a network failure and must not be reported as one: it
        // means the file we were offered was not ours, which is worth a notification of its
        // own rather than being folded into "could not check".
        Decision::Refused(reason) => Err(format!("{reason:?}")),
    }
}

fn post_state_changed(hwnd: isize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;
    // SAFETY: `hwnd` is the daemon host window, alive for the process lifetime. Both message
    // parameters are zero, so nothing is leaked when a post is dropped — the state itself
    // lives in the mutex above, and this message only asks the UI thread to look at it.
    unsafe {
        PostMessageW(hwnd, WM_APP_UPDATE_STATE, 0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(v: &str) -> Result<Option<String>, String> {
        Ok(Some(v.to_owned()))
    }
    fn current() -> Result<Option<String>, String> {
        Ok(None)
    }
    fn failed() -> Result<Option<String>, String> {
        Err("offline".to_owned())
    }

    /// The rule the owner asked for. A daily check that finds the same version every morning
    /// has learned nothing new, and saying so every morning is how a user turns notifications
    /// off entirely.
    #[test]
    fn a_version_is_announced_once_however_often_it_is_found() {
        let mut a = Announcer::default();
        assert_eq!(
            a.on_result(&found("0.2.0")),
            Some(Announcement::UpdateAvailable("0.2.0".to_owned()))
        );
        for _ in 0..10 {
            assert_eq!(
                a.on_result(&found("0.2.0")),
                None,
                "same version, same day after day"
            );
        }
    }

    #[test]
    fn a_different_version_is_new_information() {
        let mut a = Announcer::default();
        a.on_result(&found("0.2.0"));
        assert_eq!(
            a.on_result(&found("0.3.0")),
            Some(Announcement::UpdateAvailable("0.3.0".to_owned()))
        );
    }

    /// After the user actually updates, the check reports current and the remembered version
    /// is the one they now run. Holding it would be remembering an announcement about the
    /// present.
    #[test]
    fn updating_clears_the_memory() {
        let mut a = Announcer::default();
        a.on_result(&found("0.2.0"));
        assert_eq!(a.on_result(&current()), None);
        assert_eq!(
            a.on_result(&found("0.2.0")),
            Some(Announcement::UpdateAvailable("0.2.0".to_owned())),
            "a version offered again after a period of being current is worth saying again"
        );
    }

    /// A machine offline for a week must produce one toast, not seven.
    #[test]
    fn a_run_of_failures_announces_once() {
        let mut a = Announcer::default();
        assert!(matches!(
            a.on_result(&failed()),
            Some(Announcement::CheckFailed(_))
        ));
        for _ in 0..10 {
            assert_eq!(
                a.on_result(&failed()),
                None,
                "a continuing outage is not news"
            );
        }
    }

    /// Without this, a user whose network broke in March is never told about an outage in
    /// July.
    #[test]
    fn a_success_ends_the_failure_run() {
        let mut a = Announcer::default();
        a.on_result(&failed());
        a.on_result(&current());
        assert!(
            matches!(a.on_result(&failed()), Some(Announcement::CheckFailed(_))),
            "a new outage after a recovery is new information"
        );
    }

    /// The interaction between the two memories, which is where a scattered pair of flags
    /// would have gone wrong: an outage in the middle must not make an already-announced
    /// version look new when the network comes back.
    #[test]
    fn an_outage_does_not_re_announce_the_same_version_afterwards() {
        let mut a = Announcer::default();
        a.on_result(&found("0.2.0"));
        assert!(matches!(
            a.on_result(&failed()),
            Some(Announcement::CheckFailed(_))
        ));
        assert_eq!(
            a.on_result(&found("0.2.0")),
            None,
            "the version was already announced; the outage changed nothing about that"
        );
    }
}
