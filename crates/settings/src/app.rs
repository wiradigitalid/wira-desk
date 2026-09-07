#![allow(dead_code)]

//! Settings shell, shortcut capturer, and first-run tutorial
//! (Stories 5.3, 5.4, 5.5).
//! Editing is *staged*: the user edits a draft, and nothing reaches disk until
//! Save validates it. A rejected save leaves both the draft and the on-disk
//! file untouched, so an invalid entry can be corrected instead of losing work.

use shared::constants::{CAPTURE_LEASE_NONE, CAPTURE_LEASE_OBSERVE, CAPTURE_LEASE_RECORD};
use shared::shortcut::Reservation;
use shared::Config;

use crate::persistence::{
    save_and_notify, signal_capture_lease, validate_shortcut, DaemonSignal, SaveOutcome,
    ShortcutError,
};
use crate::theme::{self, ThemeMode, LISTENING_ANNOUNCEMENT, TOGGLE_AUTO_START};

/// Which pane the shell is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    General,
    Shortcuts,
    VmExceptions,
    About,
}

impl Pane {
    pub const ALL: [Pane; 4] = [
        Pane::General,
        Pane::Shortcuts,
        Pane::VmExceptions,
        Pane::About,
    ];

    /// Accessible name, also used as the visible tab label.
    pub fn label(self) -> &'static str {
        match self {
            Pane::General => "General",
            Pane::Shortcuts => "Shortcuts",
            Pane::VmExceptions => "VM & Exceptions",
            Pane::About => "About",
        }
    }

    /// Recover a pane from its position in the declared sequence.
    ///
    /// The navigation index crosses the UI boundary as an `int`, and this is the one place that
    /// reads it back. Out of range falls back to the first pane rather than panicking, for the
    /// same reason [`ShortcutField::from_index`] does: the index arrives from the UI, and a
    /// settings window that closes itself on a stale event is worse than one showing the wrong
    /// pane. Derived from [`Pane::ALL`] so removing a pane cannot leave a hand-numbered table
    /// pointing at the wrong one.
    pub fn from_index(index: i32) -> Pane {
        Pane::ALL
            .get(usize::try_from(index).unwrap_or(0))
            .copied()
            .unwrap_or(Pane::General)
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
/// The editable chord fields, **in the declared sequence**.
///
/// This order is load-bearing three times over: the Shortcuts pane is drawn from it, keyboard
/// focus follows it, and it is the precedence order that decides which of two colliding
/// actions keeps the chord. There is exactly one list of it, and `LBR-ST-14` is the rule that
/// says so. Grouping the rows under headings may gather them; it must not reorder them, which
/// is why the group boundaries are not visible here.
pub enum ShortcutField {
    Switcher,
    Fallback,
    SnapLeft,
    SnapRight,
    SnapTop,
    SnapBottom,
    SnapThirdLeft,
    SnapThirdMiddle,
    SnapThirdRight,
    SnapPercentLeft,
    SnapPercentRight,
    SnapPercentTop,
    SnapPercentBottom,
    SnapMaximize,
    MoveNextMonitor,
    Stack,
}

impl ShortcutField {
    pub const ALL: [ShortcutField; 16] = [
        ShortcutField::Switcher,
        ShortcutField::Fallback,
        ShortcutField::SnapLeft,
        ShortcutField::SnapRight,
        ShortcutField::SnapTop,
        ShortcutField::SnapBottom,
        ShortcutField::SnapThirdLeft,
        ShortcutField::SnapThirdMiddle,
        ShortcutField::SnapThirdRight,
        ShortcutField::SnapPercentLeft,
        ShortcutField::SnapPercentRight,
        ShortcutField::SnapPercentTop,
        ShortcutField::SnapPercentBottom,
        ShortcutField::SnapMaximize,
        ShortcutField::MoveNextMonitor,
        ShortcutField::Stack,
    ];

    /// Recover a field from its position in the declared sequence.
    ///
    /// The UI carries the index rather than a name, and this is the one place that reads it
    /// back. Out of range falls back to the first field rather than panicking: the index
    /// comes across a UI boundary, and a settings window that closes itself on a stale event
    /// is worse than one that acts on the wrong row.
    pub fn from_index(index: i32) -> ShortcutField {
        ShortcutField::ALL
            .get(usize::try_from(index).unwrap_or(0))
            .copied()
            .unwrap_or(ShortcutField::Switcher)
    }

    /// One line under the row title, in the user's terms.
    pub fn description(self) -> &'static str {
        match self {
            ShortcutField::Switcher => "Cycles windows of the active app on this monitor.",
            ShortcutField::Fallback => "Use when another app has taken the Win key.",
            ShortcutField::SnapLeft => "Snaps the window to the left half of this monitor.",
            ShortcutField::SnapRight => "Snaps the window to the right half of this monitor.",
            ShortcutField::SnapTop => "Snaps the window to the top half of this monitor.",
            ShortcutField::SnapBottom => "Snaps the window to the bottom half of this monitor.",
            ShortcutField::SnapMaximize => "Expands the window to fill this monitor.",
            ShortcutField::SnapThirdLeft => "Snaps the window to the left third of this monitor.",
            ShortcutField::SnapThirdMiddle => {
                "Snaps the window to the middle third of this monitor."
            }
            ShortcutField::SnapThirdRight => "Snaps the window to the right third of this monitor.",
            ShortcutField::MoveNextMonitor => "Moves the window to the next monitor, same share.",
            ShortcutField::SnapPercentLeft => {
                "Snaps the window to the left edge at its configured percentage."
            }
            ShortcutField::SnapPercentRight => {
                "Snaps the window to the right edge at its configured percentage."
            }
            ShortcutField::SnapPercentTop => {
                "Snaps the window to the top edge at its configured percentage."
            }
            ShortcutField::SnapPercentBottom => {
                "Snaps the window to the bottom edge at its configured percentage."
            }
            ShortcutField::Stack => "Arranges windows of this app in a clickable stack.",
        }
    }

    /// The five group headings, in the order the pane draws them.
    ///
    /// One home for the heading strings. `group()` returns a member of this array, the pane's
    /// Rust side selects rows by indexing it, and `the_group_headings_have_one_home` proves the
    /// two agree. Before this existed the same five literals were typed out in three places —
    /// here, in `main.rs`'s `group_rows` calls, and in the markup — and a single typo in the
    /// middle one made `group_rows` return an empty model, which draws an empty group with no
    /// error anywhere. That is the same failure `LBR-ST-14` forbids for the actions themselves,
    /// one level up.
    pub const GROUPS: [&'static str; 5] = [
        "Switching",
        "Snap to half",
        "Snap to third",
        "Snap to custom",
        "Resize, move & arrange",
    ];

    /// The heading this field sits under in the Shortcuts pane.
    ///
    /// Presentation only. Grouping gathers rows that are already adjacent in [`ALL`]; it
    /// never reorders them, so the groups fall out of the declared sequence rather than
    /// being a second ordering laid over it.
    pub fn group(self) -> &'static str {
        match self {
            ShortcutField::Switcher | ShortcutField::Fallback => "Switching",
            ShortcutField::SnapLeft
            | ShortcutField::SnapRight
            | ShortcutField::SnapTop
            | ShortcutField::SnapBottom => "Snap to half",
            ShortcutField::SnapThirdLeft
            | ShortcutField::SnapThirdMiddle
            | ShortcutField::SnapThirdRight => "Snap to third",
            ShortcutField::SnapPercentLeft
            | ShortcutField::SnapPercentRight
            | ShortcutField::SnapPercentTop
            | ShortcutField::SnapPercentBottom => "Snap to custom",
            ShortcutField::SnapMaximize | ShortcutField::MoveNextMonitor | ShortcutField::Stack => {
                "Resize, move & arrange"
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ShortcutField::Switcher => "Switch windows of the same application",
            ShortcutField::Fallback => "Fallback switch shortcut",
            ShortcutField::SnapLeft => "Snap to left half",
            ShortcutField::SnapRight => "Snap to right half",
            ShortcutField::SnapTop => "Snap to top half",
            ShortcutField::SnapBottom => "Snap to bottom half",
            ShortcutField::SnapMaximize => "Maximize",
            ShortcutField::SnapThirdLeft => "Snap to left third",
            ShortcutField::SnapThirdMiddle => "Snap to middle third",
            ShortcutField::SnapThirdRight => "Snap to right third",
            ShortcutField::MoveNextMonitor => "Move to next monitor",
            ShortcutField::SnapPercentLeft => "Snap to left edge",
            ShortcutField::SnapPercentRight => "Snap to right edge",
            ShortcutField::SnapPercentTop => "Snap to top edge",
            ShortcutField::SnapPercentBottom => "Snap to bottom edge",
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
            ShortcutField::SnapTop => &cfg.snapping.snap_half_top,
            ShortcutField::SnapBottom => &cfg.snapping.snap_half_bottom,
            ShortcutField::SnapMaximize => &cfg.snapping.snap_maximize,
            ShortcutField::SnapThirdLeft => &cfg.snapping.snap_third_left,
            ShortcutField::SnapThirdMiddle => &cfg.snapping.snap_third_middle,
            ShortcutField::SnapThirdRight => &cfg.snapping.snap_third_right,
            ShortcutField::MoveNextMonitor => &cfg.layout.move_next_monitor_shortcut,
            ShortcutField::SnapPercentLeft => &cfg.snapping.snap_percent_left,
            ShortcutField::SnapPercentRight => &cfg.snapping.snap_percent_right,
            ShortcutField::SnapPercentTop => &cfg.snapping.snap_percent_top,
            ShortcutField::SnapPercentBottom => &cfg.snapping.snap_percent_bottom,
            ShortcutField::Stack => &cfg.layout.stack_shortcut,
        }
    }

    pub fn set(self, cfg: &mut Config, value: String) {
        match self {
            ShortcutField::Switcher => cfg.switcher.shortcut = value,
            ShortcutField::Fallback => cfg.switcher.fallback_shortcut = value,
            ShortcutField::SnapLeft => cfg.snapping.snap_half_left = value,
            ShortcutField::SnapRight => cfg.snapping.snap_half_right = value,
            ShortcutField::SnapTop => cfg.snapping.snap_half_top = value,
            ShortcutField::SnapBottom => cfg.snapping.snap_half_bottom = value,
            ShortcutField::SnapMaximize => cfg.snapping.snap_maximize = value,
            ShortcutField::SnapThirdLeft => cfg.snapping.snap_third_left = value,
            ShortcutField::SnapThirdMiddle => cfg.snapping.snap_third_middle = value,
            ShortcutField::SnapThirdRight => cfg.snapping.snap_third_right = value,
            ShortcutField::MoveNextMonitor => cfg.layout.move_next_monitor_shortcut = value,
            ShortcutField::SnapPercentLeft => cfg.snapping.snap_percent_left = value,
            ShortcutField::SnapPercentRight => cfg.snapping.snap_percent_right = value,
            ShortcutField::SnapPercentTop => cfg.snapping.snap_percent_top = value,
            ShortcutField::SnapPercentBottom => cfg.snapping.snap_percent_bottom = value,
            ShortcutField::Stack => cfg.layout.stack_shortcut = value,
        }
    }

    pub fn is_enabled(self, cfg: &Config) -> bool {
        match self {
            ShortcutField::Switcher => cfg.switcher.shortcut_enabled,
            ShortcutField::Fallback => cfg.switcher.fallback_shortcut_enabled,
            ShortcutField::SnapLeft => cfg.snapping.snap_half_left_enabled,
            ShortcutField::SnapRight => cfg.snapping.snap_half_right_enabled,
            ShortcutField::SnapTop => cfg.snapping.snap_half_top_enabled,
            ShortcutField::SnapBottom => cfg.snapping.snap_half_bottom_enabled,
            ShortcutField::SnapMaximize => cfg.snapping.snap_maximize_enabled,
            ShortcutField::SnapThirdLeft => cfg.snapping.snap_third_left_enabled,
            ShortcutField::SnapThirdMiddle => cfg.snapping.snap_third_middle_enabled,
            ShortcutField::SnapThirdRight => cfg.snapping.snap_third_right_enabled,
            ShortcutField::MoveNextMonitor => cfg.layout.move_next_monitor_shortcut_enabled,
            ShortcutField::SnapPercentLeft => cfg.snapping.snap_percent_left_enabled,
            ShortcutField::SnapPercentRight => cfg.snapping.snap_percent_right_enabled,
            ShortcutField::SnapPercentTop => cfg.snapping.snap_percent_top_enabled,
            ShortcutField::SnapPercentBottom => cfg.snapping.snap_percent_bottom_enabled,
            ShortcutField::Stack => cfg.layout.stack_shortcut_enabled,
        }
    }

    pub fn set_enabled(self, cfg: &mut Config, enabled: bool) {
        match self {
            ShortcutField::Switcher => cfg.switcher.shortcut_enabled = enabled,
            ShortcutField::Fallback => cfg.switcher.fallback_shortcut_enabled = enabled,
            ShortcutField::SnapLeft => cfg.snapping.snap_half_left_enabled = enabled,
            ShortcutField::SnapRight => cfg.snapping.snap_half_right_enabled = enabled,
            ShortcutField::SnapTop => cfg.snapping.snap_half_top_enabled = enabled,
            ShortcutField::SnapBottom => cfg.snapping.snap_half_bottom_enabled = enabled,
            ShortcutField::SnapMaximize => cfg.snapping.snap_maximize_enabled = enabled,
            ShortcutField::SnapThirdLeft => cfg.snapping.snap_third_left_enabled = enabled,
            ShortcutField::SnapThirdMiddle => cfg.snapping.snap_third_middle_enabled = enabled,
            ShortcutField::SnapThirdRight => cfg.snapping.snap_third_right_enabled = enabled,
            ShortcutField::MoveNextMonitor => {
                cfg.layout.move_next_monitor_shortcut_enabled = enabled
            }
            ShortcutField::SnapPercentLeft => cfg.snapping.snap_percent_left_enabled = enabled,
            ShortcutField::SnapPercentRight => cfg.snapping.snap_percent_right_enabled = enabled,
            ShortcutField::SnapPercentTop => cfg.snapping.snap_percent_top_enabled = enabled,
            ShortcutField::SnapPercentBottom => cfg.snapping.snap_percent_bottom_enabled = enabled,
            ShortcutField::Stack => cfg.layout.stack_shortcut_enabled = enabled,
        }
    }

    pub fn has_percent(self) -> bool {
        matches!(
            self,
            ShortcutField::SnapPercentLeft
                | ShortcutField::SnapPercentRight
                | ShortcutField::SnapPercentTop
                | ShortcutField::SnapPercentBottom
                | ShortcutField::Stack
        )
    }

    /// The inclusive range a percentage row accepts, or `None` for a row with no percentage.
    ///
    /// One home for these numbers, in Rust rather than as markup literals, because markup cannot
    /// be asserted on and this is where the range can be guarded. `Stack` is the odd one: the
    /// overlapping-stack width has always accepted 10-100, while a snap edge accepts 1-99. Both
    /// pairs used to be hardcoded in different files — `shortcut_row.slint`'s stepper carried
    /// `1`/`99` and the retired Layout pane carried `10`/`100` — so moving the stack control onto
    /// a shortcut row without moving its range with it silently narrowed it.
    pub fn percent_bounds(self) -> Option<(u32, u32)> {
        use shared::constants::{
            MAX_SNAP_PERCENT, MAX_STACK_WIDTH_PERCENT, MIN_SNAP_PERCENT, MIN_STACK_WIDTH_PERCENT,
        };
        match self {
            ShortcutField::SnapPercentLeft
            | ShortcutField::SnapPercentRight
            | ShortcutField::SnapPercentTop
            | ShortcutField::SnapPercentBottom => Some((MIN_SNAP_PERCENT, MAX_SNAP_PERCENT)),
            ShortcutField::Stack => Some((MIN_STACK_WIDTH_PERCENT, MAX_STACK_WIDTH_PERCENT)),
            _ => None,
        }
    }

    pub fn percent(self, cfg: &Config) -> Option<u32> {
        match self {
            ShortcutField::SnapPercentLeft => Some(cfg.snapping.percent_left),
            ShortcutField::SnapPercentRight => Some(cfg.snapping.percent_right),
            ShortcutField::SnapPercentTop => Some(cfg.snapping.percent_top),
            ShortcutField::SnapPercentBottom => Some(cfg.snapping.percent_bottom),
            ShortcutField::Stack => Some(cfg.layout.stack_width_percent),
            _ => None,
        }
    }

    pub fn set_percent(self, cfg: &mut Config, value: u32) {
        match self {
            ShortcutField::SnapPercentLeft => cfg.snapping.percent_left = value,
            ShortcutField::SnapPercentRight => cfg.snapping.percent_right = value,
            ShortcutField::SnapPercentTop => cfg.snapping.percent_top = value,
            ShortcutField::SnapPercentBottom => cfg.snapping.percent_bottom = value,
            ShortcutField::Stack => cfg.layout.stack_width_percent = value,
            _ => {}
        }
    }

    /// The dotted TOML path this field corresponds to in `Config`.
    ///
    /// `persistence::validate_config` names a rejected field by this exact path, and `from_key`
    /// is the one place that reads a save-time rejection back into a field. The strings are
    /// still literals here rather than shared with that module, because persistence must not
    /// depend on this UI-facing enum (`LBR-ST-14`, `DEC-018`).
    ///
    /// What is **not** duplicated any more is the *order*. `persistence::validate_config` walks
    /// `shared::constants::SHORTCUT_DECLARED_ORDER` and looks each key up, so there is no second
    /// ordered table to keep in step — re-adding one is the drift `DEC-018` was written to end.
    /// A divergence is no longer silent either: `the_declared_sequence_matches_the_shared_source`
    /// catches a `key()` that drifts from the constant, and `validate_config`'s own lookup fails
    /// loudly on a key it does not know.
    pub fn key(self) -> &'static str {
        match self {
            ShortcutField::Switcher => "switcher.shortcut",
            ShortcutField::Fallback => "switcher.fallback_shortcut",
            ShortcutField::SnapLeft => "snapping.snap_half_left",
            ShortcutField::SnapRight => "snapping.snap_half_right",
            ShortcutField::SnapTop => "snapping.snap_half_top",
            ShortcutField::SnapBottom => "snapping.snap_half_bottom",
            ShortcutField::SnapMaximize => "snapping.snap_maximize",
            ShortcutField::SnapThirdLeft => "snapping.snap_third_left",
            ShortcutField::SnapThirdMiddle => "snapping.snap_third_middle",
            ShortcutField::SnapThirdRight => "snapping.snap_third_right",
            ShortcutField::MoveNextMonitor => "layout.move_next_monitor_shortcut",
            ShortcutField::SnapPercentLeft => "snapping.snap_percent_left",
            ShortcutField::SnapPercentRight => "snapping.snap_percent_right",
            ShortcutField::SnapPercentTop => "snapping.snap_percent_top",
            ShortcutField::SnapPercentBottom => "snapping.snap_percent_bottom",
            ShortcutField::Stack => "layout.stack_shortcut",
        }
    }

    /// Reverse lookup of [`ShortcutField::key`].
    pub fn from_key(key: &str) -> Option<ShortcutField> {
        ShortcutField::ALL.into_iter().find(|f| f.key() == key)
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
                "Wira Desk switches instantly between windows of the application you are currently using, \
                 instead of cycling through every open window like Alt+Tab. Window focus stays strictly \
                 on your active physical monitor and virtual desktop, eliminating multi-monitor distractions."
            }
            OnboardingStep::TrySwitching => {
                "Practice switching focus between two windows of the same app. Press Win + ` (backtick) \
                 on your keyboard or click the practice button below to watch focus shift with zero HUD delay."
            }
            OnboardingStep::Done => {
                "Wira Desk is now resident and active in your System Tray. You can customize shortcuts, \
                 snapping parameters, and VM passthrough rules anytime from Settings."
            }
        }
    }
}

