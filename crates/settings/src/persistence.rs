//! Settings-side persistence and IPC contract with the daemon.
//! Ordering is the whole point of this module: validate before replacing,
//! replace atomically, and signal reload **only after** the completed file is
//! visible. Any other order can leave the daemon reading a half-written or
//! stale configuration.

use std::path::Path;

use shared::constants::{DAEMON_WINDOW_CLASS, DAEMON_WINDOW_TITLE, WM_APP_RELOAD_CONFIG};
use shared::{config_path, Config, Shortcut};

use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW};

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

/// Validate every shortcut field in a candidate configuration.
/// Returns the offending field name and reason on the first failure, leaving
/// the caller's active configuration untouched.
pub fn validate_config(cfg: &Config) -> Result<(), (&'static str, ShortcutError)> {
    let fields: [(&'static str, &str); 6] = [
        ("switcher.shortcut", &cfg.switcher.shortcut),
        (
            "switcher.fallback_shortcut",
            &cfg.switcher.fallback_shortcut,
        ),
        ("snapping.snap_half_left", &cfg.snapping.snap_half_left),
        ("snapping.snap_half_right", &cfg.snapping.snap_half_right),
        ("snapping.snap_maximize", &cfg.snapping.snap_maximize),
        ("layout.stack_shortcut", &cfg.layout.stack_shortcut),
    ];
    for (name, value) in fields {
        validate_shortcut(value).map_err(|e| (name, e))?;
    }
    Ok(())
}

/// Outcome of a save attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved { reload_signalled: bool },
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
        reload_signalled: signal_reload(),
    }
}

/// Post the frozen reload intent to the daemon's hidden window.
/// No configuration pointer crosses the process boundary — the data travels
/// through the completed TOML file, and this is only a "look again" nudge
/// There is deliberately no file watcher and no polling.
pub fn signal_reload() -> bool {
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
            return false;
        }
        PostMessageW(hwnd, WM_APP_RELOAD_CONFIG, 0, 0) != 0
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
        assert_eq!(cfg.snapping.snap_half_left, "ctrl+win+left");
        assert_eq!(cfg.snapping.snap_half_right, "ctrl+win+right");
        assert_eq!(cfg.snapping.snap_maximize, "ctrl+win+enter");
        assert_eq!(cfg.layout.stack_shortcut, "ctrl+win+down");
        assert!(!cfg.general.auto_start);
        assert!(!cfg.layout.enable_overlapping_stack);
        assert_eq!(cfg.layout.stack_width_percent, 50);
        assert!(!cfg.vm_bypass.bypass_processes.is_empty());
        assert!(!cfg.vm_bypass.bypass_classes.is_empty());
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
        cfg.layout.enable_overlapping_stack = true;
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
