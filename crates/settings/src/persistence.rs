//! Settings-side persistence and IPC contract with the daemon.
//! Ordering is the whole point of this module: validate before replacing,
//! replace atomically, and signal reload **only after** the completed file is
//! visible. Any other order can leave the daemon reading a half-written or
//! stale configuration.

use std::path::Path;

use shared::constants::{
    DAEMON_WINDOW_CLASS, DAEMON_WINDOW_TITLE, SHORTCUT_DECLARED_ORDER, WM_APP_RELOAD_CONFIG,
};
use shared::{config_path, Config, Shortcut};

use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW};

pub use shared::shortcut::{reservation, ReservedInfo};

/// Why a submitted shortcut was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutError {
    /// A token that is not a modifier and not a known key name.
    UnsupportedToken,
    /// Modifiers only — nothing to press.
    NoMainKey,
    /// More than one non-modifier key.
    MultipleMainKeys,
    /// A main key with no modifier at all — a bare key like `"a"` is not a
    /// safe global shortcut (it would fire on ordinary typing).
    NoModifier,
    /// Parses, but cannot be written back in canonical form.
    Unrepresentable,
    /// Shortcut is reserved by the Windows operating system with owner details.
    Reserved(ReservedInfo),
    /// Shortcut is already assigned to another action in the configuration.
    DuplicateShortcut(&'static str),
    /// A custom snap percentage was outside 1..=99.
    InvalidPercentage(u32),
}

/// Validate a submitted shortcut **before** any active configuration is
/// replaced atomically.
/// Returns the canonical string to persist, so a caller cannot accidentally
/// store the user's raw input in a non-canonical form.
pub fn validate_shortcut(input: &str) -> Result<String, ShortcutError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ShortcutError::NoMainKey);
    }

    // `Shortcut::parse` collapses three distinct failures into `None`, so the
    // reason is recovered here — a user who typed two main keys deserves a
    // different message from one who typed a nonsense token.
    match Shortcut::parse(trimmed) {
        Some(sc) => {
            // A parsed shortcut with a main key but no modifier (e.g. a bare
            // `"a"`) is technically well-formed but not a legal global
            // shortcut — reject it here rather than letting it round-trip.
            if !sc.has_modifier() {
                return Err(ShortcutError::NoModifier);
            }
            if let Some(res) = reservation(&sc) {
                return Err(ShortcutError::Reserved(res));
            }
            sc.to_canonical_string()
                .ok_or(ShortcutError::Unrepresentable)
        }
        None => Err(classify_parse_failure(trimmed)),
    }
}

fn classify_parse_failure(input: &str) -> ShortcutError {
    let mut main_keys = 0usize;
    let mut unsupported = false;

    for raw in input.split('+') {
        let token = raw.trim().to_ascii_lowercase();
        if token.is_empty() {
            continue;
        }
        match token.as_str() {
            "win" | "meta" | "super" | "ctrl" | "control" | "alt" | "shift" => {}
            other => {
                if shared::shortcut::vk_from_name(other).is_some() {
                    main_keys += 1;
                } else {
                    unsupported = true;
                }
            }
        }
    }

    if unsupported {
        ShortcutError::UnsupportedToken
    } else if main_keys > 1 {
        ShortcutError::MultipleMainKeys
    } else {
        ShortcutError::NoMainKey
    }
}

