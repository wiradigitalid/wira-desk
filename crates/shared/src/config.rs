//! TOML configuration schema and loader.
//! Shared configuration model consumed by daemon and Settings.
//! Field names and defaults match the on-disk `config.toml` schema.
//! Config lives at `%APPDATA%\WiraDesk\config.toml`. All fields have defaults
//! via `#[serde(default)]` so partial or missing config still loads.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::constants::{APP_DIR_NAME, CONFIG_FILE_NAME, LOG_FILE_NAME};

/// Wira Desk root configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub switcher: SwitcherConfig,
    pub snapping: SnappingConfig,
    pub layout: LayoutConfig,
    pub vm_bypass: VmBypassConfig,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Auto-start on Windows boot (Task Scheduler highest privileges).
    pub auto_start: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwitcherConfig {
    /// Primary same-app switcher shortcut (e.g. "win+backtick").
    pub shortcut: String,
    /// Fallback shortcut (e.g. "alt+backtick").
    pub fallback_shortcut: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SnappingConfig {
    pub snap_half_left: String,
    pub snap_half_right: String,
    /// Top half. Added after the original freeze; legacy config without it keeps
    /// every value it holds and gains this default, because every field on this
    /// struct carries `#[serde(default)]`.
    pub snap_half_top: String,
    /// Bottom half, the complement of `snap_half_top`.
    pub snap_half_bottom: String,
    pub snap_maximize: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Enable the overlapping stack arrangement. **Defaults to `true`.**
    ///
    /// It used to default to `false`, described as a P2 feature, and that was the wrong
    /// shape for what the flag actually gates: it guards `plan_stack` and nothing else,
    /// so `false` does not mean "the feature is off" in any way a user can perceive — it
    /// means pressing the stack shortcut produces an empty plan and **nothing happens, with
    /// no explanation**. A chord that silently does nothing reads as broken, not as
    /// disabled.
    ///
    /// `true` changes nothing passively either. The arrangement only ever runs when the
    /// shortcut is pressed, so switching the default on cannot surprise anyone who does
    /// not press it.
    pub enable_overlapping_stack: bool,
    /// Width of each window as a percentage of screen width (default 50).
    pub stack_width_percent: u32,
    /// Overlapping stack shortcut. The field name is part of the frozen contract and must
    /// not be renumbered or reinterpreted; its *default* moved to `ctrl+alt+shift+down` when
    /// the chord family moved, because `ctrl+alt+down` became the bottom-half snap.
    pub stack_shortcut: String,
    /// Move the active window to the next monitor. Lives in `[layout]` rather than
    /// `[snapping]` because it arranges *across* screens rather than dividing one, which
    /// keeps the three config sections mapping one-to-one onto the three groups the
    /// Shortcuts pane draws.
    pub move_next_monitor_shortcut: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VmBypassConfig {
    /// Process names whose active window causes Wira Desk to pass input through unchanged.
    pub bypass_processes: Vec<String>,
    /// Window classes with the same bypass effect. Added by the frozen contract
    /// in a backward-compatible way: legacy config that only contains
    /// `bypass_processes` remains valid and receives this default.
    /// Process and class identifiers are independently configurable.
    pub bypass_classes: Vec<String>,
}

// ── Defaults (on-disk config.toml schema) ─────────────────────────────────

impl Default for SwitcherConfig {
    fn default() -> Self {
        Self {
            shortcut: "win+backtick".to_string(),
            fallback_shortcut: "alt+backtick".to_string(),
        }
    }
}

impl Default for SnappingConfig {
    fn default() -> Self {
        Self {
            snap_half_left: "ctrl+alt+left".to_string(),
            snap_half_right: "ctrl+alt+right".to_string(),
            snap_half_top: "ctrl+alt+up".to_string(),
            snap_half_bottom: "ctrl+alt+down".to_string(),
            snap_maximize: "ctrl+alt+enter".to_string(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            enable_overlapping_stack: true,
            stack_width_percent: 50,
            stack_shortcut: "ctrl+alt+shift+down".to_string(),
            move_next_monitor_shortcut: "ctrl+alt+shift+enter".to_string(),
        }
    }
}

impl Default for VmBypassConfig {
    fn default() -> Self {
        Self {
            bypass_processes: vec![
                "mstsc.exe".to_string(),
                "vmconnect.exe".to_string(),
                "vmware.exe".to_string(),
                "VirtualBoxVM.exe".to_string(),
                "MobaXterm.exe".to_string(),
            ],
            bypass_classes: vec!["VMwareUnityWindow".to_string()],
        }
    }
}

impl Config {
    /// Deserialize from a TOML string. Missing fields use defaults.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serialize to a pretty TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load config from path. If the file is missing or fails to parse,
    /// return defaults (fail-safe — the daemon must not crash on bad config).
    pub fn load_or_default(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => Self::from_toml_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Write config to path atomically (write to a temp file then rename)
    /// so the daemon never reads a half-written file.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let toml = self
            .to_toml_string()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, toml.as_bytes())?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

// ── Path helpers (%APPDATA%\WiraDesk\...) ────────────────────────────────────

/// Application data directory: `%APPDATA%\WiraDesk`. Falls back to the working
/// directory when `APPDATA` is unavailable (very rare scenario).
pub fn app_data_dir() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join(APP_DIR_NAME),
        None => PathBuf::from(".").join(APP_DIR_NAME),
    }
}

/// Full path to `config.toml`.
pub fn config_path() -> PathBuf {
    app_data_dir().join(CONFIG_FILE_NAME)
}

/// Full path to `wiradesk.log`.
pub fn log_path() -> PathBuf {
    app_data_dir().join(LOG_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrips_through_toml() {
        let cfg = Config::default();
        let toml = cfg.to_toml_string().unwrap();
        let parsed = Config::from_toml_str(&toml).unwrap();
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let toml = r#"
            [switcher]
            shortcut = "alt+tab"
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.switcher.shortcut, "alt+tab");
        // fallback remains default
        assert_eq!(cfg.switcher.fallback_shortcut, "alt+backtick");
        // other sections remain default
        assert_eq!(cfg.layout.stack_width_percent, 50);
        assert!(!cfg.vm_bypass.bypass_processes.is_empty());
    }

    #[test]
    fn empty_toml_is_full_default() {
        let cfg = Config::from_toml_str("").unwrap();
        assert_eq!(cfg, Config::default());
    }

    // ── frozen extension contract ─────────────────────────────────
    // Epics 3, 4, and 5 consume these as sibling lanes. They may read them but
    // must not renumber or reinterpret them, so the values are pinned here.

    /// The shipped chord family, pinned so it cannot drift unnoticed.
    ///
    /// These values changed once, and the fact that this test had to be edited is the point
    /// of pinning them. `Win+Ctrl+Left/Right` is Windows' own virtual-desktop navigation, and
    /// the previous default took it silently because the low-level hook sees the chord first.
    #[test]
    fn frozen_snapping_defaults() {
        let cfg = SnappingConfig::default();
        assert_eq!(cfg.snap_half_left, "ctrl+alt+left");
        assert_eq!(cfg.snap_half_right, "ctrl+alt+right");
        assert_eq!(cfg.snap_half_top, "ctrl+alt+up");
        assert_eq!(cfg.snap_half_bottom, "ctrl+alt+down");
        assert_eq!(cfg.snap_maximize, "ctrl+alt+enter");
    }

    #[test]
    fn no_shipped_default_is_a_reserved_chord() {
        // A default that fails its own validation is the exact carve-out problem the reserved
        // catalogue exists to avoid, and it is how `ctrl+win+left` shipped while
        // `Win+Ctrl+Left` was a Windows shell chord.
        let cfg = Config::default();
        for raw in [
            &cfg.switcher.shortcut,
            &cfg.snapping.snap_half_left,
            &cfg.snapping.snap_half_right,
            &cfg.snapping.snap_half_top,
            &cfg.snapping.snap_half_bottom,
            &cfg.snapping.snap_maximize,
            &cfg.layout.move_next_monitor_shortcut,
            &cfg.layout.stack_shortcut,
        ] {
            let parsed = crate::Shortcut::parse(raw).expect("every shipped default parses");
            assert!(
                crate::shortcut::reservation(&parsed).is_none(),
                "shipped default {raw} is a reserved chord"
            );
        }
        // `switcher.fallback_shortcut` is deliberately excluded: `alt+backtick` is carved out
        // by name in `DEC-003` as the product's own identity, and it is the one default the
        // catalogue is allowed to disagree with.
    }

    #[test]
    fn every_shipped_default_is_distinct() {
        // The guard for the collision `DEC-009` handles at runtime: it must never be the
        // shipped configuration that produces one.
        let cfg = Config::default();
        let all = [
            &cfg.switcher.shortcut,
            &cfg.switcher.fallback_shortcut,
            &cfg.snapping.snap_half_left,
            &cfg.snapping.snap_half_right,
            &cfg.snapping.snap_half_top,
            &cfg.snapping.snap_half_bottom,
            &cfg.snapping.snap_maximize,
            &cfg.layout.move_next_monitor_shortcut,
            &cfg.layout.stack_shortcut,
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                let a = crate::Shortcut::parse(all[i]).expect("parses");
                let b = crate::Shortcut::parse(all[j]).expect("parses");
                assert_ne!(a, b, "{} and {} are the same chord", all[i], all[j]);
            }
        }
    }

    #[test]
    fn legacy_config_without_the_vertical_halves_still_loads() {
        // A file written before the top and bottom halves existed must keep every value
        // it holds and gain only the two new defaults.
        let toml = r#"
            [snapping]
            snap_half_left = "ctrl+alt+left"
            snap_half_right = "ctrl+alt+right"
            snap_maximize = "ctrl+alt+enter"
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.snapping.snap_half_left, "ctrl+alt+left");
        assert_eq!(cfg.snapping.snap_half_right, "ctrl+alt+right");
        assert_eq!(cfg.snapping.snap_maximize, "ctrl+alt+enter");
        assert_eq!(cfg.snapping.snap_half_top, "ctrl+alt+up");
        assert_eq!(cfg.snapping.snap_half_bottom, "ctrl+alt+down");
    }

    #[test]
    fn frozen_stack_shortcut_default() {
        assert_eq!(
            LayoutConfig::default().stack_shortcut,
            "ctrl+alt+shift+down"
        );
        assert_eq!(
            LayoutConfig::default().move_next_monitor_shortcut,
            "ctrl+alt+shift+enter"
        );
    }

    #[test]
    fn frozen_bypass_process_defaults() {
        assert_eq!(
            VmBypassConfig::default().bypass_processes,
            vec![
                "mstsc.exe",
                "vmconnect.exe",
                "vmware.exe",
                "VirtualBoxVM.exe",
                "MobaXterm.exe",
            ]
        );
    }

    #[test]
    fn frozen_bypass_class_default() {
        assert_eq!(
            VmBypassConfig::default().bypass_classes,
            vec!["VMwareUnityWindow"]
        );
    }

    #[test]
    fn legacy_config_without_bypass_classes_still_loads() {
        // A config written before the freeze must keep its process
        // entries and silently gain the documented class default.
        let toml = r#"
            [vm_bypass]
            bypass_processes = ["mstsc.exe", "custom.exe"]
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(
            cfg.vm_bypass.bypass_processes,
            vec!["mstsc.exe", "custom.exe"]
        );
        assert_eq!(cfg.vm_bypass.bypass_classes, vec!["VMwareUnityWindow"]);
    }

    #[test]
    fn legacy_config_without_stack_shortcut_still_loads() {
        let toml = r#"
            [layout]
            stack_width_percent = 70
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.layout.stack_width_percent, 70);
        assert_eq!(cfg.layout.stack_shortcut, "ctrl+alt+shift+down");
        assert_eq!(
            cfg.layout.move_next_monitor_shortcut,
            "ctrl+alt+shift+enter"
        );
    }

    #[test]
    fn process_and_class_identifiers_are_independently_configurable() {
        let toml = r#"
            [vm_bypass]
            bypass_processes = ["only.exe"]
            bypass_classes = ["OnlyClass", "SecondClass"]
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.vm_bypass.bypass_processes, vec!["only.exe"]);
        assert_eq!(
            cfg.vm_bypass.bypass_classes,
            vec!["OnlyClass", "SecondClass"]
        );
    }
}
