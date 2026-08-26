//! Config reload on `WM_APP_RELOAD_CONFIG`.
//!
//! Settings posts the reload message and reports "saved and applied"; this module
//! is the daemon-side handler. The Hook loaded its shortcuts once at thread start,
//! so without reload a changed shortcut only took effect after restart.
//!
//! Two rules shape everything here:
//!
//! - Reload happens only in response to the message. Nothing here spawns a thread,
//!   arms a timer, or watches the filesystem.
//! - Actors receive owned snapshots, never shared state. The Hook owns shortcut and
//!   bypass configuration; the Worker owns arrangement configuration. Neither reads
//!   the other's, and nothing is mutated across threads.
//!
//! Rejection is all-or-nothing: an unreadable, malformed, or semantically
//! invalid file leaves every actor on its last-known-good configuration and
//! emits exactly one Tier-2 warning. A partially applied reload would be worse
//! than a rejected one, because the user would have no way to tell which half
//! took effect.

use shared::config::LayoutConfig;
use shared::{Config, Shortcut};

use crate::context::BypassPolicy;

/// Owned, immutable configuration for the Hook actor.
#[derive(Debug, Clone)]
pub struct HookSnapshot {
    pub chords: crate::hook::Chords,
    pub bypass: BypassPolicy,
}

/// Owned, immutable configuration for the Worker actor.
/// Layout only, because that is genuinely all the Worker reads today. The
/// `snapping` section holds shortcut *strings*, which belong to whichever actor
/// does the matching, not to the actor that moves windows — carrying a copy
/// here would be a field nothing reads and an invitation to read the wrong one.
#[derive(Debug, Clone)]
pub struct WorkerSnapshot {
    pub layout: LayoutConfig,
}

/// Why a candidate configuration was refused.
/// Kept distinct rather than collapsed into one "invalid" so the Tier-2 warning
/// can tell the user which of three very different things went wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Missing, locked, or otherwise unreadable.
    Unreadable,
    /// Present but not valid TOML, or not shaped like a `Config`.
    Malformed,
    /// Parses as a `Config`, but a shortcut string is not a shortcut.
    InvalidShortcut,
    /// A shortcut is reserved by the Windows operating system.
    ReservedShortcut,
    /// Two actions carry the same chord. Refused wholesale here, unlike at startup, and
    /// `DEC-009` states why: a reload has a last-known-good configuration to keep and a
    /// human who just acted, and the settings process cannot produce a duplicate — so one
    /// arriving here means the file was hand-edited, which deserves a straight answer
    /// rather than a quiet repair.
    DuplicateShortcut,
}

