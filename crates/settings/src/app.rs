//! Settings shell, shortcut capturer, and first-run tutorial
//! (Stories 5.3, 5.4, 5.5).
//! Editing is *staged*: the user edits a draft, and nothing reaches disk until
//! Save validates it. A rejected save leaves both the draft and the on-disk
//! file untouched, so an invalid entry can be corrected instead of losing work.

use shared::Config;

use crate::persistence::{save_and_notify, validate_shortcut, SaveOutcome, ShortcutError};
use crate::theme::{
    self, ThemeMode, LISTENING_ANNOUNCEMENT, STACK_WIDTH_INPUT, STACK_WIDTH_SLIDER,
    TOGGLE_AUTO_START, TOGGLE_OVERLAPPING_STACK,
};

/// Which pane the shell is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    General,
    Shortcuts,
    Layout,
    About,
}

impl Pane {
    pub const ALL: [Pane; 4] = [Pane::General, Pane::Shortcuts, Pane::Layout, Pane::About];

    /// Accessible name, also used as the visible tab label.
    pub fn label(self) -> &'static str {
        match self {
            Pane::General => "General",
            Pane::Shortcuts => "Shortcuts",
            Pane::Layout => "Layout",
            Pane::About => "About",
        }
    }

    /// Reverse lookup of [`Pane::label`].
    /// Lets the renderer draw its tab bar by iterating [`focus_order`]'s
    /// declared sequence instead of a second, independent iteration over
    /// [`Pane::ALL`] — the two cannot drift apart if only one of them is the
    /// actual source of the draw order.
    pub fn from_label(label: &str) -> Option<Pane> {
        Pane::ALL.into_iter().find(|p| p.label() == label)
    }
}

/// Which shortcut field the capturer is bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutField {
    Switcher,
    Fallback,
    SnapLeft,
    SnapRight,
    SnapMaximize,
    Stack,
}

impl ShortcutField {
    pub const ALL: [ShortcutField; 6] = [
        ShortcutField::Switcher,
        ShortcutField::Fallback,
        ShortcutField::SnapLeft,
        ShortcutField::SnapRight,
        ShortcutField::SnapMaximize,
        ShortcutField::Stack,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ShortcutField::Switcher => "Switch windows of the same application",
            ShortcutField::Fallback => "Fallback switch shortcut",
            ShortcutField::SnapLeft => "Snap to left half",
            ShortcutField::SnapRight => "Snap to right half",
            ShortcutField::SnapMaximize => "Maximize",
            ShortcutField::Stack => "Overlapping stack",
        }
    }

    /// Reverse lookup of [`ShortcutField::label`].
    /// Lets the Shortcuts pane draw its fields by iterating [`focus_order`]'s
    /// declared sequence instead of a second, independent iteration over
    /// [`ShortcutField::ALL`] — the two cannot drift apart if only one of
    /// them is the actual source of the draw order.
    pub fn from_label(label: &str) -> Option<ShortcutField> {
        ShortcutField::ALL.into_iter().find(|f| f.label() == label)
    }

    pub fn get(self, cfg: &Config) -> &str {
        match self {
            ShortcutField::Switcher => &cfg.switcher.shortcut,
            ShortcutField::Fallback => &cfg.switcher.fallback_shortcut,
            ShortcutField::SnapLeft => &cfg.snapping.snap_half_left,
            ShortcutField::SnapRight => &cfg.snapping.snap_half_right,
            ShortcutField::SnapMaximize => &cfg.snapping.snap_maximize,
            ShortcutField::Stack => &cfg.layout.stack_shortcut,
        }
    }

    pub fn set(self, cfg: &mut Config, value: String) {
        match self {
            ShortcutField::Switcher => cfg.switcher.shortcut = value,
            ShortcutField::Fallback => cfg.switcher.fallback_shortcut = value,
            ShortcutField::SnapLeft => cfg.snapping.snap_half_left = value,
            ShortcutField::SnapRight => cfg.snapping.snap_half_right = value,
            ShortcutField::SnapMaximize => cfg.snapping.snap_maximize = value,
            ShortcutField::Stack => cfg.layout.stack_shortcut = value,
        }
    }
}

/// Shortcut capturer state.
/// `Listening` is a first-class state rather than a boolean on the widget, so
/// the accessible value can report it — the accessibility contract forbids
/// communicating it through visual text alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureState {
    Idle,
    Listening(ShortcutField),
}

impl CaptureState {
    pub fn is_listening_for(&self, field: ShortcutField) -> bool {
        matches!(self, CaptureState::Listening(f) if *f == field)
    }

