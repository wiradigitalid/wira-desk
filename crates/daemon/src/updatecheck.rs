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

/// What the tray needs to know, written by the checker thread and read by the UI thread.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateState {
    /// Version offered, when one is. `None` means nothing newer, or not yet checked.
    pub available: Option<String>,
    /// Message for the toast that a failing run raises exactly once.
    pub pending_notice: Option<String>,
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

/// Take the pending notice, leaving none behind. The caller shows it; nobody shows it twice.
pub fn take_notice() -> Option<String> {
    state()
        .lock()
        .ok()
        .and_then(|mut s| s.pending_notice.take())
}

/// Whether a failure should raise a notification, and the memory that keeps it to one.
///
/// **The rule is one notification per run of failures, not one per failure.** A machine
/// offline for a week produces one toast rather than seven. A success clears the memory, so
/// the next outage is announced again — the alternative, notifying only ever once, would
/// mean a user who fixed their network in March is never told about an outage in July.
///
/// Kept as a plain struct with no I/O so the rule is reachable from a test. It is the only
/// part of this module that can be.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FailureRun {
    announced: bool,
}

impl FailureRun {
    /// Record a failure. `true` when the caller should notify.
    pub fn on_failure(&mut self) -> bool {
        if self.announced {
            return false;
        }
        self.announced = true;
        true
    }

    /// Record a success, ending any run in progress.
    pub fn on_success(&mut self) {
        self.announced = false;
    }
}

/// Start the checker. Called once, after the tray window exists.
pub fn spawn(hwnd: isize) {
    thread::spawn(move || {
        let mut failures = FailureRun::default();
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
            let notify = match &outcome {
                Ok(_) => {
                    failures.on_success();
                    None
                }
                Err(reason) => failures.on_failure().then(|| reason.clone()),
            };

            if let Ok(mut s) = state().lock() {
                s.available = outcome.ok().flatten();
                if let Some(message) = notify {
                    s.pending_notice = Some(message);
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

    /// The rule the owner asked for, and the reason it needs a memory at all: a machine
    /// offline for a week must produce one toast, not seven.
    #[test]
    fn a_run_of_failures_announces_once() {
        let mut run = FailureRun::default();
        assert!(
            run.on_failure(),
            "the first failure is the one worth saying"
        );
        for _ in 0..10 {
            assert!(
                !run.on_failure(),
                "a continuing outage is not new information"
            );
        }
    }

    /// Without this, a user whose network broke in March would never be told about an outage
    /// in July -- which is the failure mode of notifying only ever once.
    #[test]
    fn a_success_ends_the_run_so_the_next_outage_is_announced() {
        let mut run = FailureRun::default();
        assert!(run.on_failure());
        assert!(!run.on_failure());

        run.on_success();
        assert!(
            run.on_failure(),
            "a new outage after a recovery is new information"
        );
    }

    #[test]
    fn success_on_a_healthy_run_changes_nothing() {
        let mut run = FailureRun::default();
        run.on_success();
        run.on_success();
        assert!(
            run.on_failure(),
            "successes must not consume the first announcement"
        );
    }
}
