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
    pub snap_maximize: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Enable overlapping stack layout for small screens (P2 feature).
    pub enable_overlapping_stack: bool,
    /// Width of each window as a percentage of screen width (default 50).
    pub stack_width_percent: u32,
    /// Overlapping stack shortcut. Introduced by the frozen contract with the
    /// reviewable default `ctrl+win+down`; consumers may use it but must not
    /// renumber or reinterpret this field.
    pub stack_shortcut: String,
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
            snap_half_left: "ctrl+win+left".to_string(),
            snap_half_right: "ctrl+win+right".to_string(),
            snap_maximize: "ctrl+win+enter".to_string(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            enable_overlapping_stack: false,
            stack_width_percent: 50,
            stack_shortcut: "ctrl+win+down".to_string(),
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

    #[test]
    fn frozen_snapping_defaults() {
        let cfg = SnappingConfig::default();
        assert_eq!(cfg.snap_half_left, "ctrl+win+left");
        assert_eq!(cfg.snap_half_right, "ctrl+win+right");
        assert_eq!(cfg.snap_maximize, "ctrl+win+enter");
    }

    #[test]
    fn frozen_stack_shortcut_default() {
        assert_eq!(LayoutConfig::default().stack_shortcut, "ctrl+win+down");
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
        assert_eq!(cfg.layout.stack_shortcut, "ctrl+win+down");
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