    /// Accessible value announced for a control in this state.
    pub fn announcement(&self, field: ShortcutField, current: &str) -> String {
        if self.is_listening_for(field) {
            LISTENING_ANNOUNCEMENT.to_string()
        } else {
            format!("{}. Current shortcut {}.", field.label(), current)
        }
    }
}

/// First-run onboarding progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    TrySwitching,
    Done,
}

impl OnboardingStep {
    pub fn next(self) -> OnboardingStep {
        match self {
            OnboardingStep::Welcome => OnboardingStep::TrySwitching,
            OnboardingStep::TrySwitching | OnboardingStep::Done => OnboardingStep::Done,
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            OnboardingStep::Welcome => "Welcome to Wira Desk",
            OnboardingStep::TrySwitching => "Try switching windows",
            OnboardingStep::Done => "You are all set",
        }
    }

    /// Cycling stays inside the active application rather than across everything,
    /// which is what makes it different from Alt+Tab.
    pub fn body(self) -> &'static str {
        match self {
            OnboardingStep::Welcome => {
                "Wira Desk switches between windows of the application you are already using, \
                 instead of every window on the system like Alt+Tab does. Cycling also stays \
                 on the current monitor and virtual desktop, rather than reaching across all \
                 of them."
            }
            OnboardingStep::TrySwitching => {
                "Open a second window of this application, then press the switch shortcut. \
                 Focus moves to the next window of the same application, on the same monitor."
            }
            OnboardingStep::Done => "You can change any shortcut later from the Shortcuts pane.",
        }
    }
}

/// Result of a Save attempt, in a form the UI can render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFeedback {
    None,
    Saved { reload_signalled: bool },
    Error(String),
}

/// Human-readable message for a validation failure.
pub fn describe(field: &str, err: ShortcutError) -> String {
    let reason = match err {
        ShortcutError::UnsupportedToken => "contains a key name Wira Desk does not recognize",
        ShortcutError::NoMainKey => "needs a main key in addition to modifiers",
        ShortcutError::MultipleMainKeys => "may only contain one main key",
        ShortcutError::NoModifier => "needs at least one modifier (ctrl, win, alt, or shift)",
        ShortcutError::Unrepresentable => "cannot be saved in a supported form",
    };
    format!("{field} {reason}.")
}

/// The Settings model. Rendering is a thin layer over this; every decision that
/// matters is testable without a window.
pub struct SettingsModel {
    pub saved: Config,
    pub draft: Config,
    pub pane: Pane,
    pub capture: CaptureState,
    pub feedback: SaveFeedback,
    pub theme: ThemeMode,
    pub onboarding: Option<OnboardingStep>,
}