/// Validate every shortcut field in a candidate configuration, ensuring both validity
/// and uniqueness across actions.
/// Returns the offending field name and reason on the first failure, leaving
/// the caller's active configuration untouched.
pub fn validate_config(cfg: &Config) -> Result<(), (&'static str, ShortcutError)> {
    // The validation and duplicate-rejection sequence is derived directly from
    // `shared::constants::SHORTCUT_DECLARED_ORDER` (`LBR-ST-14`, `DEC-018`).
    // The order decides which of two colliding fields is named as the first holder.
    // Iterating the shared constant guarantees that this validation walk matches
    // the daemon's precedence order while keeping this module free of any dependency
    // on the UI-facing `ShortcutField` enum. The key-to-value mapping below is not a
    // second declared order: the constant dictates the iteration order.
    fn field_for<'a>(cfg: &'a Config, key: &str) -> (&'a str, bool) {
        match key {
            "switcher.shortcut" => (&cfg.switcher.shortcut, cfg.switcher.shortcut_enabled),
            "switcher.fallback_shortcut" => (
                &cfg.switcher.fallback_shortcut,
                cfg.switcher.fallback_shortcut_enabled,
            ),
            "snapping.snap_half_left" => (
                &cfg.snapping.snap_half_left,
                cfg.snapping.snap_half_left_enabled,
            ),
            "snapping.snap_half_right" => (
                &cfg.snapping.snap_half_right,
                cfg.snapping.snap_half_right_enabled,
            ),
            "snapping.snap_half_top" => (
                &cfg.snapping.snap_half_top,
                cfg.snapping.snap_half_top_enabled,
            ),
            "snapping.snap_half_bottom" => (
                &cfg.snapping.snap_half_bottom,
                cfg.snapping.snap_half_bottom_enabled,
            ),
            "snapping.snap_third_left" => (
                &cfg.snapping.snap_third_left,
                cfg.snapping.snap_third_left_enabled,
            ),
            "snapping.snap_third_middle" => (
                &cfg.snapping.snap_third_middle,
                cfg.snapping.snap_third_middle_enabled,
            ),
            "snapping.snap_third_right" => (
                &cfg.snapping.snap_third_right,
                cfg.snapping.snap_third_right_enabled,
            ),
            "snapping.snap_percent_left" => (
                &cfg.snapping.snap_percent_left,
                cfg.snapping.snap_percent_left_enabled,
            ),
            "snapping.snap_percent_right" => (
                &cfg.snapping.snap_percent_right,
                cfg.snapping.snap_percent_right_enabled,
            ),
            "snapping.snap_percent_top" => (
                &cfg.snapping.snap_percent_top,
                cfg.snapping.snap_percent_top_enabled,
            ),
            "snapping.snap_percent_bottom" => (
                &cfg.snapping.snap_percent_bottom,
                cfg.snapping.snap_percent_bottom_enabled,
            ),
            "snapping.snap_maximize" => (
                &cfg.snapping.snap_maximize,
                cfg.snapping.snap_maximize_enabled,
            ),
            "layout.move_next_monitor_shortcut" => (
                &cfg.layout.move_next_monitor_shortcut,
                cfg.layout.move_next_monitor_shortcut_enabled,
            ),
            "layout.stack_shortcut" => (
                &cfg.layout.stack_shortcut,
                cfg.layout.stack_shortcut_enabled,
            ),
            // Unreachable while this match and `SHORTCUT_DECLARED_ORDER` hold the same sixteen
            // keys. It is not left to chance: the walk visits every key in the constant
            // regardless of config, so `default_config_passes_its_own_validation` traverses all
            // sixteen arms on every test run and a seventeenth key added without an arm fails
            // the suite loudly, naming itself. Failing open here — skipping an unknown key —
            // would leave a field silently unvalidated, which is the worse trade.
            other => {
                panic!("{other} is in the shared declared order but unknown in validate_config")
            }
        }
    }

    let mut seen: Vec<(&'static str, String)> = Vec::with_capacity(SHORTCUT_DECLARED_ORDER.len());

    for &name in &SHORTCUT_DECLARED_ORDER {
        let (value, enabled) = field_for(cfg, name);
        let canonical = validate_shortcut(value).map_err(|e| (name, e))?;
        if enabled {
            if let Some((first_name, _)) = seen.iter().find(|(_, s)| *s == canonical) {
                return Err((name, ShortcutError::DuplicateShortcut(first_name)));
            }
            seen.push((name, canonical));
        }
    }

    for (name, val) in [
        ("snapping.percent_left", cfg.snapping.percent_left),
        ("snapping.percent_right", cfg.snapping.percent_right),
        ("snapping.percent_top", cfg.snapping.percent_top),
        ("snapping.percent_bottom", cfg.snapping.percent_bottom),
    ] {
        if !(shared::constants::MIN_SNAP_PERCENT..=shared::constants::MAX_SNAP_PERCENT)
            .contains(&val)
        {
            return Err((name, ShortcutError::InvalidPercentage(val)));
        }
    }

    if !(shared::constants::MIN_STACK_WIDTH_PERCENT..=shared::constants::MAX_STACK_WIDTH_PERCENT)
        .contains(&cfg.layout.stack_width_percent)
    {
        return Err((
            "layout.stack_width_percent",
            ShortcutError::InvalidPercentage(cfg.layout.stack_width_percent),
        ));
    }

    Ok(())
}