impl RejectReason {
    pub fn message(self) -> &'static str {
        match self {
            RejectReason::Unreadable => {
                "Config reload skipped: file unreadable; keeping current settings"
            }
            RejectReason::Malformed => {
                "Config reload skipped: file is not valid TOML; keeping current settings"
            }
            RejectReason::InvalidShortcut => {
                "Config reload skipped: shortcut is not parseable; keeping current settings"
            }
            RejectReason::ReservedShortcut => {
                "Config reload skipped: contains reserved system shortcuts; keeping current settings"
            }
            RejectReason::DuplicateShortcut => {
                "Config reload skipped: two actions share one shortcut; keeping current settings"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReloadOutcome {
    Applied { auto_start: bool },
    Rejected(RejectReason),
}

/// Where the candidate configuration text comes from.
/// A seam rather than a direct `std::fs` call so the reject paths can be tested
/// without a filesystem — the failure modes that matter here are precisely the
/// ones that are awkward to stage on disk.
pub trait ConfigSource {
    fn read(&self) -> Result<String, ()>;
}

/// Delivery of owned snapshots to the actors that own them.
pub trait ActorSink {
    /// Returns false if the Hook actor could not be reached.
    fn deliver_hook(&mut self, snapshot: HookSnapshot) -> bool;
    fn deliver_worker(&mut self, snapshot: WorkerSnapshot);
}

/// 's Task Scheduler registration, behind a seam.
pub trait AutoStartControl {
    fn is_registered(&self) -> bool;
    fn enable(&mut self) -> bool;
    fn disable(&mut self) -> bool;
}

/// Tier-2 warning channel.
pub trait WarnSink {
    fn warn(&mut self, message: &str);
}

/// Validate candidate text into the two owned snapshots.
/// Pure: no IO, no Win32, no globals. Everything that can reject a reload is
/// decided here, before any actor has been touched.
pub fn validate(text: &str) -> Result<(Config, HookSnapshot, WorkerSnapshot), RejectReason> {
    let cfg = Config::from_toml_str(text).map_err(|_| RejectReason::Malformed)?;

    // Both shortcuts must parse. `load_shortcuts` substitutes a default on a
    // bad string at startup, which is right for startup — the daemon must come
    // up — but wrong here: silently ignoring half of what the user just saved
    // while reporting success is the exact dishonesty this story exists to fix.
    let primary = Shortcut::parse(&cfg.switcher.shortcut).ok_or(RejectReason::InvalidShortcut)?;
    let fallback =
        Shortcut::parse(&cfg.switcher.fallback_shortcut).ok_or(RejectReason::InvalidShortcut)?;
    let snap_left =
        Shortcut::parse(&cfg.snapping.snap_half_left).ok_or(RejectReason::InvalidShortcut)?;
    let snap_right =
        Shortcut::parse(&cfg.snapping.snap_half_right).ok_or(RejectReason::InvalidShortcut)?;
    let snap_top =
        Shortcut::parse(&cfg.snapping.snap_half_top).ok_or(RejectReason::InvalidShortcut)?;
    let snap_bottom =
        Shortcut::parse(&cfg.snapping.snap_half_bottom).ok_or(RejectReason::InvalidShortcut)?;
    let snap_maximize =
        Shortcut::parse(&cfg.snapping.snap_maximize).ok_or(RejectReason::InvalidShortcut)?;
    let stack = Shortcut::parse(&cfg.layout.stack_shortcut).ok_or(RejectReason::InvalidShortcut)?;
    let move_next_monitor = Shortcut::parse(&cfg.layout.move_next_monitor_shortcut)
        .ok_or(RejectReason::InvalidShortcut)?;

    let chords = crate::hook::Chords {
        primary: Some(primary),
        fallback: Some(fallback),
        snap_left: Some(snap_left),
        snap_right: Some(snap_right),
        snap_top: Some(snap_top),
        snap_bottom: Some(snap_bottom),
        snap_maximize: Some(snap_maximize),
        move_next_monitor: Some(move_next_monitor),
        stack: Some(stack),
    };

    // Reject reserved shortcuts on reload. Walked through the declared sequence rather than
    // a hand-listed array, so a chord added to `Chords` cannot be missed here — the omission
    // would be silent, and its effect would be a reserved chord accepted on reload but
    // refused at startup.
    for slot in chords.in_declared_order() {
        if let Some(sc) = slot.chord {
            if shared::shortcut::reservation(&sc).is_some() {
                return Err(RejectReason::ReservedShortcut);
            }
        }
    }

    // Two actions on one chord. Refused wholesale rather than unbound, which is the
    // opposite of what startup does — deliberately, per `DEC-009` and `BR-6`. Walked over
    // the declared sequence so a chord added to `Chords` is covered without a second list.
    let slots = chords.in_declared_order();
    for i in 1..slots.len() {
        let Some(chord) = slots[i].chord else {
            continue;
        };
        if slots[..i].iter().any(|s| s.chord == Some(chord)) {
            return Err(RejectReason::DuplicateShortcut);
        }
    }

    let hook = HookSnapshot {
        chords,
        bypass: BypassPolicy::from_config(&cfg.vm_bypass),
    };
    let worker = WorkerSnapshot {
        layout: cfg.layout.clone(),
    };
    Ok((cfg, hook, worker))
}

/// Read, validate, and — only if valid — hand each actor its own snapshot.
/// Called from the `WM_APP_RELOAD_CONFIG` arm and nowhere else.
pub fn reload<S, A, C, W>(
    source: &S,
    sink: &mut A,
    autostart: &mut C,
    warn: &mut W,
) -> ReloadOutcome
where
    S: ConfigSource + ?Sized,
    A: ActorSink + ?Sized,
    C: AutoStartControl + ?Sized,
    W: WarnSink + ?Sized,
{
    let text = match source.read() {
        Ok(t) => t,
        Err(()) => return reject(warn, RejectReason::Unreadable),
    };

    let (cfg, hook, worker) = match validate(&text) {
        Ok(v) => v,
        Err(reason) => return reject(warn, reason),
    };

    // Worker first, and deliberately: it is the same thread as the caller, so
    // it cannot fail. Delivering it after a failed Hook post would be a partial
    // update, which the reject contract forbids.
    if !sink.deliver_hook(hook) {
        return reject(warn, RejectReason::Unreadable);
    }
    sink.deliver_worker(worker);

    converge_auto_start(autostart, cfg.general.auto_start);

    ReloadOutcome::Applied {
        auto_start: cfg.general.auto_start,
    }
}

fn reject<W: WarnSink + ?Sized>(warn: &mut W, reason: RejectReason) -> ReloadOutcome {
    // Exactly one warning per rejected reload — the AC latches Tier-2, and a
    // second warning for the same event would read as a second failure.
    warn.warn(reason.message());
    ReloadOutcome::Rejected(reason)
}

/// Drive Task Scheduler registration to the requested state, and no further.
/// Reading the current state first is what makes repeated reloads idempotent:
/// `schtasks /Create` on an already-registered task is a process spawn with no
/// effect, and the AC asks duplicate requests to converge, not to churn.
fn converge_auto_start<C: AutoStartControl + ?Sized>(autostart: &mut C, desired: bool) {
    let current = autostart.is_registered();
    if current == desired {
        return;
    }
    if desired {
        autostart.enable();
    } else {
        autostart.disable();
    }
}

// ── Production wiring ────────────────────────────────────────────────────────

/// Reads the real `%APPDATA%\WiraDesk\config.toml`.
/// Deliberately not `Config::load_or_default`: that helper substitutes defaults
/// for an unreadable or malformed file, which is correct at startup but would
/// make a corrupt file indistinguishable from a valid one here — and this
/// module's whole job is to tell those apart.
pub struct FileSource;

impl ConfigSource for FileSource {
    fn read(&self) -> Result<String, ()> {
        std::fs::read_to_string(shared::config_path()).map_err(|_| ())
    }
}

/// Hands the Hook its snapshot by thread message, and the Worker its own
/// directly — the Worker *is* the calling thread.
pub struct Win32Sink {
    pub hook_thread_id: u32,
}

impl ActorSink for Win32Sink {
    fn deliver_hook(&mut self, snapshot: HookSnapshot) -> bool {
        if self.hook_thread_id == 0 {
            return false;
        }
        // The snapshot is staged in a slot the Hook thread owns the other end of, and the
        // message carries nothing. Passing the pointer in `lParam` would work, but it would
        // leave the handler unable to tell a staged pointer from an arbitrary integer — see
        // `hook::PENDING_SNAPSHOT`.
        let staged = crate::hook::stage_snapshot(snapshot);
        // SAFETY: no pointer and no ownership crosses this call — both `wParam` and
        // `lParam` are zero, so this is a pure wake-up. `hook_thread_id` is checked
        // non-zero above, and an id that no longer names a live thread makes
        // `PostThreadMessageW` fail rather than fault.
        let posted = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                self.hook_thread_id,
                shared::constants::WM_APP_CONFIG_SNAPSHOT,
                0,
                0,
            )
        } != 0;
        if !posted {
            // No wake-up will arrive, so the staged snapshot would sit there until the next
            // reload replaced it. Reclaim it now: leaking here would be invisible and
            // unbounded across repeated saves.
            crate::hook::unstage_snapshot(staged);
        }
        posted
    }

    fn deliver_worker(&mut self, snapshot: WorkerSnapshot) {
        crate::worker::install_config_snapshot(snapshot);
    }
}

/// 's accepted Task Scheduler implementation, unchanged.
pub struct TaskSchedulerAutoStart;

impl AutoStartControl for TaskSchedulerAutoStart {
    fn is_registered(&self) -> bool {
        crate::autostart::is_registered()
    }
    fn enable(&mut self) -> bool {
        crate::autostart::enable()
    }
    fn disable(&mut self) -> bool {
        crate::autostart::disable()
    }
}

/// Tier-2 warning: log line plus a latched tray dot, never a popup.
pub struct TrayWarn {
    pub worker_hwnd: windows_sys::Win32::Foundation::HWND,
}

impl WarnSink for TrayWarn {
    fn warn(&mut self, message: &str) {
        crate::log::warn(self.worker_hwnd, message);
    }
}

/// Entry point for the `WM_APP_RELOAD_CONFIG` arm. Called from there and
/// nowhere else — there is no timer, watcher, or idle wake-up in this module.
pub fn handle_reload_message(
    worker_hwnd: windows_sys::Win32::Foundation::HWND,
    hook_thread_id: u32,
) -> ReloadOutcome {
    let mut sink = Win32Sink { hook_thread_id };
    let mut autostart = TaskSchedulerAutoStart;
    let mut warn = TrayWarn { worker_hwnd };
    reload(&FileSource, &mut sink, &mut autostart, &mut warn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeSource(Result<String, ()>);
    impl ConfigSource for FakeSource {
        fn read(&self) -> Result<String, ()> {
            self.0.clone()
        }
    }

    #[derive(Default)]
    struct FakeSink {
        hook: RefCell<Vec<HookSnapshot>>,
        worker: RefCell<Vec<WorkerSnapshot>>,
        hook_reachable: bool,
    }
    impl FakeSink {
        fn reachable() -> Self {
            Self {
                hook_reachable: true,
                ..Default::default()
            }
        }
    }
    impl ActorSink for FakeSink {
        fn deliver_hook(&mut self, snapshot: HookSnapshot) -> bool {
            if !self.hook_reachable {
                return false;
            }
            self.hook.borrow_mut().push(snapshot);
            true
        }
        fn deliver_worker(&mut self, snapshot: WorkerSnapshot) {
            self.worker.borrow_mut().push(snapshot);
        }
    }

    #[derive(Default)]
    struct FakeAutoStart {
        registered: bool,
        enable_calls: u32,
        disable_calls: u32,
    }
    impl AutoStartControl for FakeAutoStart {
        fn is_registered(&self) -> bool {
            self.registered
        }
        fn enable(&mut self) -> bool {
            self.enable_calls += 1;
            self.registered = true;
            true
        }
        fn disable(&mut self) -> bool {
            self.disable_calls += 1;
            self.registered = false;
            true
        }
    }

    #[derive(Default)]
    struct FakeWarn(Vec<String>);
    impl WarnSink for FakeWarn {
        fn warn(&mut self, message: &str) {
            self.0.push(message.to_string());
        }
    }

    const VALID: &str = r#"
[general]
auto_start = false
[switcher]
shortcut = "win+backtick"
fallback_shortcut = "alt+backtick"
"#;

    fn run(source: FakeSource) -> (ReloadOutcome, FakeSink, FakeAutoStart, FakeWarn) {
        let mut sink = FakeSink::reachable();
        let mut auto = FakeAutoStart::default();
        let mut warn = FakeWarn::default();
        let outcome = reload(&source, &mut sink, &mut auto, &mut warn);
        (outcome, sink, auto, warn)
    }

    #[test]
    fn unreadable_file_changes_nothing_and_warns_once() {
        let (outcome, sink, _, warn) = run(FakeSource(Err(())));
        assert_eq!(outcome, ReloadOutcome::Rejected(RejectReason::Unreadable));
        assert!(
            sink.hook.borrow().is_empty(),
            "hook must keep last-known-good"
        );
        assert!(sink.worker.borrow().is_empty());
        assert_eq!(warn.0.len(), 1, "exactly one Tier-2 warning per rejection");
    }

    #[test]
    fn malformed_toml_changes_nothing_and_warns_once() {
        let (outcome, sink, _, warn) = run(FakeSource(Ok("this is not toml {{{".into())));
        assert_eq!(outcome, ReloadOutcome::Rejected(RejectReason::Malformed));
        assert!(sink.hook.borrow().is_empty());
        assert!(sink.worker.borrow().is_empty());
        assert_eq!(warn.0.len(), 1);
    }

    #[test]
    fn unparseable_shortcut_is_rejected_rather_than_defaulted() {
        // Startup substitutes a default here; reload must not, or the user is
        // told a shortcut was applied that in fact was not.
        let text = VALID.replace("win+backtick", "not-a-shortcut");
        let (outcome, sink, _, warn) = run(FakeSource(Ok(text)));
        assert_eq!(
            outcome,
            ReloadOutcome::Rejected(RejectReason::InvalidShortcut)
        );
        assert!(sink.hook.borrow().is_empty());
        assert_eq!(warn.0.len(), 1);
    }

    #[test]
    fn no_actor_is_updated_when_the_hook_cannot_be_reached() {
        let mut sink = FakeSink::default(); // hook unreachable
        let mut auto = FakeAutoStart::default();
        let mut warn = FakeWarn::default();
        let outcome = reload(
            &FakeSource(Ok(VALID.into())),
            &mut sink,
            &mut auto,
            &mut warn,
        );
        assert!(matches!(outcome, ReloadOutcome::Rejected(_)));
        assert!(
            sink.worker.borrow().is_empty(),
            "delivering to the Worker after a failed Hook post is a partial update"
        );
    }

    #[test]
    fn valid_config_reaches_both_actors_without_warning() {
        let (outcome, sink, _, warn) = run(FakeSource(Ok(VALID.into())));
        assert_eq!(outcome, ReloadOutcome::Applied { auto_start: false });
        assert_eq!(sink.hook.borrow().len(), 1);
        assert_eq!(sink.worker.borrow().len(), 1);
        assert!(warn.0.is_empty());
    }

    #[test]
    fn a_duplicate_chord_rejects_the_whole_reload() {
        // A hand-edited file binding two actions to one chord. The settings process refuses
        // to save such a pair, so a duplicate arriving on this path means the file was
        // edited by hand — and `DEC-009` says that deserves a straight answer.
        let text = format!(
            "{VALID}
[snapping]
snap_half_left = \"ctrl+alt+left\"
snap_half_right = \"ctrl+alt+left\"
"
        );
        let (outcome, sink, _, warn) = run(FakeSource(Ok(text)));
        assert_eq!(
            outcome,
            ReloadOutcome::Rejected(RejectReason::DuplicateShortcut)
        );
        assert!(
            sink.hook.borrow().is_empty(),
            "no actor may receive a partial update"
        );
        assert!(sink.worker.borrow().is_empty());
        assert_eq!(warn.0.len(), 1, "exactly one Tier-2 warning per rejection");
    }

    #[test]
    fn a_rejected_reload_leaves_every_actor_on_last_known_good() {
        // The property the all-or-nothing contract exists for, asserted for the new reason
        // rather than only for the ones that already had it: a half-applied reload leaves the
        // user unable to tell which half took effect.
        let text = format!(
            "{VALID}
[snapping]
snap_half_top = \"ctrl+alt+up\"
snap_half_bottom = \"ctrl+alt+up\"
"
        );
        let (outcome, sink, auto, _) = run(FakeSource(Ok(text)));
        assert!(matches!(outcome, ReloadOutcome::Rejected(_)));
        assert!(sink.hook.borrow().is_empty());
        assert!(sink.worker.borrow().is_empty());
        assert_eq!(
            (auto.enable_calls, auto.disable_calls),
            (0, 0),
            "a refused reload must not converge auto-start either"
        );
    }

    #[test]
    fn the_shipped_defaults_do_not_collide_with_each_other() {
        // The guard that would have caught the collision `DEC-008` created if the default
        // for a new field were ever chosen carelessly.
        let (outcome, _, _, _) = run(FakeSource(Ok(String::new())));
        assert!(
            matches!(outcome, ReloadOutcome::Applied { .. }),
            "an empty file is all defaults, and they must be mutually distinct"
        );
    }

    #[test]
    fn shortcuts_in_the_snapshot_are_the_ones_that_were_saved() {
        let text = VALID.replace("alt+backtick", "ctrl+alt+tab");
        let (_, sink, _, _) = run(FakeSource(Ok(text)));
        let hook = &sink.hook.borrow()[0];
        assert_eq!(hook.chords.primary, Shortcut::parse("win+backtick"));
        assert_eq!(hook.chords.fallback, Shortcut::parse("ctrl+alt+tab"));
    }

    #[test]
    fn auto_start_is_enabled_only_when_it_is_not_already_registered() {
        let text = VALID.replace("auto_start = false", "auto_start = true");
        let mut sink = FakeSink::reachable();
        let mut auto = FakeAutoStart {
            registered: true,
            ..Default::default()
        };
        let mut warn = FakeWarn::default();
        reload(&FakeSource(Ok(text)), &mut sink, &mut auto, &mut warn);
        assert_eq!(auto.enable_calls, 0, "already registered; nothing to do");
    }

    #[test]
    fn auto_start_converges_when_the_requested_state_differs() {
        let text = VALID.replace("auto_start = false", "auto_start = true");
        let mut sink = FakeSink::reachable();
        let mut auto = FakeAutoStart::default();
        let mut warn = FakeWarn::default();
        reload(&FakeSource(Ok(text)), &mut sink, &mut auto, &mut warn);
        assert_eq!(auto.enable_calls, 1);
        assert!(auto.registered);
    }

    #[test]
    fn auto_start_disable_converges_and_then_stays_put() {
        let mut sink = FakeSink::reachable();
        let mut auto = FakeAutoStart {
            registered: true,
            ..Default::default()
        };
        let mut warn = FakeWarn::default();
        // auto_start = false in VALID.
        reload(
            &FakeSource(Ok(VALID.into())),
            &mut sink,
            &mut auto,
            &mut warn,
        );
        reload(
            &FakeSource(Ok(VALID.into())),
            &mut sink,
            &mut auto,
            &mut warn,
        );
        assert_eq!(auto.disable_calls, 1, "duplicate reloads must converge");
    }

    #[test]
    fn a_rejected_reload_never_touches_auto_start() {
        let mut sink = FakeSink::reachable();
        let mut auto = FakeAutoStart::default();
        let mut warn = FakeWarn::default();
        reload(&FakeSource(Err(())), &mut sink, &mut auto, &mut warn);
        assert_eq!(auto.enable_calls, 0);
        assert_eq!(auto.disable_calls, 0);
    }

    #[test]
    fn missing_sections_fall_back_to_defaults_rather_than_rejecting() {
        // `#[serde(default)]` on Config means a sparse file is valid, not
        // malformed. Rejecting it would strand users whose file predates a
        // newly added section.
        let (outcome, _, _, warn) = run(FakeSource(Ok("[general]\nauto_start = false\n".into())));
        assert!(matches!(outcome, ReloadOutcome::Applied { .. }));
        assert!(warn.0.is_empty());
    }

    #[test]
    fn each_reject_reason_has_its_own_message() {
        let msgs = [
            RejectReason::Unreadable.message(),
            RejectReason::Malformed.message(),
            RejectReason::InvalidShortcut.message(),
        ];
        for (i, a) in msgs.iter().enumerate() {
            for b in msgs.iter().skip(i + 1) {
                assert_ne!(a, b, "reject reasons must be distinguishable in the log");
            }
        }
    }
}