impl SettingsModel {
    pub fn new(saved: Config, onboarding: bool) -> Self {
        SettingsModel {
            draft: saved.clone(),
            saved,
            pane: Pane::General,
            capture: CaptureState::Idle,
            feedback: SaveFeedback::None,
            theme: theme::detect_theme(),
            onboarding: onboarding.then_some(OnboardingStep::Welcome),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.draft != self.saved
    }

    pub fn begin_capture(&mut self, field: ShortcutField) {
        self.capture = CaptureState::Listening(field);
    }

    /// Switch the active pane, cancelling an in-progress capture if it is
    /// left running in the background.
    /// Without this, navigating away from Shortcuts mid-capture leaves the
    /// capturer silently `Listening`; returning to the pane later resumes it
    /// as if nothing happened, and a keystroke meant for another purpose could
    /// still be waiting to be consumed as a shortcut edit.
    pub fn set_pane(&mut self, pane: Pane) {
        if pane != Pane::Shortcuts {
            self.cancel_capture();
        }
        self.pane = pane;
    }

    /// Cancel capture without changing anything — the Escape affordance.
    pub fn cancel_capture(&mut self) {
        self.capture = CaptureState::Idle;
    }

    /// Accept a captured combination.
    /// Validation happens here, before the draft changes, so an unusable
    /// combination never becomes the displayed value.
    pub fn accept_capture(&mut self, combination: &str) -> Result<(), ShortcutError> {
        let CaptureState::Listening(field) = self.capture else {
            return Ok(());
        };
        let canonical = validate_shortcut(combination)?;
        field.set(&mut self.draft, canonical);
        self.capture = CaptureState::Idle;
        Ok(())
    }

    /// Discard edits.
    pub fn revert(&mut self) {
        self.draft = self.saved.clone();
        self.capture = CaptureState::Idle;
        self.feedback = SaveFeedback::None;
    }

    /// Validate, persist, and signal reload.
    pub fn save(&mut self, path: &std::path::Path) {
        match save_and_notify(&self.draft, path) {
            SaveOutcome::Saved { reload_signalled } => {
                self.saved = self.draft.clone();
                self.feedback = SaveFeedback::Saved { reload_signalled };
            }
            SaveOutcome::Rejected(field, err) => {
                self.feedback = SaveFeedback::Error(describe(field, err));
            }
            SaveOutcome::WriteFailed(msg) => {
                self.feedback = SaveFeedback::Error(format!("Could not save settings: {msg}"));
            }
        }
    }

    /// Advance the tutorial. Returns true once it has finished.
    pub fn advance_onboarding(&mut self) -> bool {
        match self.onboarding {
            Some(OnboardingStep::Done) | None => true,
            Some(step) => {
                let next = step.next();
                self.onboarding = Some(next);
                next == OnboardingStep::Done
            }
        }
    }

    /// Skip Tutorial. Equivalent to completing it: a valid configuration is
    /// still written so onboarding does not repeat unintentionally.
    pub fn skip_onboarding(&mut self) {
        self.onboarding = Some(OnboardingStep::Done);
    }
}

/// Deterministic keyboard focus order for the current pane.
/// Declared explicitly rather than left to widget declaration order, so a
/// future reordering of the drawing code cannot silently scramble tab order.
/// The renderer calls [`assert_focus_order`] against the stops it actually
/// drew, so the declaration and the drawing cannot drift apart unnoticed.
pub fn focus_order(pane: Pane) -> Vec<&'static str> {
    let mut order: Vec<&'static str> = Pane::ALL.iter().map(|p| p.label()).collect();
    match pane {
        Pane::General => {
            order.push(TOGGLE_AUTO_START.name);
        }
        Pane::Shortcuts => {
            for f in ShortcutField::ALL {
                order.push(f.label());
            }
        }
        Pane::Layout => {
            order.push(TOGGLE_OVERLAPPING_STACK.name);
            order.push(STACK_WIDTH_SLIDER.name);
            order.push(STACK_WIDTH_INPUT.name);
        }
        Pane::About => {}
    }
    order.push("Save");
    order.push("Revert");
    order
}