/// What became of a message posted to the daemon.
///
/// **Three outcomes rather than a `bool`, and the reason is a defect this replaces.** Both
/// failures return `0` from `PostMessageW` and are indistinguishable at the call site, but
/// they mean opposite things to a user: one says the daemon is not running, the other says
/// it is running and refused to listen. Collapsing them showed "saved for next launch" —
/// a sentence that reads like a normal outcome — to a user whose daemon was plainly alive
/// in the tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonSignal {
    /// The daemon has it and will act on it.
    Delivered,
    /// No daemon window exists. Not an error: it reads the configuration when it starts.
    DaemonAbsent,
    /// The daemon is there and Windows dropped the message. In practice this means this
    /// process sits below the daemon's integrity level — Settings started from Explorer
    /// rather than from the tray — and UIPI discarded the post. See
    /// `.how/settings/02-contracts/contract-reload-config.md`.
    Refused,
}

impl DaemonSignal {
    /// Whether the daemon actually received it.
    pub fn delivered(self) -> bool {
        self == DaemonSignal::Delivered
    }
}

/// Outcome of a save attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved { reload: DaemonSignal },
    Rejected(&'static str, ShortcutError),
    WriteFailed(String),
}

/// Validate, persist atomically, then signal reload — in that order.
/// `Config::save` already writes to a temporary file and renames, so a failure
/// before the rename leaves the previous file intact. The reload
/// signal is emitted only after that call returns, which is the point at which
/// the completed file is visible to the daemon.
pub fn save_and_notify(cfg: &Config, path: &Path) -> SaveOutcome {
    if let Err((field, err)) = validate_config(cfg) {
        return SaveOutcome::Rejected(field, err);
    }

    if let Err(e) = cfg.save(path) {
        return SaveOutcome::WriteFailed(e.to_string());
    }

    SaveOutcome::Saved {
        reload: signal_reload(),
    }
}

/// Signal the daemon to arm the shortcut-capture lease at `level`
/// (`shared::constants::CAPTURE_LEASE_NONE` / `_OBSERVE` / `_RECORD`), or
/// disarm it with `CAPTURE_LEASE_NONE`.
///
/// `lParam` carries this process's own id (`std::process::id()`), never a
/// window handle — the daemon's comparison is against a process id it reads
/// itself from `GetForegroundWindow`, and sending anything else here is
/// exactly the shape of `DEF-3`.
pub fn signal_capture_lease(level: usize) -> DaemonSignal {
    let class: Vec<u16> = DAEMON_WINDOW_CLASS
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let title: Vec<u16> = DAEMON_WINDOW_TITLE
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: `class` and `title` are NUL-terminated wide string locals that outlive the call.
    // `FindWindowW` returns 0 if the daemon is not running (handled cleanly). `lParam` carries
    // the current process ID as a plain integer (`isize`), crossing the process boundary safely.
    unsafe {
        let hwnd = FindWindowW(class.as_ptr(), title.as_ptr());
        if hwnd == 0 {
            return DaemonSignal::DaemonAbsent;
        }
        let lparam = std::process::id() as isize;
        if PostMessageW(hwnd, shared::constants::WM_APP_CAPTURE_LEASE, level, lparam) != 0 {
            DaemonSignal::Delivered
        } else {
            DaemonSignal::Refused
        }
    }
}