/// Whether a lease post is owed, given what is wanted and what became of the last one.
///
/// Extracted from `sync_capture_lease` so the rule is reachable from a test. The live path
/// needs a running daemon and an integrity boundary between two processes, and a unit test
/// has neither — which is precisely why the defect this encodes survived: the only way to
/// meet it was to run the product from the wrong place on the right machine.
fn lease_post_owed(desired: usize, sent: usize, last: DaemonSignal) -> bool {
    // The second clause is the fix. Without it a post that never arrived is remembered as
    // one that did, no further post is ever made, and the mistake lasts as long as the
    // window does.
    desired != sent || !last.delivered()
}

/// Result of a Save attempt, in a form the UI can render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFeedback {
    None,
    Saved { reload: DaemonSignal },
    Error(String),
}

/// Render a raw `+`-joined shortcut string (`"ctrl+alt+down"`) the way every
/// shortcut display in this UI must look (`"Ctrl + Alt + ↓"`). The single
/// source of that mapping — a display built any other way (as
/// `KeyCheckState::tick` once did, working straight from
/// `shared::shortcut::name_from_vk`) drifts from this one silently, which is
/// exactly the bug where the Key check readout showed the word `"down"`
/// while every other row in the pane showed `↓`.
pub fn format_shortcut_display(raw: &str) -> String {
    if raw.is_empty() {
        return "None".to_string();
    }
    raw.split('+')
        .map(|token| {
            let lower = token.to_lowercase();
            match lower.trim() {
                "win" => "Win".to_string(),
                "ctrl" => "Ctrl".to_string(),
                "alt" => "Alt".to_string(),
                "shift" => "Shift".to_string(),
                "backtick" => "`".to_string(),
                "enter" => "Enter".to_string(),
                "tab" => "Tab".to_string(),
                "space" => "Space".to_string(),
                "escape" => "Esc".to_string(),
                "left" => "←".to_string(),
                "right" => "→".to_string(),
                "up" => "↑".to_string(),
                "down" => "↓".to_string(),
                "f1" => "F1".to_string(),
                "f2" => "F2".to_string(),
                "f3" => "F3".to_string(),
                "f4" => "F4".to_string(),
                "f5" => "F5".to_string(),
                "f6" => "F6".to_string(),
                "f7" => "F7".to_string(),
                "f8" => "F8".to_string(),
                "f9" => "F9".to_string(),
                "f10" => "F10".to_string(),
                "f11" => "F11".to_string(),
                "f12" => "F12".to_string(),
                _ => token.trim().to_uppercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Human-readable message for a validation failure.
/// `field` arrives two ways: a dotted config key, from `SaveOutcome::Rejected`
/// on the save path, or an already-resolved label, from the inline capture
/// path. `ShortcutField::from_key` resolves the former to a label and leaves
/// the latter untouched, so this message never surfaces a TOML path to the
/// user — the confusion `DEC-001` and `LBR-ST-8` require it not to (a user
/// should not have to know `switcher.fallback_shortcut` is what they see
/// labelled "Fallback switch shortcut").
pub fn describe(field: &str, err: ShortcutError) -> String {
    let label = ShortcutField::from_key(field)
        .map(ShortcutField::label)
        .unwrap_or(field);
    match err {
        ShortcutError::UnsupportedToken => {
            format!("{label} contains a key name Wira Desk does not recognize.")
        }
        ShortcutError::NoMainKey => format!("{label} needs a main key in addition to modifiers."),
        ShortcutError::MultipleMainKeys => format!("{label} may only contain one main key."),
        ShortcutError::NoModifier => {
            format!("{label} needs at least one modifier (ctrl, win, alt, or shift).")
        }
        ShortcutError::Unrepresentable => {
            format!("{label} cannot be saved in a supported form.")
        }
        ShortcutError::Reserved(info) => match info.kind {
            Reservation::Immutable => {
                format!(
                    "Windows uses this chord to {}. This one cannot be changed by any app.",
                    info.owner
                )
            }
            Reservation::ShellOwned => {
                format!(
                    "Windows uses this chord to {}. Try adding Ctrl (e.g. Ctrl + Win + key).",
                    info.owner
                )
            }
        },
        ShortcutError::DuplicateShortcut(other) => {
            let other_label = ShortcutField::from_key(other)
                .map(ShortcutField::label)
                .unwrap_or(other);
            format!(
                "{label} conflicts with {other_label}. Each action must have a unique shortcut."
            )
        }
        ShortcutError::InvalidPercentage(_) => {
            if field == "layout.stack_width_percent" {
                format!(
                    "Stack width percentage must be between {}% and {}%.",
                    shared::constants::MIN_STACK_WIDTH_PERCENT,
                    shared::constants::MAX_STACK_WIDTH_PERCENT
                )
            } else {
                let name = match field {
                    "snapping.percent_left" => "Left edge snap percentage",
                    "snapping.percent_right" => "Right edge snap percentage",
                    "snapping.percent_top" => "Top edge snap percentage",
                    "snapping.percent_bottom" => "Bottom edge snap percentage",
                    _ => label,
                };
                format!(
                    "{name} must be between {}% and {}%.",
                    shared::constants::MIN_SNAP_PERCENT,
                    shared::constants::MAX_SNAP_PERCENT
                )
            }
        }
    }
}

/// Verdict reported by KeyCheck diagnostic observer.
/// Evaluates delivery reachability honestly without predicting or probing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyCheckVerdict {
    /// Ready / idle
    #[default]
    Idle = 0,
    /// Keystroke arrived and reached window
    Received = 1,
    /// Claimed by other app via RegisterHotKey, but Wira Desk hook receives it first
    ClaimedByOtherApp = 2,
    /// Intercepted by external LL hook / swallowed, chord dead
    Intercepted = 3,
    /// Daemon is not running (window-only checking)
    DaemonNotRunning = 4,
}

/// A chord the daemon's hook reported observing (`WM_APP_RECORDED_CHORD`),
/// still waiting on a matching window key event so the correlation `DEC-005`
/// is built on can be drawn. Cleared either by a matching `record_key`
/// (row 1: hook yes, window yes) or by `tick` running out the grace period
/// with no match (row 2: hook yes, window no — another application claimed
/// the chord ahead of the window, but the hook still saw it first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingHookReport {
    vk: u16,
    ctrl: bool,
    win: bool,
    alt: bool,
    shift: bool,
    ticks_left: u8,
}

/// How long a hook report waits for a matching window event before the
/// correlation gives up and reports it as claimed by another application.
/// Correlation is deterministic — the hook always precedes window delivery —
/// so this is a grace period for cross-process message delivery, not a race
/// window: `DEC-005` explicitly rules out timing or debouncing to decide
/// which row applies, only to bound how long row 1 gets to arrive before row
/// 2 is reported instead.
const HOOK_REPORT_GRACE_TICKS: u8 = 4;

/// Pure observer state for the KeyCheck live diagnostic instrument.
/// DEC-005: purely observes what reaches the window without interfering
/// with draft edits, dirty state, or Save validation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyCheckState {
    pub mod_ctrl: bool,
    pub mod_win: bool,
    pub mod_alt: bool,
    pub mod_shift: bool,
    pub last_display: String,
    pub last_canonical: String,
    pub verdict: KeyCheckVerdict,
    pub beat: bool,
    pending_hook: Option<PendingHookReport>,
}

impl KeyCheckState {
    /// Update modifier state from physical keyboard events.
    pub fn update_modifiers(&mut self, ctrl: bool, win: bool, alt: bool, shift: bool) {
        self.mod_ctrl = ctrl;
        self.mod_win = win;
        self.mod_alt = alt;
        self.mod_shift = shift;
    }

    /// The daemon's hook reported observing this chord (`DEC-004`'s observe
    /// or record lease). Starts the grace period a matching window event may
    /// still arrive within; does not by itself produce a verdict, since
    /// `DEC-005` reports observations correlated against the window, never a
    /// single signal alone.
    pub fn record_hook_report(&mut self, vk: u16, ctrl: bool, win: bool, alt: bool, shift: bool) {
        self.pending_hook = Some(PendingHookReport {
            vk,
            ctrl,
            win,
            alt,
            shift,
            ticks_left: HOOK_REPORT_GRACE_TICKS,
        });
    }

    /// One diagnostic tick (driven by a UI timer, not real time), independent
    /// of whether a hook report is pending. When a pending report's grace
    /// period runs out with no matching window event, that is row 2 of
    /// `DEC-005`'s table: the hook saw the chord, the window never did —
    /// another application claimed it through `RegisterHotKey`, ahead of the
    /// window but behind Wira Desk's hook.
    pub fn tick(&mut self) {
        let Some(pending) = &mut self.pending_hook else {
            return;
        };
        pending.ticks_left = pending.ticks_left.saturating_sub(1);
        if pending.ticks_left == 0 {
            let mut parts = Vec::new();
            if pending.ctrl {
                parts.push("ctrl");
            }
            if pending.win {
                parts.push("win");
            }
            if pending.alt {
                parts.push("alt");
            }
            if pending.shift {
                parts.push("shift");
            }
            // Routed through the same `format_shortcut_display` every other
            // shortcut readout in this UI uses, so `down`/`backtick`/etc.
            // render as `↓`/`` ` ``/etc. here too, never as the raw
            // `shared::shortcut::name_from_vk` token.
            self.last_display = match shared::shortcut::name_from_vk(pending.vk) {
                Some(name) => {
                    parts.push(name.as_str());
                    format_shortcut_display(&parts.join("+"))
                }
                None => {
                    let mods = format_shortcut_display(&parts.join("+"));
                    format!("{mods} + (code 0x{:02X})", pending.vk)
                }
            };
            self.last_canonical.clear();
            self.verdict = KeyCheckVerdict::ClaimedByOtherApp;
            self.beat = true;
            self.pending_hook = None;
        }
    }

    /// Record a keystroke event that arrived at the window. `vk` is the raw
    /// key the same physical press would report as, when it can be resolved
    /// (`shared::shortcut::vk_from_name`) — used only to correlate against a
    /// pending hook report, never stored.
    pub fn record_key(
        &mut self,
        display: &str,
        canonical: &str,
        daemon_running: bool,
        mods: (bool, bool, bool, bool),
        vk: Option<u16>,
    ) {
        self.last_display = display.to_string();
        self.last_canonical = canonical.to_string();

        let matched = matches!(
            (self.pending_hook, vk),
            (Some(p), Some(v)) if p.vk == v && (p.ctrl, p.win, p.alt, p.shift) == mods
        );
        if matched {
            self.pending_hook = None;
        }

        self.verdict = if matched {
            // Row 1: hook yes, window yes — nothing intercepts this chord.
            KeyCheckVerdict::Received
        } else if !daemon_running {
            // Row 4: the daemon is not running.
            KeyCheckVerdict::DaemonNotRunning
        } else {
            // Row 3: the daemon is running but no hook report correlates
            // with this window event — an earlier third-party LL hook
            // swallowed the chord before Wira Desk's own hook saw it.
            // DEF-4: this used to reuse `DaemonNotRunning`, telling a user
            // with a healthy daemon to restart the one thing that was never
            // the problem.
            KeyCheckVerdict::Intercepted
        };
        self.beat = true;
    }

    /// Complete a beat pulse.
    pub fn clear_beat(&mut self) {
        self.beat = false;
    }
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
    /// Which simulated dummy window has focus in Step 2 of Onboarding (0 or 1).
    pub onboarding_focus_index: usize,
    /// Whether simulated cycling has been triggered at least once in Step 2.
    pub onboarding_simulated_success: bool,
    /// The one line the About pane shows about updates. Written for every outcome by
    /// `update::describe`, because "update failed" tells a user nothing and the difference
    /// between "you are offline" and "the file offered was not ours" is the difference
    /// between waiting and worrying.
    pub update_status: String,
    /// A check or a download is in flight. Disables both buttons, so a second click cannot
    /// start a second worker against the same staging directory.
    pub update_busy: bool,
    /// A validated newer release, once a check has found one. Its presence is what swaps the
    /// button from "Check for updates" to "Download and install".
    pub update_available: Option<shared::update::Release>,
    /// The tutorial's start-at-sign-in answer, pre-checked, and **held apart from
    /// `draft.general.auto_start` on purpose.**
    ///
    /// The question is asked on the last screen. Skip lives on the first one, so a user who
    /// skips never sees it — and if the answer went straight into the draft, skipping would
    /// register an elevated logon task for someone who was never asked. Keeping it separate
    /// means only [`SettingsModel::accept_onboarding_auto_start`] can commit it, and only
    /// finishing calls that.
    pub onboarding_auto_start: bool,
    /// Pure observer state for KeyCheck live keyboard responsiveness instrument.
    pub key_check: KeyCheckState,
    /// The field the most recent successful capture overwrote, and the chord
    /// it held immediately before that. This is the only record of a
    /// displaced chord that exists, and it is what lets `swap_shortcuts` hand
    /// it back to whichever action lost it — without it, "swap" would have
    /// nothing to swap the new chord's collision partner *back to*, since the
    /// two fields already hold the same value the instant a collision exists.
    pub last_capture: Option<(ShortcutField, String)>,
    /// The lease level last actually signalled to the daemon, so
    /// `sync_capture_lease` posts only on a change (DEC-004's "one owning
    /// place per lease" — never one ad hoc `signal_capture_lease` call per
    /// call site).
    sent_lease_level: usize,
    /// What became of the last lease post. Kept because "we sent it" and "the daemon has
    /// it" are different facts, and treating them as one made a refusal permanent.
    lease_signal: DaemonSignal,
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
            update_status: String::new(),
            update_busy: false,
            update_available: None,
            onboarding_focus_index: 0,
            onboarding_simulated_success: false,
            // Pre-checked. A tray utility that does not come back after a restart is a
            // utility the user has to remember to start, which is the thing it exists to
            // save them from. Asking with the box already ticked is the honest middle:
            // visible, one click to decline, and never decided in silence.
            onboarding_auto_start: true,
            key_check: KeyCheckState::default(),
            last_capture: None,
            sent_lease_level: CAPTURE_LEASE_NONE,
            lease_signal: DaemonSignal::Delivered,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.draft != self.saved
    }

    /// What the capture lease should be right now, purely from `(pane,
    /// capture)`. Record implies observe; observe never implies record —
    /// `DEC-004`'s table, expressed as one derivation rather than four
    /// separately-armed call sites that could drift out of sync with each
    /// other.
    fn desired_lease_level(&self) -> usize {
        match &self.capture {
            CaptureState::Listening(_) => CAPTURE_LEASE_RECORD,
            CaptureState::Idle if self.pane == Pane::Shortcuts => CAPTURE_LEASE_OBSERVE,
            CaptureState::Idle => CAPTURE_LEASE_NONE,
        }
    }

    /// The one owning place that arms or disarms the capture lease. Called
    /// after every state change that can affect `(pane, capture)`, so a lease
    /// is never left mismatched with what the UI is actually doing — the
    /// live defect this replaces was `set_pane` cancelling a capture without
    /// disarming anything. Posts to the daemon only when the desired level
    /// actually changes, which is also what makes leaving the Shortcuts pane
    /// or losing foreground fail closed rather than leaving a stale lease
    /// armed forever: the daemon's own foreground check and heartbeat reaping
    /// cover the case this call is never reached at all (a crash, a killed
    /// process).
    pub fn sync_capture_lease(&mut self) {
        let desired = self.desired_lease_level();
        // Retry when the last attempt did not land, not only when the level changed.
        //
        // Posting only on a change is right while posts arrive. It is wrong the moment one
        // does not: the next sync sees no change, sends nothing, and Settings goes on
        // believing a lease is armed that the daemon never heard of. That is exactly what a
        // Settings started outside the tray did — every post refused by UIPI, the first
        // refusal remembered as a success, and shortcut recording quietly receiving
        // nothing for the life of the window.
        if lease_post_owed(desired, self.sent_lease_level, self.lease_signal) {
            self.lease_signal = signal_capture_lease(desired);
            self.sent_lease_level = desired;
        }
    }

    /// Whether the daemon is running and refusing this process's messages — the shape a
    /// non-elevated Settings takes against an elevated daemon. Distinct from the daemon
    /// being absent, which is ordinary and needs no warning.
    pub fn lease_refused(&self) -> bool {
        self.lease_signal == DaemonSignal::Refused
    }

    pub fn begin_capture(&mut self, field: ShortcutField) {
        self.capture = CaptureState::Listening(field);
        self.sync_capture_lease();
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
        self.sync_capture_lease();
    }

    /// Cancel capture without changing anything — the Escape affordance.
    pub fn cancel_capture(&mut self) {
        self.capture = CaptureState::Idle;
        self.sync_capture_lease();
    }

    /// Accept a captured combination.
    /// Validation happens here, before the draft changes, so an unusable
    /// combination never becomes the displayed value. The chord `field` held
    /// before this overwrite is recorded as `last_capture` — a collision this
    /// capture creates is only resolvable through `swap_shortcuts` while that
    /// record still names this same field.
    pub fn accept_capture(&mut self, combination: &str) -> Result<(), ShortcutError> {
        let CaptureState::Listening(field) = self.capture else {
            return Ok(());
        };
        let canonical = validate_shortcut(combination)?;
        let previous = field.get(&self.draft).to_string();
        field.set(&mut self.draft, canonical);
        self.last_capture = Some((field, previous));
        self.capture = CaptureState::Idle;
        self.sync_capture_lease();
        // A capture can create or resolve a collision; either way the status
        // bar's last word on the draft (a stale "Settings saved" banner, or a
        // stale error) is no longer the truth about it.
        self.feedback = SaveFeedback::None;
        Ok(())
    }

    /// Set a numeric snap percentage on the draft.
    pub fn set_percent(&mut self, field: ShortcutField, value: u32) {
        field.set_percent(&mut self.draft, value);
        self.feedback = SaveFeedback::None;
    }

    /// Set the enabled state of a shortcut action on the draft.
    pub fn set_action_enabled(&mut self, field: ShortcutField, enabled: bool) {
        field.set_enabled(&mut self.draft, enabled);
        self.feedback = SaveFeedback::None;
    }

    /// Check if a field currently conflicts with any other field in the draft.
    /// Returns the conflicting field if any.
    pub fn find_conflict(&self, field: ShortcutField) -> Option<ShortcutField> {
        if !field.is_enabled(&self.draft) {
            return None;
        }
        let val = field.get(&self.draft);
        ShortcutField::ALL.into_iter().find(|&other| {
            other != field && other.is_enabled(&self.draft) && other.get(&self.draft) == val
        })
    }

    /// Check if any shortcut conflict exists across the draft.
    pub fn has_any_conflict(&self) -> bool {
        ShortcutField::ALL
            .into_iter()
            .any(|field| self.find_conflict(field).is_some())
    }

    /// Whether at least one standing conflict can be resolved with
    /// `swap_shortcuts` right now. The status bar uses this to decide whether
    /// it may tell the user Swap is an option — a conflict from a hand-edited
    /// `config.toml`, or one whose triggering capture has since been
    /// superseded, has no displaced chord on record and cannot be swapped.
    pub fn any_swappable_conflict(&self) -> bool {
        ShortcutField::ALL
            .into_iter()
            .any(|field| self.find_conflict(field).is_some() && self.can_swap(field))
    }

    /// Whether `field` is the field the most recent capture wrote into.
    /// This is what the pane checks before offering `swap_shortcuts` on a
    /// conflicted row: the *other* party in the collision never lost
    /// anything, so there is nothing on record to give it back, and
    /// exchanging its current value with `field`'s would exchange two now
    /// identical strings.
    pub fn can_swap(&self, field: ShortcutField) -> bool {
        matches!(&self.last_capture, Some((last_field, _)) if *last_field == field)
    }

    /// Resolve a collision by giving `conf_field` the chord `field`'s last
    /// capture displaced, restoring what the two actions held before that
    /// capture collided them. Requires `field` to be the field `can_swap`
    /// reports true for; called with any other field this is a no-op, since
    /// no displaced chord is on record for it and there is nothing to give
    /// back. Does not check whether the returned chord collides with a third
    /// field — the pane re-evaluates conflicts on the next frame regardless.
    pub fn swap_shortcuts(&mut self, field: ShortcutField, conf_field: ShortcutField) {
        let Some((last_field, previous)) = self.last_capture.take() else {
            return;
        };
        if last_field != field {
            self.last_capture = Some((last_field, previous));
            return;
        }
        conf_field.set(&mut self.draft, previous);
        self.feedback = SaveFeedback::None;
    }

    /// Discard edits.
    pub fn revert(&mut self) {
        self.draft = self.saved.clone();
        self.capture = CaptureState::Idle;
        self.sync_capture_lease();
        self.feedback = SaveFeedback::None;
        // The reverted draft no longer carries whatever this capture wrote,
        // so the chord it once displaced is no longer displaced by anything.
        self.last_capture = None;
    }

    /// Validate, persist, and signal reload.
    pub fn save(&mut self, path: &std::path::Path) {
        match save_and_notify(&self.draft, path) {
            SaveOutcome::Saved { reload } => {
                self.saved = self.draft.clone();
                self.feedback = SaveFeedback::Saved { reload };
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
    ///
    /// `Some(Done)` is deliberately terminal and idempotent, and
    /// `skip_reaches_the_same_terminal_state_as_completing` depends on that. Reaching the
    /// end is not the same event as leaving the screen — see [`Self::dismiss_onboarding`].
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

    /// Leave the tutorial view, so the settings panes become reachable.
    ///
    /// This exists because the two facts were conflated. The model uses `Some(Done)` as
    /// "the tutorial ran to its end", which `skip` also produces; the view asks
    /// `onboarding.is_some()` to decide whether to *draw* the tutorial. With no way to say
    /// "finished and dismissed", the final screen's button called `advance_onboarding`,
    /// which correctly reported success and correctly changed nothing — leaving the user on
    /// the last screen with no route into Settings, and the titlebar close (wired to skip,
    /// which hides the window) the only way out.
    ///
    /// Deliberately separate from `advance_onboarding` so that reaching the end still means
    /// what every existing test says it means.
    pub fn dismiss_onboarding(&mut self) {
        self.onboarding = None;
    }

    /// Commit the tutorial's start-at-sign-in answer into the draft.
    ///
    /// Called only when the tutorial is *finished*, never when it is skipped. Skipping
    /// happens on the first screen, before the question has been shown, and a skipped
    /// question is not a yes — registering a task that runs elevated at every sign-in
    /// because someone dismissed a tutorial would be exactly the kind of quiet system
    /// change this product refuses to make.
    pub fn accept_onboarding_auto_start(&mut self) {
        self.draft.general.auto_start = self.onboarding_auto_start;
    }

    /// Simulate toggling dummy window focus in Onboarding Step 2.
    pub fn toggle_onboarding_simulation(&mut self) {
        self.onboarding_focus_index = if self.onboarding_focus_index == 0 {
            1
        } else {
            0
        };
        self.onboarding_simulated_success = true;
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
        Pane::VmExceptions => {
            order.push(theme::VM_BYPASS_PROCESS_LIST.name);
            order.push(theme::VM_BYPASS_CLASS_LIST.name);
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
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("wiradesk-app-test-{}", std::process::id()));
        p
    }

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
        m.set_pane(Pane::VmExceptions);
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
    fn field_declaration_order_is_the_precedence_order() {
        // The discriminant is what crosses the UI boundary as `listening_field` and comes
        // back through `from_index`, and it is also the position that decides which of two
        // colliding actions keeps the chord. Those are only the same thing while the
        // discriminants match `ALL`'s order, so that identity is asserted rather than assumed.
        for (i, f) in ShortcutField::ALL.into_iter().enumerate() {
            assert_eq!(f as usize, i, "{} sits out of declared order", f.label());
            assert_eq!(
                ShortcutField::from_index(i as i32),
                f,
                "round trip through the UI index must return the same field"
            );
        }
    }

    #[test]
    fn field_declaration_order_places_percent_snap_ahead_of_stack() {
        assert!((ShortcutField::SnapPercentLeft as usize) < (ShortcutField::Stack as usize));
        assert!((ShortcutField::SnapPercentRight as usize) < (ShortcutField::Stack as usize));
        assert!((ShortcutField::SnapPercentTop as usize) < (ShortcutField::Stack as usize));
        assert!((ShortcutField::SnapPercentBottom as usize) < (ShortcutField::Stack as usize));
    }

    #[test]
    fn field_declaration_order_includes_third_snap_fields() {
        // `DEC-014` moves the thirds up behind the halves; Maximize's relative position moved
        // out to `snap_maximize_now_sorts_after_every_snap_variant`, which asserts the new
        // ordering rather than the inverted old one.
        assert_eq!(ShortcutField::SnapThirdLeft as usize, 6);
        assert_eq!(ShortcutField::SnapThirdMiddle as usize, 7);
        assert_eq!(ShortcutField::SnapThirdRight as usize, 8);
        assert!(
            (ShortcutField::SnapBottom as usize) < (ShortcutField::SnapThirdLeft as usize),
            "the halves stay ahead of the thirds"
        );
        assert!(
            (ShortcutField::SnapThirdRight as usize) < (ShortcutField::SnapPercentLeft as usize),
            "the thirds stay ahead of the custom-percentage edges"
        );
    }

    #[test]
    fn an_out_of_range_index_does_not_panic() {
        // The index arrives from the UI. A settings window that closes itself on a stale
        // event is worse than one that acts on the first row.
        assert_eq!(ShortcutField::from_index(-1), ShortcutField::Switcher);
        assert_eq!(ShortcutField::from_index(99), ShortcutField::Switcher);
    }

    #[test]
    fn every_field_has_a_distinct_key_label_and_description() {
        let mut keys: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.key()).collect();
        let mut labels: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.label()).collect();
        let mut descs: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.description()).collect();
        for list in [&mut keys, &mut labels, &mut descs] {
            let before = list.len();
            list.sort_unstable();
            list.dedup();
            assert_eq!(before, list.len(), "two fields share a value");
        }
        // Both reverse lookups must resolve every field: `from_key` is how a save-time
        // rejection becomes a human label, and `from_label` is how the renderer maps a focus
        // stop back to a field. A field missing from either breaks silently, not at compile
        // time.
        for f in ShortcutField::ALL {
            assert_eq!(ShortcutField::from_key(f.key()), Some(f));
            assert_eq!(ShortcutField::from_label(f.label()), Some(f));
        }
    }

    #[test]
    fn shortcut_field_group_declares_five_taxonomic_groups() {
        // `DEC-014`: the taxonomy groups by the literal shape of what the action snaps to,
        // replacing three groups that fell out of declaration order. Membership is asserted
        // field by field rather than by counting headings, because a field silently landing
        // in a neighbouring group is exactly the drift the old taxonomy was.
        use ShortcutField::*;
        let expected: [(ShortcutField, &str); 16] = [
            (Switcher, "Switching"),
            (Fallback, "Switching"),
            (SnapLeft, "Snap to half"),
            (SnapRight, "Snap to half"),
            (SnapTop, "Snap to half"),
            (SnapBottom, "Snap to half"),
            (SnapThirdLeft, "Snap to third"),
            (SnapThirdMiddle, "Snap to third"),
            (SnapThirdRight, "Snap to third"),
            (SnapPercentLeft, "Snap to custom"),
            (SnapPercentRight, "Snap to custom"),
            (SnapPercentTop, "Snap to custom"),
            (SnapPercentBottom, "Snap to custom"),
            (SnapMaximize, "Resize, move & arrange"),
            (MoveNextMonitor, "Resize, move & arrange"),
            (Stack, "Resize, move & arrange"),
        ];
        for (field, group) in expected {
            assert_eq!(
                field.group(),
                group,
                "{} is in the wrong group",
                field.label()
            );
        }
        // And no sixth group leaks in from a field the table above forgot. Deduplicating the
        // headings in declared order also proves each group's rows are contiguous.
        let mut headings: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.group()).collect();
        headings.dedup();
        assert_eq!(
            headings,
            vec![
                "Switching",
                "Snap to half",
                "Snap to third",
                "Snap to custom",
                "Resize, move & arrange",
            ],
            "the pane headings, in declared order, are exactly the five `DEC-014` names"
        );
    }

    #[test]
    fn pane_enum_no_longer_declares_layout() {
        // `DEC-014`'s accepted extension: the Layout pane is retired, not merely emptied. Its one
        // remaining control moves onto the Overlapping Stack row. Asserted on the count as well as
        // the name, because `Pane::ALL`'s length is itself a hand-written number and a pane list
        // that still says five is stale whether or not anything in it says "Layout".
        assert_eq!(Pane::ALL.len(), 4, "Settings ships four panes");
        assert!(
            Pane::from_label("Layout").is_none(),
            "no pane answers to the retired name"
        );
        for p in Pane::ALL {
            assert_ne!(p.label(), "Layout", "a pane still labels itself Layout");
        }
    }

    #[test]
    fn pane_declaration_order_is_the_navigation_index() {
        // The navigation index crossing the UI boundary is the discriminant, and that identity
        // only holds while the discriminants match `ALL`'s order. Same guard, and same reason, as
        // `field_declaration_order_is_the_precedence_order` — a hand-numbered table that drifts
        // from `ALL` shows one pane while highlighting another's nav entry, and nothing crashes.
        for (i, p) in Pane::ALL.into_iter().enumerate() {
            assert_eq!(p as usize, i, "{} sits out of declared order", p.label());
        }
    }

    #[test]
    fn every_pane_index_round_trips_through_the_ui_boundary() {
        // Asserted against the DISCRIMINANT, not against the loop counter. `from_index` reads
        // `ALL` by position, so `from_index(i) == ALL[i]` is true of any ordering whatsoever and
        // would be a test that cannot fail. What has to hold is that the value `main.rs` sends
        // across the boundary — `pane as i32` — comes back as the same pane, which breaks the
        // moment `ALL`'s order and the discriminants disagree.
        for p in Pane::ALL {
            assert_eq!(
                Pane::from_index(p as i32),
                p,
                "the index `main.rs` sends across the UI boundary must return the same pane"
            );
        }
        // The index arrives from the UI, so out of range must not panic.
        assert_eq!(Pane::from_index(-1), Pane::General);
        assert_eq!(Pane::from_index(99), Pane::General);
    }

    #[test]
    fn overlapping_stack_row_has_percent_true() {
        // `DEC-014`'s extension puts the stack width on the Overlapping Stack row, using the same
        // `has_percent` plumbing the four `Snap to custom` rows already use.
        assert!(
            ShortcutField::Stack.has_percent(),
            "the Overlapping Stack row carries a percentage"
        );
    }

    #[test]
    fn every_percentage_row_reaches_all_four_percent_seams() {
        // `set_percent` and `percent` both end in `_ => {}` / `_ => None` catch-alls, so a field
        // added to `has_percent()` but forgotten in either one fails **silently** — the row draws
        // a control that discards every edit. That is exactly what this ticket was caught on, and
        // `Stack`'s own round-trip test only proves it for `Stack`. This pins the general rule:
        // the four seams agree for every field, so the next percentage row cannot repeat it.
        let mut cfg = Config::default();
        for f in ShortcutField::ALL {
            let claims = f.has_percent();
            assert_eq!(
                claims,
                f.percent_bounds().is_some(),
                "{} disagrees between has_percent and percent_bounds",
                f.label()
            );
            assert_eq!(
                claims,
                f.percent(&cfg).is_some(),
                "{} disagrees between has_percent and percent",
                f.label()
            );
            if !claims {
                continue;
            }
            // And the write actually lands rather than falling through the catch-all. Chosen
            // inside every row's own bounds so no row is refused for being out of range.
            let (min, max) = f.percent_bounds().expect("a percentage row has bounds");
            let target = min + (max - min) / 2;
            f.set_percent(&mut cfg, target);
            assert_eq!(
                f.percent(&cfg),
                Some(target),
                "{} accepted a percentage and discarded it",
                f.label()
            );
        }
    }

    #[test]
    fn the_stack_row_carries_its_own_percent_bounds() {
        // A snap edge accepts 1-99; the overlapping-stack width accepts 10-100. Setting
        // `has_percent` on `Stack` without carrying its range across silently narrows it, because
        // the stepper's bounds used to be literals in the markup.
        assert_eq!(ShortcutField::Stack.percent_bounds(), Some((10, 100)));
        for f in [
            ShortcutField::SnapPercentLeft,
            ShortcutField::SnapPercentRight,
            ShortcutField::SnapPercentTop,
            ShortcutField::SnapPercentBottom,
        ] {
            assert_eq!(f.percent_bounds(), Some((1, 99)), "{}", f.label());
        }
        // And the two facts cannot drift apart: a row that draws a percentage must have a range,
        // and a row with a range must draw one. Either half alone is a silently broken control.
        for f in ShortcutField::ALL {
            assert_eq!(
                f.has_percent(),
                f.percent_bounds().is_some(),
                "{} disagrees about whether it has a percentage",
                f.label()
            );
        }
    }

    #[test]
    fn the_group_headings_have_one_home() {
        // `ShortcutField::GROUPS` is that home. Walking `ALL` and deduplicating the headings
        // must reproduce it exactly — which is only true while every heading `group()` returns
        // is a member, in the drawn order, with each group's rows contiguous. A heading typed
        // out somewhere else and misspelled selects no rows and draws an empty group silently.
        let mut walked: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.group()).collect();
        walked.dedup();
        assert_eq!(walked, ShortcutField::GROUPS.to_vec());
    }

    #[test]
    fn snap_maximize_now_sorts_after_every_snap_variant() {
        // `DEC-014` moves Maximize from index 6 to index 13 — out of the old "Snap & resize"
        // group and into the last one, behind every snap variant. This inverts the previous
        // `SnapMaximize < SnapThirdLeft` assertion on purpose: it is a precedence change, so
        // the new relative position is asserted rather than the old one simply deleted.
        let maximize = ShortcutField::SnapMaximize as usize;
        for earlier in [
            ShortcutField::SnapLeft,
            ShortcutField::SnapRight,
            ShortcutField::SnapTop,
            ShortcutField::SnapBottom,
            ShortcutField::SnapThirdLeft,
            ShortcutField::SnapThirdMiddle,
            ShortcutField::SnapThirdRight,
            ShortcutField::SnapPercentLeft,
            ShortcutField::SnapPercentRight,
            ShortcutField::SnapPercentTop,
            ShortcutField::SnapPercentBottom,
        ] {
            assert!(
                (earlier as usize) < maximize,
                "{} must keep precedence over Maximize",
                earlier.label()
            );
        }
        // It leads its own group, ahead of the two non-snap actions.
        assert!(maximize < (ShortcutField::MoveNextMonitor as usize));
        assert!(maximize < (ShortcutField::Stack as usize));
    }

    #[test]
    fn snap_custom_labels_no_longer_repeat_the_group_name() {
        // The "Snap to custom" heading now carries the meaning the `(custom %)` suffix used
        // to. `label()` is also `from_label`'s lookup key, so this asserts the new label
        // round-trips rather than only that the old substring is gone.
        for f in [
            ShortcutField::SnapPercentLeft,
            ShortcutField::SnapPercentRight,
            ShortcutField::SnapPercentTop,
            ShortcutField::SnapPercentBottom,
        ] {
            assert!(
                !f.label().contains("(custom"),
                "{} still repeats its group name",
                f.label()
            );
            assert_eq!(ShortcutField::from_label(f.label()), Some(f));
        }
        assert_eq!(ShortcutField::SnapPercentLeft.label(), "Snap to left edge");
        assert_eq!(
            ShortcutField::SnapPercentRight.label(),
            "Snap to right edge"
        );
        assert_eq!(ShortcutField::SnapPercentTop.label(), "Snap to top edge");
        assert_eq!(
            ShortcutField::SnapPercentBottom.label(),
            "Snap to bottom edge"
        );
    }

    #[test]
    fn the_declared_sequence_matches_the_shared_source() {
        // `LBR-ST-14` / `DEC-018`: one declared sequence, and it lives in `shared` so the
        // daemon's collision precedence and this pane's order cannot drift apart. Asserted
        // against the constant, never against a fresh literal of the same order — a second
        // literal here would be the very thing the rule forbids.
        let keys: Vec<&str> = ShortcutField::ALL.iter().map(|f| f.key()).collect();
        assert_eq!(keys, shared::constants::SHORTCUT_DECLARED_ORDER.to_vec());
    }

    #[test]
    fn grouping_never_reorders_the_declared_sequence() {
        // `LBR-ST-14`: a heading may gather rows, never move them. Concatenating the groups
        // in pane order must reproduce `ALL` exactly — which is only true while each group's
        // members are contiguous in the declared sequence.
        let mut regrouped: Vec<ShortcutField> = Vec::new();
        for heading in [
            "Switching",
            "Snap to half",
            "Snap to third",
            "Snap to custom",
            "Resize, move & arrange",
        ] {
            regrouped.extend(
                ShortcutField::ALL
                    .into_iter()
                    .filter(|f| f.group() == heading),
            );
        }
        assert_eq!(regrouped, ShortcutField::ALL.to_vec());
    }

    #[test]
    fn no_chord_field_lives_outside_the_shortcuts_pane() {
        // The capture lease is armed from which pane is showing (`DEC-004`), so a chord field
        // in a second pane means two panes arming the observe lease — a regression in the
        // `DEC-005` key check. Asserted because the pane split is the thing a future tidy-up
        // is most likely to undo.
        for pane in Pane::ALL {
            if pane == Pane::Shortcuts {
                continue;
            }
            let order = focus_order(pane);
            for f in ShortcutField::ALL {
                assert!(
                    !order.contains(&f.label()),
                    "{} appears in the {} pane",
                    f.label(),
                    pane.label()
                );
            }
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

    /// The defect this guards was reported from a real install: completing the tutorial
    /// made everything disappear instead of landing in Settings, so a user who finished
    /// was worse off than one who skipped.
    ///
    /// Dismissal has to be expressible separately from reaching the end, because the
    /// terminal state `Some(Done)` is what `skip` produces too, and the view decides
    /// whether to draw the tutorial from `onboarding.is_some()`.
    #[test]
    fn dismissing_the_tutorial_makes_settings_reachable() {
        let mut m = SettingsModel::new(Config::default(), true);
        while !m.advance_onboarding() {}
        assert_eq!(
            m.onboarding,
            Some(OnboardingStep::Done),
            "advancing to the end lands on the last screen, not past it"
        );

        // While this holds, the settings panes are not rendered at all.
        assert!(m.onboarding.is_some());

        m.dismiss_onboarding();
        assert!(
            m.onboarding.is_none(),
            "after finishing, the tutorial must stop being drawn or Settings stays unreachable"
        );
    }

    /// The asymmetry that matters more than the checkbox itself. Skip is on the first
    /// screen and the question is on the last, so a skipped tutorial has not answered it —
    /// and answering it yes would register a task that runs elevated at every sign-in with
    /// no prompt, for a user who never saw the question.
    #[test]
    fn skipping_the_tutorial_does_not_answer_the_auto_start_question() {
        let mut skipped = SettingsModel::new(Config::default(), true);
        assert!(
            skipped.onboarding_auto_start,
            "the question is pre-checked, which is the whole reason this test exists"
        );
        skipped.skip_onboarding();
        assert!(
            !skipped.draft.general.auto_start,
            "skipping must leave auto-start off; only finishing commits the answer"
        );
    }

    #[test]
    fn finishing_commits_the_pre_checked_auto_start_answer() {
        let mut m = SettingsModel::new(Config::default(), true);
        while !m.advance_onboarding() {}
        m.accept_onboarding_auto_start();
        assert!(m.draft.general.auto_start);
    }

    #[test]
    fn finishing_after_declining_leaves_auto_start_off() {
        let mut m = SettingsModel::new(Config::default(), true);
        m.onboarding_auto_start = false;
        while !m.advance_onboarding() {}
        m.accept_onboarding_auto_start();
        assert!(
            !m.draft.general.auto_start,
            "unticking the box must survive finishing, or the question was decoration"
        );
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
    fn onboarding_dummy_window_focus_toggles_on_shortcut() {
        let mut m = SettingsModel::new(Config::default(), true);
        assert_eq!(m.onboarding_focus_index, 0);
        assert!(!m.onboarding_simulated_success);

        m.toggle_onboarding_simulation();
        assert_eq!(m.onboarding_focus_index, 1);
        assert!(m.onboarding_simulated_success);

        m.toggle_onboarding_simulation();
        assert_eq!(m.onboarding_focus_index, 0);
        assert!(m.onboarding_simulated_success);
    }

    #[test]
    fn onboarding_escape_triggers_skip() {
        let mut m = SettingsModel::new(Config::default(), true);
        assert_eq!(m.onboarding, Some(OnboardingStep::Welcome));
        m.skip_onboarding();
        assert_eq!(m.onboarding, Some(OnboardingStep::Done));
    }

    #[test]
    fn shortcut_conflict_detection_finds_both_directions() {
        let mut m = model();
        assert!(!m.has_any_conflict());
        assert_eq!(m.find_conflict(ShortcutField::Switcher), None);

        // Assign same shortcut to SnapLeft as Switcher ("win+backtick")
        m.draft.snapping.snap_half_left = "win+backtick".to_string();
        assert!(m.has_any_conflict());
        assert_eq!(
            m.find_conflict(ShortcutField::Switcher),
            Some(ShortcutField::SnapLeft)
        );
        assert_eq!(
            m.find_conflict(ShortcutField::SnapLeft),
            Some(ShortcutField::Switcher)
        );
    }

    #[test]
    fn disabling_an_action_retains_its_stored_chord() {
        let mut m = model();
        let original_chord = ShortcutField::SnapLeft.get(&m.draft).to_string();
        assert!(!original_chord.is_empty());
        assert!(ShortcutField::SnapLeft.is_enabled(&m.draft));

        ShortcutField::SnapLeft.set_enabled(&mut m.draft, false);
        assert!(!ShortcutField::SnapLeft.is_enabled(&m.draft));
        assert_eq!(
            ShortcutField::SnapLeft.get(&m.draft),
            original_chord,
            "disabling an action must retain its stored chord"
        );
    }

    #[test]
    fn re_enabling_an_action_restores_its_prior_chord() {
        let mut m = model();
        let original_chord = ShortcutField::SnapThirdLeft.get(&m.draft).to_string();

        ShortcutField::SnapThirdLeft.set_enabled(&mut m.draft, false);
        assert!(!ShortcutField::SnapThirdLeft.is_enabled(&m.draft));

        ShortcutField::SnapThirdLeft.set_enabled(&mut m.draft, true);
        assert!(ShortcutField::SnapThirdLeft.is_enabled(&m.draft));
        assert_eq!(
            ShortcutField::SnapThirdLeft.get(&m.draft),
            original_chord,
            "re-enabling an action must restore its prior chord"
        );
    }

    #[test]
    fn find_conflict_excludes_disabled_actions() {
        let mut m = model();
        m.draft.snapping.snap_half_left = "win+backtick".to_string();
        assert_eq!(
            m.find_conflict(ShortcutField::Switcher),
            Some(ShortcutField::SnapLeft)
        );
        assert_eq!(
            m.find_conflict(ShortcutField::SnapLeft),
            Some(ShortcutField::Switcher)
        );

        // Disable SnapLeft
        ShortcutField::SnapLeft.set_enabled(&mut m.draft, false);
        assert_eq!(m.find_conflict(ShortcutField::Switcher), None);
        assert_eq!(m.find_conflict(ShortcutField::SnapLeft), None);
        assert!(!m.has_any_conflict());
    }

    #[test]
    fn a_hand_edited_conflict_has_nothing_swap_can_undo() {
        // A collision that did not arise from a capture in this session (a
        // hand-edited config.toml, or one loaded from disk) has no recorded
        // "previous" chord for either field, so neither should be offered as
        // the swappable side.
        let mut m = model();
        m.draft.snapping.snap_half_left = "win+backtick".to_string();
        assert!(m.has_any_conflict());
        assert!(!m.can_swap(ShortcutField::Switcher));
        assert!(!m.can_swap(ShortcutField::SnapLeft));
        assert!(!m.any_swappable_conflict());
    }

    #[test]
    fn swap_gives_the_conflict_partner_back_the_chord_the_capture_displaced() {
        let mut m = model();
        let snap_left_before = m.draft.snapping.snap_half_left.clone();
        let switcher_before = m.draft.switcher.shortcut.clone();
        assert_ne!(
            snap_left_before, switcher_before,
            "test setup requires these to start distinct"
        );

        // Capture Switcher's existing chord for SnapLeft. This is the only
        // way a collision can enter the draft: a legal chord that happens to
        // already belong to another action (DEC-001 / SCN-03).
        m.begin_capture(ShortcutField::SnapLeft);
        m.accept_capture(&switcher_before).unwrap();

        assert!(m.has_any_conflict());
        assert_eq!(m.draft.snapping.snap_half_left, switcher_before);
        assert_eq!(m.draft.switcher.shortcut, switcher_before);

        // Only the field the capture actually wrote into can be swapped —
        // the field it collided with lost nothing and has nothing on record.
        assert!(m.can_swap(ShortcutField::SnapLeft));
        assert!(!m.can_swap(ShortcutField::Switcher));
        assert!(m.any_swappable_conflict());

        let conf_field = m
            .find_conflict(ShortcutField::SnapLeft)
            .expect("SnapLeft must still read as conflicted before the swap");
        m.swap_shortcuts(ShortcutField::SnapLeft, conf_field);

        // SnapLeft keeps the chord the user just captured; Switcher gets back
        // the chord SnapLeft displaced. Two distinct values again — not the
        // identity swap the old implementation performed on equal strings.
        assert_eq!(m.draft.snapping.snap_half_left, switcher_before);
        assert_eq!(m.draft.switcher.shortcut, snap_left_before);
        assert!(!m.has_any_conflict());
    }

    #[test]
    fn swap_is_a_no_op_when_called_for_the_field_that_did_not_just_capture() {
        let mut m = model();
        let switcher_before = m.draft.switcher.shortcut.clone();
        m.begin_capture(ShortcutField::SnapLeft);
        m.accept_capture(&switcher_before).unwrap();
        let before = m.draft.clone();

        // Calling with the roles reversed must not corrupt the draft — there
        // is no displaced chord on record for Switcher to give back.
        m.swap_shortcuts(ShortcutField::Switcher, ShortcutField::SnapLeft);
        assert_eq!(m.draft, before, "an inert swap must not touch the draft");
        assert!(m.has_any_conflict(), "the collision must still stand");
    }

    #[test]
    fn reverting_forgets_the_displaced_chord() {
        let mut m = model();
        let switcher_before = m.draft.switcher.shortcut.clone();
        m.begin_capture(ShortcutField::SnapLeft);
        m.accept_capture(&switcher_before).unwrap();
        assert!(m.can_swap(ShortcutField::SnapLeft));

        m.revert();
        assert!(!m.can_swap(ShortcutField::SnapLeft));
    }

    /// The rule as it was before, and it is correct as far as it goes: a level the daemon
    /// already has does not need sending again.
    #[test]
    fn a_delivered_lease_at_the_same_level_is_not_resent() {
        assert!(!lease_post_owed(
            CAPTURE_LEASE_RECORD,
            CAPTURE_LEASE_RECORD,
            DaemonSignal::Delivered
        ));
        assert!(lease_post_owed(
            CAPTURE_LEASE_OBSERVE,
            CAPTURE_LEASE_RECORD,
            DaemonSignal::Delivered
        ));
    }

    /// The defect, as a test. A Settings started outside the tray sits below the daemon's
    /// integrity level, every post is discarded by UIPI, and the old rule remembered the
    /// first discard as a success — so nothing was ever sent again and shortcut recording
    /// received nothing for the life of the window.
    #[test]
    fn a_refused_lease_is_retried_rather_than_remembered_as_sent() {
        assert!(
            lease_post_owed(
                CAPTURE_LEASE_RECORD,
                CAPTURE_LEASE_RECORD,
                DaemonSignal::Refused
            ),
            "a lease the daemon never received is still owed, however many times we tried"
        );
    }

    /// The same holds when the daemon simply is not running yet — it may start while this
    /// window is open, and the lease must reach it when it does.
    #[test]
    fn an_absent_daemon_leaves_the_lease_owed() {
        assert!(lease_post_owed(
            CAPTURE_LEASE_RECORD,
            CAPTURE_LEASE_RECORD,
            DaemonSignal::DaemonAbsent
        ));
    }

    /// Only a refusal earns the warning. An absent daemon is an ordinary state — telling
    /// someone their window cannot reach a program that is not running would be noise, and
    /// it is the distinction the old `bool` could not express.
    #[test]
    fn only_a_refusal_is_worth_warning_about() {
        let mut m = model();

        m.lease_signal = DaemonSignal::DaemonAbsent;
        assert!(!m.lease_refused());

        m.lease_signal = DaemonSignal::Delivered;
        assert!(!m.lease_refused());

        m.lease_signal = DaemonSignal::Refused;
        assert!(m.lease_refused());
    }

    #[test]
    fn a_successful_capture_clears_stale_feedback() {
        let mut m = model();
        m.feedback = SaveFeedback::Saved {
            reload: DaemonSignal::Delivered,
        };
        m.begin_capture(ShortcutField::SnapRight);
        m.accept_capture("ctrl+alt+f7").unwrap();
        assert_eq!(m.feedback, SaveFeedback::None);
    }

    #[test]
    fn reserved_windows_system_shortcut_is_rejected() {
        let mut m = model();
        m.begin_capture(ShortcutField::Switcher);
        // Win + L is reserved for Lock Workstation
        let res = m.accept_capture("win+l");
        assert!(matches!(res, Err(ShortcutError::Reserved(_))));
    }

    #[test]
    fn duplicate_shortcut_is_rejected_on_save() {
        let mut m = model();
        m.draft.snapping.snap_half_left = "win+backtick".to_string(); // Duplicate of Switcher
        let dir = temp_dir();
        let path = dir.join("config.toml");
        m.save(&path);
        assert!(matches!(m.feedback, SaveFeedback::Error(_)));
    }

    #[test]
    fn onboarding_teaches_the_spatial_philosophy() {
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
        // A raw config key, as the save path passes it, must be resolved to
        // the field's human label — a user was never shown `config.toml` and
        // should never be asked to recognize `switcher.shortcut` in it.
        let msg = describe("switcher.shortcut", ShortcutError::MultipleMainKeys);
        assert!(msg.contains(ShortcutField::Switcher.label()));
        assert!(
            !msg.contains("switcher.shortcut"),
            "message must not leak the config key: {msg:?}"
        );
        assert!(msg.contains("one main key"));
    }

    #[test]
    fn validation_messages_pass_through_an_already_resolved_label() {
        // The inline capture path already resolves to a label before calling
        // describe(); from_key must not mangle a string that is not a key.
        let msg = describe(ShortcutField::Switcher.label(), ShortcutError::NoModifier);
        assert!(msg.contains(ShortcutField::Switcher.label()));
    }

    #[test]
    fn duplicate_conflict_message_names_both_actions_not_config_keys() {
        let msg = describe(
            ShortcutField::Fallback.key(),
            ShortcutError::DuplicateShortcut(ShortcutField::Switcher.key()),
        );
        assert!(msg.contains(ShortcutField::Fallback.label()));
        assert!(msg.contains(ShortcutField::Switcher.label()));
        assert!(!msg.contains("switcher.fallback_shortcut"));
        assert!(!msg.contains("switcher.shortcut"));
    }

    #[test]
    fn shortcut_field_key_round_trips_with_from_key() {
        for field in ShortcutField::ALL {
            assert_eq!(ShortcutField::from_key(field.key()), Some(field));
        }
        assert_eq!(ShortcutField::from_key("not.a.real.key"), None);
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
    fn an_out_of_range_percentage_save_reports_actionable_error() {
        let mut m = model();
        m.draft.snapping.percent_left = 0;
        let mut path = std::env::temp_dir();
        path.push(format!("wiradesk-app-pct-test-{}.toml", std::process::id()));
        m.save(&path);
        if let SaveFeedback::Error(msg) = &m.feedback {
            assert!(msg.contains("Left edge snap percentage"));
            assert!(msg.contains("between 1% and 99%"));
        } else {
            panic!("expected SaveFeedback::Error");
        }
        assert_eq!(m.saved.snapping.percent_left, 50);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn percent_get_and_set_updates_draft_and_marks_dirty() {
        let mut m = model();
        assert_eq!(ShortcutField::SnapPercentLeft.percent(&m.draft), Some(50));
        assert_eq!(ShortcutField::Switcher.percent(&m.draft), None);

        ShortcutField::SnapPercentLeft.set_percent(&mut m.draft, 70);
        assert_eq!(m.draft.snapping.percent_left, 70);
        assert!(m.is_dirty());
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

    // ── DEC-004 capture lease: one owning place ───────────────────────────

    #[test]
    fn idle_on_a_non_shortcuts_pane_desires_no_lease() {
        let m = model();
        assert_eq!(m.desired_lease_level(), CAPTURE_LEASE_NONE);
    }

    #[test]
    fn opening_the_shortcuts_pane_desires_the_observe_lease() {
        let mut m = model();
        m.set_pane(Pane::Shortcuts);
        assert_eq!(m.desired_lease_level(), CAPTURE_LEASE_OBSERVE);
    }

    #[test]
    fn listening_for_a_field_desires_the_record_lease_even_off_the_shortcuts_pane() {
        // Cannot happen in practice (capture only starts from the Shortcuts
        // pane), but the derivation must not depend on pane at all once a
        // field is listening — record implies observe, never the reverse.
        let mut m = model();
        m.begin_capture(ShortcutField::Switcher);
        m.pane = Pane::General;
        assert_eq!(m.desired_lease_level(), CAPTURE_LEASE_RECORD);
    }

    #[test]
    fn leaving_the_shortcuts_pane_mid_capture_disarms_down_to_none_not_observe() {
        // The live defect this replaces: `set_pane` used to cancel the
        // capture without disarming anything, leaving whatever lease was
        // last sent standing. Leaving the pane entirely must fall all the
        // way to `none`, not merely down to `observe`.
        let mut m = model();
        m.set_pane(Pane::Shortcuts);
        m.begin_capture(ShortcutField::Switcher);
        assert_eq!(m.sent_lease_level, CAPTURE_LEASE_RECORD);

        m.set_pane(Pane::General);
        assert_eq!(m.capture, CaptureState::Idle);
        assert_eq!(m.desired_lease_level(), CAPTURE_LEASE_NONE);
        assert_eq!(m.sent_lease_level, CAPTURE_LEASE_NONE);
    }

    #[test]
    fn accepting_a_capture_drops_the_lease_back_to_observe() {
        let mut m = model();
        m.set_pane(Pane::Shortcuts);
        m.begin_capture(ShortcutField::SnapRight);
        assert_eq!(m.sent_lease_level, CAPTURE_LEASE_RECORD);

        m.accept_capture("ctrl+alt+f7").unwrap();
        assert_eq!(m.sent_lease_level, CAPTURE_LEASE_OBSERVE);
    }

    // ── DEC-005 correlation ───────────────────────────────────────────────

    #[test]
    fn a_hook_report_matched_by_the_window_reports_received() {
        let mut kc = KeyCheckState::default();
        kc.record_hook_report(0x31, false, true, false, false); // Win+1
        kc.record_key(
            "Win + 1",
            "win+1",
            true,
            (false, true, false, false),
            Some(0x31),
        );
        assert_eq!(kc.verdict, KeyCheckVerdict::Received);
    }

    #[test]
    fn a_hook_report_never_matched_by_the_window_reports_claimed_by_another_app() {
        let mut kc = KeyCheckState::default();
        kc.record_hook_report(0x31, false, true, false, false);
        for _ in 0..HOOK_REPORT_GRACE_TICKS {
            assert_eq!(
                kc.verdict,
                KeyCheckVerdict::Idle,
                "must not guess before the grace period ends"
            );
            kc.tick();
        }
        assert_eq!(kc.verdict, KeyCheckVerdict::ClaimedByOtherApp);
    }

    #[test]
    fn a_claimed_by_other_app_display_uses_the_same_symbols_as_everywhere_else() {
        // Regression: this readout used to build its display text straight
        // from `shared::shortcut::name_from_vk` (e.g. the word "down"),
        // while every other shortcut readout in the UI goes through
        // `format_shortcut_display` and shows the arrow/backtick glyph
        // instead — so the Key check pane alone showed "Ctrl + Alt + down"
        // next to a row that showed "Ctrl + Alt + ↓" for the same key.
        let mut kc = KeyCheckState::default();
        kc.record_hook_report(0x28, true, false, true, false); // Ctrl+Alt+Down
        for _ in 0..HOOK_REPORT_GRACE_TICKS {
            kc.tick();
        }
        assert_eq!(kc.last_display, "Ctrl + Alt + ↓");
    }

    #[test]
    fn a_claimed_by_other_app_display_marks_an_unrepresentable_key_explicitly() {
        let mut kc = KeyCheckState::default();
        kc.record_hook_report(0xBA, false, true, false, false); // Win+Semicolon
        for _ in 0..HOOK_REPORT_GRACE_TICKS {
            kc.tick();
        }
        assert_eq!(kc.last_display, "Win + (code 0xBA)");
    }

    #[test]
    fn a_window_event_with_no_hook_report_while_the_daemon_runs_reports_intercepted() {
        // DEF-4: the daemon is running (third arg `true`) but no hook report
        // ever correlated — an earlier third-party LL hook swallowed the
        // chord before Wira Desk's own hook saw it. DEC-005 names this
        // `Intercepted`, distinct from the daemon genuinely being absent.
        let mut kc = KeyCheckState::default();
        kc.record_key(
            "Alt + 1",
            "alt+1",
            true,
            (false, false, true, false),
            Some(0x31),
        );
        assert_eq!(kc.verdict, KeyCheckVerdict::Intercepted);
    }

    #[test]
    fn a_window_event_with_the_daemon_absent_reports_daemon_not_running() {
        let mut kc = KeyCheckState::default();
        kc.record_key(
            "Alt + 1",
            "alt+1",
            false,
            (false, false, true, false),
            Some(0x31),
        );
        assert_eq!(kc.verdict, KeyCheckVerdict::DaemonNotRunning);
    }

    #[test]
    fn key_check_state_pure_observer_does_not_mutate_draft() {
        let mut m = model();
        let initial_draft = m.draft.clone();
        assert!(!m.is_dirty());

        // Update modifiers and record key. A matching hook report must have
        // arrived first for this to correlate as `Received` — DEC-005 never
        // reports a verdict from the window signal alone.
        m.key_check.update_modifiers(true, false, true, false);
        m.key_check
            .record_hook_report(0x31, false, false, true, false);
        m.key_check.record_key(
            "Alt + 1",
            "alt+1",
            true,
            (false, false, true, false),
            Some(0x31),
        );

        assert!(m.key_check.mod_ctrl);
        assert!(m.key_check.mod_alt);
        assert_eq!(m.key_check.last_display, "Alt + 1");
        assert_eq!(m.key_check.last_canonical, "alt+1");
        assert_eq!(m.key_check.verdict, KeyCheckVerdict::Received);
        assert!(m.key_check.beat);

        // Draft and is_dirty must remain 100% untouched
        assert_eq!(m.draft, initial_draft);
        assert!(!m.is_dirty());

        // Clearing beat clears the heartbeat flag
        m.key_check.clear_beat();
        assert!(!m.key_check.beat);
    }
}