/// Compare the stops the renderer actually produced against the declaration.
/// Returns the first mismatch, or `None` when they agree. In debug builds the
/// renderer treats a mismatch as a bug and reports it; in release it is ignored
/// so a cosmetic drift can never crash a user's Settings window.
#[cfg(debug_assertions)]
pub fn focus_order_mismatch(pane: Pane, drawn: &[&str]) -> Option<String> {
    let expected = focus_order(pane);
    if expected.len() != drawn.len() {
        return Some(format!(
            "{pane:?}: declared {} focus stops, drew {}",
            expected.len(),
            drawn.len()
        ));
    }
    expected
        .iter()
        .zip(drawn.iter())
        .position(|(e, d)| e != d)
        .map(|i| {
            format!(
                "{pane:?}: stop {i} declared {:?}, drew {:?}",
                expected[i], drawn[i]
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> SettingsModel {
        SettingsModel::new(Config::default(), false)
    }

    #[test]
    #[cfg(debug_assertions)]
    fn matching_drawn_order_reports_no_mismatch() {
        for pane in Pane::ALL {
            let declared = focus_order(pane);
            assert_eq!(focus_order_mismatch(pane, &declared), None);
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn a_reordered_stop_is_detected() {
        let mut drawn = focus_order(Pane::Shortcuts);
        drawn.swap(4, 5);
        let mismatch = focus_order_mismatch(Pane::Shortcuts, &drawn);
        assert!(mismatch.is_some(), "swapped focus stops went unnoticed");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn a_missing_stop_is_detected() {
        let mut drawn = focus_order(Pane::General);
        drawn.pop();
        assert!(focus_order_mismatch(Pane::General, &drawn).is_some());
    }

    // ── Staged editing ──────────────────────────────────────────────────────

    #[test]
    fn a_new_model_is_not_dirty() {
        assert!(!model().is_dirty());
    }

    #[test]
    fn editing_the_draft_marks_dirty_without_touching_saved() {
        let mut m = model();
        m.draft.general.auto_start = true;
        assert!(m.is_dirty());
        assert!(!m.saved.general.auto_start);
    }

    #[test]
    fn revert_discards_edits() {
        let mut m = model();
        m.draft.layout.stack_width_percent = 90;
        m.revert();
        assert!(!m.is_dirty());
        assert_eq!(m.draft.layout.stack_width_percent, 50);
    }

    // ── Shortcut capture ────────────────────────────────────────────────

    #[test]
    fn capture_starts_idle() {
        assert_eq!(model().capture, CaptureState::Idle);
    }

    #[test]
    fn beginning_capture_targets_exactly_one_field() {
        let mut m = model();
        m.begin_capture(ShortcutField::SnapLeft);
        assert!(m.capture.is_listening_for(ShortcutField::SnapLeft));
        assert!(!m.capture.is_listening_for(ShortcutField::SnapRight));
    }

    #[test]
    fn cancelling_capture_changes_nothing() {
        let mut m = model();
        let before = m.draft.clone();
        m.begin_capture(ShortcutField::Switcher);
        m.cancel_capture();
        assert_eq!(m.capture, CaptureState::Idle);
        assert_eq!(m.draft, before);
    }

    #[test]
    fn accepting_a_valid_capture_stores_the_canonical_form() {
        let mut m = model();
        m.begin_capture(ShortcutField::Switcher);
        m.accept_capture("SHIFT+Ctrl+A").unwrap();
        assert_eq!(m.draft.switcher.shortcut, "ctrl+shift+a");
        assert_eq!(m.capture, CaptureState::Idle);
    }

    #[test]
    fn rejecting_a_capture_leaves_the_draft_and_keeps_listening() {
        let mut m = model();
        let before = m.draft.clone();
        m.begin_capture(ShortcutField::Switcher);
        assert_eq!(m.accept_capture("ctrl+win"), Err(ShortcutError::NoMainKey));
        assert_eq!(m.draft, before, "an invalid capture modified the draft");
        assert!(
            m.capture.is_listening_for(ShortcutField::Switcher),
            "capture should stay open so the user can try again"
        );
    }

    #[test]
    fn switching_pane_away_from_shortcuts_cancels_an_open_capture() {
        let mut m = model();
        m.set_pane(Pane::Shortcuts);
        m.begin_capture(ShortcutField::Switcher);
        assert!(m.capture.is_listening_for(ShortcutField::Switcher));

        m.set_pane(Pane::General);
        assert_eq!(m.pane, Pane::General);
        assert_eq!(
            m.capture,
            CaptureState::Idle,
            "leaving Shortcuts mid-capture must not leave it silently Listening"
        );
    }

    #[test]
    fn switching_between_other_panes_does_not_disturb_an_idle_capture() {
        let mut m = model();
        m.set_pane(Pane::Layout);
        m.set_pane(Pane::About);
        assert_eq!(m.pane, Pane::About);
        assert_eq!(m.capture, CaptureState::Idle);
    }

    #[test]
    fn accepting_without_listening_is_a_no_op() {
        let mut m = model();
        let before = m.draft.clone();
        m.accept_capture("ctrl+a").unwrap();
        assert_eq!(m.draft, before);
    }

    #[test]
    fn listening_state_is_announced_not_merely_drawn() {
        let mut m = model();
        m.begin_capture(ShortcutField::Switcher);
        let announced = m
            .capture
            .announcement(ShortcutField::Switcher, "win+backtick");
        assert_eq!(announced, LISTENING_ANNOUNCEMENT);

        let idle = CaptureState::Idle.announcement(ShortcutField::Switcher, "win+backtick");
        assert!(
            idle.contains("win+backtick"),
            "current value must be spoken"
        );
    }

    #[test]
    fn every_shortcut_field_round_trips_through_get_and_set() {
        let mut cfg = Config::default();
        for f in ShortcutField::ALL {
            f.set(&mut cfg, "ctrl+win+f9".to_string());
            assert_eq!(
                f.get(&cfg),
                "ctrl+win+f9",
                "{} did not round-trip",
                f.label()
            );
        }
    }

    #[test]
    fn shortcut_field_labels_are_unique() {
        let labels: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.label()).collect();
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate shortcut field label");
            }
        }
    }

    // ── Deterministic focus order ───────────────────────────────────────

    #[test]
    fn focus_order_starts_with_navigation_and_ends_with_actions() {
        for pane in Pane::ALL {
            let order = focus_order(pane);
            assert_eq!(order[0], "General");
            assert_eq!(order[order.len() - 2], "Save");
            assert_eq!(order[order.len() - 1], "Revert");
        }
    }

    #[test]
    fn focus_order_has_no_duplicates() {
        for pane in Pane::ALL {
            let order = focus_order(pane);
            for (i, a) in order.iter().enumerate() {
                for b in order.iter().skip(i + 1) {
                    assert_ne!(a, b, "duplicate focus stop {a} in {pane:?}");
                }
            }
        }
    }

    #[test]
    fn shortcuts_pane_exposes_every_field() {
        let order = focus_order(Pane::Shortcuts);
        for f in ShortcutField::ALL {
            assert!(
                order.contains(&f.label()),
                "{} is unreachable by keyboard",
                f.label()
            );
        }
    }

    #[test]
    fn focus_order_is_stable_across_calls() {
        for pane in Pane::ALL {
            assert_eq!(focus_order(pane), focus_order(pane));
        }
    }

    // ── Onboarding ──────────────────────────────────────────────────────

    #[test]
    fn onboarding_is_absent_unless_requested() {
        assert!(model().onboarding.is_none());
    }

    #[test]
    fn onboarding_starts_at_welcome_and_advances_to_done() {
        let mut m = SettingsModel::new(Config::default(), true);
        assert_eq!(m.onboarding, Some(OnboardingStep::Welcome));
        assert!(!m.advance_onboarding());
        assert_eq!(m.onboarding, Some(OnboardingStep::TrySwitching));
        assert!(m.advance_onboarding());
        assert_eq!(m.onboarding, Some(OnboardingStep::Done));
    }

    #[test]
    fn advancing_past_done_stays_done() {
        let mut m = SettingsModel::new(Config::default(), true);
        m.skip_onboarding();
        assert!(m.advance_onboarding());
        assert_eq!(m.onboarding, Some(OnboardingStep::Done));
    }

    #[test]
    fn skip_reaches_the_same_terminal_state_as_completing() {
        let mut skipped = SettingsModel::new(Config::default(), true);
        skipped.skip_onboarding();

        let mut completed = SettingsModel::new(Config::default(), true);
        while !completed.advance_onboarding() {}

        assert_eq!(skipped.onboarding, completed.onboarding);
    }

    #[test]
    fn onboarding_teaches_the_spatial_philosophy() {
        // Onboarding must explain how this differs from Alt+Tab; without it the
        // feature reads as a broken Alt+Tab.
        let body = OnboardingStep::Welcome.body();
        assert!(body.contains("Alt+Tab"), "the contrast must be explicit");
        assert!(body.contains("application"));
        // PRD §6 also requires explaining that cycling is scoped to the
        // current monitor/virtual desktop, not every one of them.
        assert!(body.contains("monitor"));
        assert!(body.contains("virtual desktop"));
    }

    #[test]
    fn every_onboarding_step_has_heading_and_body() {
        for step in [
            OnboardingStep::Welcome,
            OnboardingStep::TrySwitching,
            OnboardingStep::Done,
        ] {
            assert!(!step.heading().trim().is_empty());
            assert!(!step.body().trim().is_empty());
        }
    }

    // ── Feedback ────────────────────────────────────────────────────────────

    #[test]
    fn validation_messages_name_the_field_and_the_reason() {
        let msg = describe("switcher.shortcut", ShortcutError::MultipleMainKeys);
        assert!(msg.contains("switcher.shortcut"));
        assert!(msg.contains("one main key"));
    }

    #[test]
    fn a_rejected_save_reports_an_error_and_does_not_promote_the_draft() {
        let mut m = model();
        m.draft.switcher.shortcut = "ctrl+win".to_string();
        let mut path = std::env::temp_dir();
        path.push(format!("wiradesk-app-test-{}.toml", std::process::id()));
        m.save(&path);
        assert!(matches!(m.feedback, SaveFeedback::Error(_)));
        assert_eq!(m.saved.switcher.shortcut, "win+backtick");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_successful_save_promotes_the_draft_and_clears_dirty() {
        let mut m = model();
        m.draft.general.auto_start = true;
        let mut path = std::env::temp_dir();
        path.push(format!("wiradesk-app-ok-{}.toml", std::process::id()));
        m.save(&path);
        assert!(matches!(m.feedback, SaveFeedback::Saved { .. }));
        assert!(!m.is_dirty());
        assert!(m.saved.general.auto_start);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn pane_from_label_round_trips_with_label() {
        for pane in Pane::ALL {
            assert_eq!(Pane::from_label(pane.label()), Some(pane));
        }
        assert_eq!(Pane::from_label("not a real pane"), None);
    }

    #[test]
    fn shortcut_field_from_label_round_trips_with_label() {
        for field in ShortcutField::ALL {
            assert_eq!(ShortcutField::from_label(field.label()), Some(field));
        }
        assert_eq!(ShortcutField::from_label("not a real field"), None);
    }

    #[test]
    fn pane_labels_are_unique() {
        let labels: Vec<&str> = Pane::ALL.iter().map(|p| p.label()).collect();
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }
}