/// Post the frozen reload intent to the daemon's hidden window.
/// No configuration pointer crosses the process boundary — the data travels
/// through the completed TOML file, and this is only a "look again" nudge
/// There is deliberately no file watcher and no polling.
pub fn signal_reload() -> DaemonSignal {
    let class: Vec<u16> = DAEMON_WINDOW_CLASS
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let title: Vec<u16> = DAEMON_WINDOW_TITLE
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: both buffers are NUL-terminated locals that outlive the call, and neither Win32
    // call retains them. `FindWindowW` returns 0 when the daemon is not running, which is
    // handled as a normal outcome rather than an error.
    //
    // This crosses a process boundary, so what is *not* passed matters as much as what is:
    // `wParam` and `lParam` are both zero, so no pointer from this process is handed to the
    // daemon — it would be meaningless in that address space. The configuration travels
    // through the completed TOML file, and this is only a nudge to re-read it, which is also
    // why a failed post is reported rather than retried.
    unsafe {
        let hwnd = FindWindowW(class.as_ptr(), title.as_ptr());
        if hwnd == 0 {
            // The daemon is not running. Not an error: it will read the file
            // when it next starts.
            return DaemonSignal::DaemonAbsent;
        }
        if PostMessageW(hwnd, WM_APP_RELOAD_CONFIG, 0, 0) != 0 {
            DaemonSignal::Delivered
        } else {
            // The window exists and the post was dropped. UIPI is the reachable cause:
            // this process is below the daemon's integrity level.
            DaemonSignal::Refused
        }
    }
}

/// First-run launch state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchIntent {
    /// No configuration exists — start the onboarding tutorial.
    Onboarding,
    /// A configuration exists — start the normal Settings window.
    Settings,
}

/// The frozen launch contract argument for first run.
/// Re-exported from `shared` rather than redeclared: the daemon produces this
/// flag and Settings consumes it, so a divergence must be a compile error, not
/// an onboarding screen that silently never appears.
pub use shared::ONBOARDING_FLAG;

/// Decide the launch intent from the presence of a configuration file.
pub fn launch_intent(path: &Path) -> LaunchIntent {
    if path.exists() {
        LaunchIntent::Settings
    } else {
        LaunchIntent::Onboarding
    }
}

/// Resolve the launch intent from process arguments and disk state.
/// An explicit `--onboarding` wins so the tutorial can be replayed on demand.
pub fn resolve_launch_intent<I: IntoIterator<Item = String>>(args: I) -> LaunchIntent {
    if args.into_iter().any(|a| a == ONBOARDING_FLAG) {
        return LaunchIntent::Onboarding;
    }
    launch_intent(&config_path())
}

/// Complete onboarding by writing a valid configuration.
/// Both "finished the tutorial" and "Skip Tutorial" land here, which is what
/// stops onboarding from repeating unintentionally.
/// The shell currently reaches the same behaviour through `SettingsModel::save`;
/// this is the contract-level entry point used by the first-run tests.
#[allow(dead_code)]
pub fn complete_onboarding(cfg: &Config, path: &Path) -> SaveOutcome {
    save_and_notify(cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("wiradesk-settings-test-{}", std::process::id()));
        p
    }

    // ── : frozen defaults ─────────────────────────────────────────

    #[test]
    fn default_config_uses_frozen_shortcuts() {
        let cfg = Config::default();
        assert_eq!(cfg.switcher.shortcut, "win+backtick");
        assert_eq!(cfg.switcher.fallback_shortcut, "alt+backtick");
        assert!(cfg.switcher.shortcut_enabled);
        assert!(cfg.switcher.fallback_shortcut_enabled);
        assert_eq!(cfg.snapping.snap_half_left, "ctrl+alt+left");
        assert_eq!(cfg.snapping.snap_half_right, "ctrl+alt+right");
        assert_eq!(cfg.snapping.snap_half_top, "ctrl+alt+up");
        assert_eq!(cfg.snapping.snap_half_bottom, "ctrl+alt+down");
        assert_eq!(cfg.snapping.snap_maximize, "ctrl+alt+enter");
        assert_eq!(cfg.snapping.snap_percent_left, "ctrl+alt+shift+left");
        assert_eq!(cfg.snapping.snap_percent_right, "ctrl+alt+shift+right");
        assert_eq!(cfg.snapping.snap_percent_top, "ctrl+alt+shift+up");
        assert_eq!(cfg.snapping.snap_percent_bottom, "ctrl+alt+shift+down");
        assert_eq!(cfg.snapping.snap_third_left, "ctrl+alt+1");
        assert_eq!(cfg.snapping.snap_third_middle, "ctrl+alt+2");
        assert_eq!(cfg.snapping.snap_third_right, "ctrl+alt+3");
        assert!(cfg.snapping.snap_half_left_enabled);
        assert!(cfg.snapping.snap_half_right_enabled);
        assert!(cfg.snapping.snap_half_top_enabled);
        assert!(cfg.snapping.snap_half_bottom_enabled);
        assert!(cfg.snapping.snap_maximize_enabled);
        assert!(cfg.snapping.snap_percent_left_enabled);
        assert!(cfg.snapping.snap_percent_right_enabled);
        assert!(cfg.snapping.snap_percent_top_enabled);
        assert!(cfg.snapping.snap_percent_bottom_enabled);
        assert!(cfg.snapping.snap_third_left_enabled);
        assert!(cfg.snapping.snap_third_middle_enabled);
        assert!(cfg.snapping.snap_third_right_enabled);
        assert_eq!(cfg.snapping.percent_left, 50);
        assert_eq!(cfg.snapping.percent_right, 50);
        assert_eq!(cfg.snapping.percent_top, 50);
        assert_eq!(cfg.snapping.percent_bottom, 50);
        assert_eq!(
            cfg.layout.move_next_monitor_shortcut,
            "ctrl+alt+shift+enter"
        );
        assert!(cfg.layout.move_next_monitor_shortcut_enabled);
        assert_eq!(cfg.layout.stack_shortcut, "ctrl+alt+shift+s");
        assert!(cfg.layout.stack_shortcut_enabled);
        assert!(!cfg.general.auto_start);
        assert_eq!(cfg.layout.stack_width_percent, 50);
        assert!(!cfg.vm_bypass.bypass_processes.is_empty());
        assert!(!cfg.vm_bypass.bypass_classes.is_empty());
    }

    #[test]
    fn a_config_predating_the_enabled_field_loads_every_action_enabled() {
        let toml = r#"
            [switcher]
            shortcut = "win+backtick"
            fallback_shortcut = "alt+backtick"

            [snapping]
            snap_half_left = "ctrl+alt+left"
            snap_half_right = "ctrl+alt+right"
            snap_half_top = "ctrl+alt+up"
            snap_half_bottom = "ctrl+alt+down"
            snap_maximize = "ctrl+alt+enter"
            snap_percent_left = "ctrl+alt+shift+left"
            snap_percent_right = "ctrl+alt+shift+right"
            snap_percent_top = "ctrl+alt+shift+up"
            snap_percent_bottom = "ctrl+alt+shift+down"
            snap_third_left = "ctrl+alt+1"
            snap_third_middle = "ctrl+alt+2"
            snap_third_right = "ctrl+alt+3"
            percent_left = 50
            percent_right = 50
            percent_top = 50
            percent_bottom = 50

            [layout]
            stack_shortcut = "ctrl+alt+shift+s"
            move_next_monitor_shortcut = "ctrl+alt+shift+enter"
            stack_width_percent = 50
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert!(cfg.switcher.shortcut_enabled);
        assert!(cfg.switcher.fallback_shortcut_enabled);
        assert!(cfg.snapping.snap_half_left_enabled);
        assert!(cfg.snapping.snap_half_right_enabled);
        assert!(cfg.snapping.snap_half_top_enabled);
        assert!(cfg.snapping.snap_half_bottom_enabled);
        assert!(cfg.snapping.snap_maximize_enabled);
        assert!(cfg.snapping.snap_percent_left_enabled);
        assert!(cfg.snapping.snap_percent_right_enabled);
        assert!(cfg.snapping.snap_percent_top_enabled);
        assert!(cfg.snapping.snap_percent_bottom_enabled);
        assert!(cfg.snapping.snap_third_left_enabled);
        assert!(cfg.snapping.snap_third_middle_enabled);
        assert!(cfg.snapping.snap_third_right_enabled);
        assert!(cfg.layout.move_next_monitor_shortcut_enabled);
        assert!(cfg.layout.stack_shortcut_enabled);
    }

    #[test]
    fn a_duplicate_names_the_holder_by_the_shared_declared_order() {
        // `LBR-ST-14`: one declared sequence decides which of two colliding actions is named
        // as the holder. This module walks its own table to find that holder, so its order is
        // part of that sequence — and until `DEC-018` it was a third, separately kept copy
        // that `DEC-014`'s reorder missed. The consequence is user-visible on the Save path:
        // Settings would refuse the save naming one action as the holder while the daemon's
        // precedence unbound the other.
        //
        // Asserted through the behaviour rather than by re-listing the order, because a test
        // that restates the sequence is itself another copy of it.
        for (earlier, later, set) in [
            (
                "snapping.snap_third_left",
                "snapping.snap_maximize",
                (|c: &mut Config, v: &str| {
                    c.snapping.snap_third_left = v.to_string();
                    c.snapping.snap_maximize = v.to_string();
                }) as fn(&mut Config, &str),
            ),
            (
                "snapping.snap_percent_left",
                "snapping.snap_maximize",
                |c: &mut Config, v: &str| {
                    c.snapping.snap_percent_left = v.to_string();
                    c.snapping.snap_maximize = v.to_string();
                },
            ),
            (
                "snapping.snap_percent_left",
                "layout.move_next_monitor_shortcut",
                |c: &mut Config, v: &str| {
                    c.snapping.snap_percent_left = v.to_string();
                    c.layout.move_next_monitor_shortcut = v.to_string();
                },
            ),
        ] {
            let mut cfg = Config::default();
            set(&mut cfg, "ctrl+alt+f9");
            let err = validate_config(&cfg).expect_err("a shared chord must be refused");
            assert_eq!(
                err.0, later,
                "the later action in the declared sequence is the one refused"
            );
            match err.1 {
                ShortcutError::DuplicateShortcut(holder) => assert_eq!(
                    holder, earlier,
                    "the earlier action in the declared sequence keeps the chord"
                ),
                other => panic!("expected a duplicate rejection, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_disabled_action_is_excluded_from_collision_detection() {
        let mut cfg = Config::default();
        cfg.switcher.shortcut = "ctrl+alt+left".to_string();
        cfg.snapping.snap_half_left = "ctrl+alt+left".to_string();

        assert_eq!(
            validate_config(&cfg),
            Err((
                "snapping.snap_half_left",
                ShortcutError::DuplicateShortcut("switcher.shortcut")
            ))
        );

        // Disabled field produces no collision
        cfg.snapping.snap_half_left_enabled = false;
        assert!(validate_config(&cfg).is_ok());

        // First field disabled: second registers normally
        cfg.snapping.snap_half_left_enabled = true;
        cfg.switcher.shortcut_enabled = false;
        assert!(validate_config(&cfg).is_ok());

        // Both disabled: no collision
        cfg.snapping.snap_half_left_enabled = false;
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn an_out_of_range_percentage_is_rejected_before_save() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("reject_pct.toml");

        let mut cfg = Config::default();
        cfg.snapping.percent_left = 0;
        assert_eq!(
            validate_config(&cfg),
            Err(("snapping.percent_left", ShortcutError::InvalidPercentage(0)))
        );
        assert!(matches!(
            save_and_notify(&cfg, &path),
            SaveOutcome::Rejected("snapping.percent_left", ShortcutError::InvalidPercentage(0))
        ));

        cfg.snapping.percent_left = 50;
        cfg.snapping.percent_right = 100;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "snapping.percent_right",
                ShortcutError::InvalidPercentage(100)
            ))
        );

        cfg.snapping.percent_right = 50;
        cfg.snapping.percent_top = 150;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "snapping.percent_top",
                ShortcutError::InvalidPercentage(150)
            ))
        );
    }

    #[test]
    fn an_out_of_range_percentage_typed_then_saved_is_refused() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("reject_pct_typed.toml");

        let mut cfg = Config::default();
        cfg.snapping.percent_left = 0;
        assert_eq!(
            validate_config(&cfg),
            Err(("snapping.percent_left", ShortcutError::InvalidPercentage(0)))
        );
        assert!(matches!(
            save_and_notify(&cfg, &path),
            SaveOutcome::Rejected("snapping.percent_left", ShortcutError::InvalidPercentage(0))
        ));

        cfg.snapping.percent_left = 50;
        cfg.snapping.percent_bottom = 100;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "snapping.percent_bottom",
                ShortcutError::InvalidPercentage(100)
            ))
        );

        cfg.snapping.percent_bottom = 50;
        cfg.layout.stack_width_percent = 0;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "layout.stack_width_percent",
                ShortcutError::InvalidPercentage(0)
            ))
        );
        assert!(matches!(
            save_and_notify(&cfg, &path),
            SaveOutcome::Rejected(
                "layout.stack_width_percent",
                ShortcutError::InvalidPercentage(0)
            )
        ));

        cfg.layout.stack_width_percent = 5;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "layout.stack_width_percent",
                ShortcutError::InvalidPercentage(5)
            ))
        );

        cfg.layout.stack_width_percent = 105;
        assert_eq!(
            validate_config(&cfg),
            Err((
                "layout.stack_width_percent",
                ShortcutError::InvalidPercentage(105)
            ))
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn stack_width_percent_round_trips_through_the_shortcut_row_path() {
        // The Overlapping Stack row edits `layout.stack_width_percent` through the same
        // `ShortcutField` percentage plumbing every `Snap to custom` row uses. That path has a
        // catch-all arm, so a field left out of it does not fail to compile — it accepts the edit
        // and discards it, drawing a control that silently does nothing. This asserts the value
        // actually reaches the config and survives a serialise/parse round trip.
        let mut cfg = Config::default();
        crate::app::ShortcutField::Stack.set_percent(&mut cfg, 42);
        assert_eq!(
            cfg.layout.stack_width_percent, 42,
            "the shortcut-row percentage path must reach `layout.stack_width_percent`"
        );
        assert_eq!(
            crate::app::ShortcutField::Stack.percent(&cfg),
            Some(42),
            "and must read back what it wrote"
        );

        // Round-tripped through the real save path rather than a serde call, so this covers what
        // a user's Save actually does to the file on disk.
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("stack-width-round-trip.toml");
        assert!(
            matches!(save_and_notify(&cfg, &path), SaveOutcome::Saved { .. }),
            "42 is inside the stack range, so the save must be accepted"
        );
        let reloaded = Config::from_toml_str(&std::fs::read_to_string(&path).unwrap())
            .expect("the saved file parses back");
        assert_eq!(reloaded.layout.stack_width_percent, 42);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn default_config_passes_its_own_validation() {
        assert!(validate_config(&Config::default()).is_ok());
    }

    // ── : validation before replacement ───────────────────────────

    #[test]
    fn valid_shortcut_returns_canonical_form() {
        assert_eq!(
            validate_shortcut(" WIN + Backtick ").unwrap(),
            "win+backtick"
        );
        assert_eq!(validate_shortcut("shift+ctrl+a").unwrap(), "ctrl+shift+a");
    }

    #[test]
    fn unsupported_token_is_reported_as_such() {
        assert_eq!(
            validate_shortcut("win+notarealkey"),
            Err(ShortcutError::UnsupportedToken)
        );
    }

    #[test]
    fn modifier_only_is_reported_as_no_main_key() {
        assert_eq!(validate_shortcut("ctrl+win"), Err(ShortcutError::NoMainKey));
        assert_eq!(validate_shortcut(""), Err(ShortcutError::NoMainKey));
        assert_eq!(validate_shortcut("   "), Err(ShortcutError::NoMainKey));
    }

    #[test]
    fn bare_main_key_without_a_modifier_is_rejected() {
        assert_eq!(validate_shortcut("a"), Err(ShortcutError::NoModifier));
        assert_eq!(
            validate_shortcut("backtick"),
            Err(ShortcutError::NoModifier)
        );
    }

    #[test]
    fn two_main_keys_are_reported_distinctly() {
        assert_eq!(
            validate_shortcut("ctrl+a+b"),
            Err(ShortcutError::MultipleMainKeys)
        );
        assert_eq!(
            validate_shortcut("win+left+right"),
            Err(ShortcutError::MultipleMainKeys)
        );
    }

    #[test]
    fn an_invalid_field_names_itself() {
        let mut cfg = Config::default();
        cfg.snapping.snap_maximize = "ctrl+win".to_string();
        assert_eq!(
            validate_config(&cfg),
            Err(("snapping.snap_maximize", ShortcutError::NoMainKey))
        );
    }

    #[test]
    fn rejection_leaves_the_previous_file_intact() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("reject.toml");

        let good = Config::default();
        assert!(matches!(
            save_and_notify(&good, &path),
            SaveOutcome::Saved { .. }
        ));
        let before = std::fs::read_to_string(&path).unwrap();

        let mut bad = Config::default();
        bad.switcher.shortcut = "win+nonsense".to_string();
        assert!(matches!(
            save_and_notify(&bad, &path),
            SaveOutcome::Rejected("switcher.shortcut", ShortcutError::UnsupportedToken)
        ));

        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after, "a rejected save modified the file");
        let _ = std::fs::remove_file(&path);
    }

    // ── : lossless round-trip ─────────────────────────────────────

    #[test]
    fn saved_config_round_trips_without_loss() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.toml");

        let mut cfg = Config::default();
        cfg.general.auto_start = true;
        cfg.snapping.snap_third_left_enabled = false;
        cfg.layout.stack_shortcut_enabled = false;
        cfg.layout.stack_width_percent = 70;
        cfg.vm_bypass.bypass_classes.push("CustomClass".to_string());

        assert!(matches!(
            save_and_notify(&cfg, &path),
            SaveOutcome::Saved { .. }
        ));
        let loaded = Config::load_or_default(&path);
        assert_eq!(loaded, cfg);
        let _ = std::fs::remove_file(&path);
    }

    // ── : first-run contract ──────────────────────────────────────

    #[test]
    fn missing_config_selects_onboarding() {
        let dir = temp_dir();
        let missing = dir.join("definitely-not-here.toml");
        let _ = std::fs::remove_file(&missing);
        assert_eq!(launch_intent(&missing), LaunchIntent::Onboarding);
    }

    #[test]
    fn existing_config_selects_settings() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("exists.toml");
        Config::default().save(&path).unwrap();
        assert_eq!(launch_intent(&path), LaunchIntent::Settings);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn explicit_flag_forces_onboarding() {
        assert_eq!(
            resolve_launch_intent(vec![
                "wiradesk-settings.exe".to_string(),
                ONBOARDING_FLAG.to_string()
            ]),
            LaunchIntent::Onboarding
        );
    }

    #[test]
    fn onboarding_flag_is_the_frozen_spelling() {
        assert_eq!(ONBOARDING_FLAG, "--onboarding");
    }

    #[test]
    fn completing_onboarding_writes_a_valid_config_so_it_does_not_repeat() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("onboarding.toml");
        let _ = std::fs::remove_file(&path);

        assert_eq!(launch_intent(&path), LaunchIntent::Onboarding);
        assert!(matches!(
            complete_onboarding(&Config::default(), &path),
            SaveOutcome::Saved { .. }
        ));
        assert_eq!(launch_intent(&path), LaunchIntent::Settings);
        let _ = std::fs::remove_file(&path);
    }

    // ── : IPC shape ───────────────────────────────────────────────

    #[test]
    fn reload_signal_is_harmless_when_no_daemon_is_running() {
        // Returns false rather than erroring: the daemon will read the file
        // when it next starts.
        let _ = signal_reload();
    }

    #[test]
    fn reload_uses_the_frozen_message_identifier() {
        assert_eq!(WM_APP_RELOAD_CONFIG, 0x8000 + 1);
    }
}
