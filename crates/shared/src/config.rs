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
    pub mouse: MouseConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Auto-start on Windows boot (Task Scheduler highest privileges).
    pub auto_start: bool,
    /// Check for a newer release periodically. **Defaults to `true`**, by the owner's
    /// decision, taken against the alternative of asking once during onboarding.
    ///
    /// This is the only setting in this product that causes a network request, and
    /// `PRIVACY.md` describes that request line by line rather than summarising it: what is
    /// sent, what an IP address reveals anyway, and where this switch is. Turning it off
    /// stops the periodic check entirely; the manual button in Settings stays, so a user can
    /// ask once without leaving anything running.
    pub check_updates: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            check_updates: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwitcherConfig {
    /// Primary same-app switcher shortcut (e.g. "win+backtick").
    pub shortcut: String,
    pub shortcut_enabled: bool,
    /// Fallback shortcut (e.g. "alt+backtick").
    pub fallback_shortcut: String,
    pub fallback_shortcut_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SnappingConfig {
    pub snap_half_left: String,
    pub snap_half_left_enabled: bool,
    pub snap_half_right: String,
    pub snap_half_right_enabled: bool,
    /// Top half. Added after the original freeze; legacy config without it keeps
    /// every value it holds and gains this default, because every field on this
    /// struct carries `#[serde(default)]`.
    pub snap_half_top: String,
    pub snap_half_top_enabled: bool,
    /// Bottom half, the complement of `snap_half_top`.
    pub snap_half_bottom: String,
    pub snap_half_bottom_enabled: bool,
    pub snap_maximize: String,
    pub snap_maximize_enabled: bool,
    /// Custom percentage snap against the left edge.
    pub snap_percent_left: String,
    pub snap_percent_left_enabled: bool,
    /// Custom percentage snap against the right edge.
    pub snap_percent_right: String,
    pub snap_percent_right_enabled: bool,
    /// Custom percentage snap against the top edge.
    pub snap_percent_top: String,
    pub snap_percent_top_enabled: bool,
    /// Custom percentage snap against the bottom edge.
    pub snap_percent_bottom: String,
    pub snap_percent_bottom_enabled: bool,
    /// Snap the active window to the left third of the work area.
    pub snap_third_left: String,
    pub snap_third_left_enabled: bool,
    /// Snap the active window to the middle third of the work area.
    pub snap_third_middle: String,
    pub snap_third_middle_enabled: bool,
    /// Snap the active window to the right third of the work area.
    pub snap_third_right: String,
    pub snap_third_right_enabled: bool,
    /// Percentage of work-area width for left-edge snap (default 50).
    pub percent_left: u32,
    /// Percentage of work-area width for right-edge snap (default 50).
    pub percent_right: u32,
    /// Percentage of work-area height for top-edge snap (default 50).
    pub percent_top: u32,
    /// Percentage of work-area height for bottom-edge snap (default 50).
    pub percent_bottom: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "LayoutConfigDe")]
pub struct LayoutConfig {
    /// Width of each window as a percentage of screen width (default 50).
    pub stack_width_percent: u32,
    /// Overlapping stack shortcut. The field name is part of the frozen contract and must
    /// not be renumbered or reinterpreted; its *default* moved to `ctrl+alt+shift+s` (DEC-011)
    /// to free the arrow tier for custom-percentage edge snaps.
    pub stack_shortcut: String,
    pub stack_shortcut_enabled: bool,
    /// Move the active window to the next monitor. Lives in `[layout]` rather than
    /// `[snapping]` because it arranges *across* screens rather than dividing one, which
    /// keeps the three config sections mapping one-to-one onto the three groups the
    /// Shortcuts pane draws.
    pub move_next_monitor_shortcut: String,
    pub move_next_monitor_shortcut_enabled: bool,
}

#[derive(Deserialize)]
#[serde(default)]
struct LayoutConfigDe {
    stack_width_percent: u32,
    stack_shortcut: String,
    stack_shortcut_enabled: Option<bool>,
    move_next_monitor_shortcut: String,
    move_next_monitor_shortcut_enabled: bool,
    enable_overlapping_stack: Option<bool>,
}

impl Default for LayoutConfigDe {
    fn default() -> Self {
        let def = LayoutConfig::default();
        Self {
            stack_width_percent: def.stack_width_percent,
            stack_shortcut: def.stack_shortcut,
            stack_shortcut_enabled: None,
            move_next_monitor_shortcut: def.move_next_monitor_shortcut,
            move_next_monitor_shortcut_enabled: def.move_next_monitor_shortcut_enabled,
            enable_overlapping_stack: None,
        }
    }
}

impl From<LayoutConfigDe> for LayoutConfig {
    fn from(de: LayoutConfigDe) -> Self {
        let stack_shortcut_enabled = match (de.stack_shortcut_enabled, de.enable_overlapping_stack)
        {
            (Some(enabled), _) => enabled,
            (None, Some(old)) => old,
            (None, None) => true,
        };
        LayoutConfig {
            stack_width_percent: de.stack_width_percent,
            stack_shortcut: de.stack_shortcut,
            stack_shortcut_enabled,
            move_next_monitor_shortcut: de.move_next_monitor_shortcut,
            move_next_monitor_shortcut_enabled: de.move_next_monitor_shortcut_enabled,
        }
    }
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

/// Curated action presets for driverless mouse auxiliary navigation (CAP-17, SPEC-8, SPEC-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseActionPreset {
    #[serde(rename = "next_virtual_desktop")]
    NextVirtualDesktop,
    #[serde(rename = "prev_virtual_desktop")]
    PrevVirtualDesktop,
    #[serde(rename = "task_view")]
    TaskView,
    #[serde(rename = "show_desktop")]
    ShowDesktop,
    #[serde(rename = "cycle_forward")]
    CycleForward,
    #[serde(rename = "snap_left")]
    SnapLeft,
    #[serde(rename = "snap_right")]
    SnapRight,
    #[serde(rename = "snap_top")]
    SnapTop,
    #[serde(rename = "snap_bottom")]
    SnapBottom,
    #[serde(rename = "snap_third_left")]
    SnapThirdLeft,
    #[serde(rename = "snap_third_center")]
    SnapThirdCenter,
    #[serde(rename = "snap_third_right")]
    SnapThirdRight,
    #[serde(rename = "snap_percent_left")]
    SnapPercentLeft,
    #[serde(rename = "snap_percent_right")]
    SnapPercentRight,
    #[serde(rename = "snap_percent_top")]
    SnapPercentTop,
    #[serde(rename = "snap_percent_bottom")]
    SnapPercentBottom,
    #[serde(rename = "maximize")]
    Maximize,
    #[serde(rename = "overlapping_stack")]
    OverlappingStack,
    #[serde(rename = "move_next_monitor")]
    MoveNextMonitor,
    #[serde(rename = "passthrough")]
    Passthrough,
}

impl MouseActionPreset {
    pub const ALL: [MouseActionPreset; 20] = [
        MouseActionPreset::NextVirtualDesktop,
        MouseActionPreset::PrevVirtualDesktop,
        MouseActionPreset::TaskView,
        MouseActionPreset::ShowDesktop,
        MouseActionPreset::CycleForward,
        MouseActionPreset::SnapLeft,
        MouseActionPreset::SnapRight,
        MouseActionPreset::SnapTop,
        MouseActionPreset::SnapBottom,
        MouseActionPreset::SnapThirdLeft,
        MouseActionPreset::SnapThirdCenter,
        MouseActionPreset::SnapThirdRight,
        MouseActionPreset::SnapPercentLeft,
        MouseActionPreset::SnapPercentRight,
        MouseActionPreset::SnapPercentTop,
        MouseActionPreset::SnapPercentBottom,
        MouseActionPreset::Maximize,
        MouseActionPreset::OverlappingStack,
        MouseActionPreset::MoveNextMonitor,
        MouseActionPreset::Passthrough,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NextVirtualDesktop => "next_virtual_desktop",
            Self::PrevVirtualDesktop => "prev_virtual_desktop",
            Self::TaskView => "task_view",
            Self::ShowDesktop => "show_desktop",
            Self::CycleForward => "cycle_forward",
            Self::SnapLeft => "snap_left",
            Self::SnapRight => "snap_right",
            Self::SnapTop => "snap_top",
            Self::SnapBottom => "snap_bottom",
            Self::SnapThirdLeft => "snap_third_left",
            Self::SnapThirdCenter => "snap_third_center",
            Self::SnapThirdRight => "snap_third_right",
            Self::SnapPercentLeft => "snap_percent_left",
            Self::SnapPercentRight => "snap_percent_right",
            Self::SnapPercentTop => "snap_percent_top",
            Self::SnapPercentBottom => "snap_percent_bottom",
            Self::Maximize => "maximize",
            Self::OverlappingStack => "overlapping_stack",
            Self::MoveNextMonitor => "move_next_monitor",
            Self::Passthrough => "passthrough",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Self::NextVirtualDesktop => "Next Virtual Desktop",
            Self::PrevVirtualDesktop => "Previous Virtual Desktop",
            Self::TaskView => "Task View",
            Self::ShowDesktop => "Show Desktop",
            Self::CycleForward => "Cycle Same-App Window Forward",
            Self::SnapLeft => "Snap Window Left",
            Self::SnapRight => "Snap Window Right",
            Self::SnapTop => "Snap Window Top",
            Self::SnapBottom => "Snap Window Bottom",
            Self::SnapThirdLeft => "Snap Left Third",
            Self::SnapThirdCenter => "Snap Center Third",
            Self::SnapThirdRight => "Snap Right Third",
            Self::SnapPercentLeft => "Snap Custom % Left",
            Self::SnapPercentRight => "Snap Custom % Right",
            Self::SnapPercentTop => "Snap Custom % Top",
            Self::SnapPercentBottom => "Snap Custom % Bottom",
            Self::Maximize => "Maximize Window",
            Self::OverlappingStack => "Overlapping Stack",
            Self::MoveNextMonitor => "Move to Next Monitor",
            Self::Passthrough => "Default / Passthrough",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::NextVirtualDesktop | Self::PrevVirtualDesktop => "Virtual Desktops",
            Self::TaskView | Self::ShowDesktop => "Windows Shell",
            Self::CycleForward => "Switching",
            Self::SnapLeft | Self::SnapRight | Self::SnapTop | Self::SnapBottom => "Snap to Half",
            Self::SnapThirdLeft | Self::SnapThirdCenter | Self::SnapThirdRight => "Snap to Third",
            Self::SnapPercentLeft
            | Self::SnapPercentRight
            | Self::SnapPercentTop
            | Self::SnapPercentBottom => "Snap to Custom",
            Self::Maximize | Self::OverlappingStack | Self::MoveNextMonitor => "Arrange & Move",
            Self::Passthrough => "Passthrough",
        }
    }

    pub fn parse_slug(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("default") {
            return Some(Self::Passthrough);
        }
        Self::ALL
            .iter()
            .copied()
            .find(|preset| s.eq_ignore_ascii_case(preset.as_str()))
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|p| p == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Option<Self> {
        Self::ALL.get(i).copied()
    }
}

impl std::str::FromStr for MouseActionPreset {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_slug(s).ok_or_else(|| format!("unknown mouse action preset: {s}"))
    }
}

impl std::fmt::Display for MouseActionPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Auxiliary mouse navigation configuration (SPEC-8, FR-32).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    /// Enable auxiliary mouse navigation.
    pub enabled: bool,
    /// Physical XBUTTON1 (Back) preset action slug.
    pub thumb_back: String,
    /// Physical XBUTTON2 (Forward) preset action slug.
    pub thumb_forward: String,
    /// Horizontal wheel left tilt preset action slug.
    pub tilt_left: String,
    /// Horizontal wheel right tilt preset action slug.
    pub tilt_right: String,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thumb_back: "prev_virtual_desktop".to_string(),
            thumb_forward: "next_virtual_desktop".to_string(),
            tilt_left: "show_desktop".to_string(),
            tilt_right: "task_view".to_string(),
        }
    }
}

// ── Defaults (on-disk config.toml schema) ─────────────────────────────────

impl Default for SwitcherConfig {
    fn default() -> Self {
        Self {
            shortcut: "win+backtick".to_string(),
            shortcut_enabled: true,
            fallback_shortcut: "alt+backtick".to_string(),
            fallback_shortcut_enabled: true,
        }
    }
}

impl Default for SnappingConfig {
    fn default() -> Self {
        Self {
            snap_half_left: "ctrl+alt+left".to_string(),
            snap_half_left_enabled: true,
            snap_half_right: "ctrl+alt+right".to_string(),
            snap_half_right_enabled: true,
            snap_half_top: "ctrl+alt+up".to_string(),
            snap_half_top_enabled: true,
            snap_half_bottom: "ctrl+alt+down".to_string(),
            snap_half_bottom_enabled: true,
            snap_maximize: "ctrl+alt+enter".to_string(),
            snap_maximize_enabled: true,
            snap_percent_left: "ctrl+alt+shift+left".to_string(),
            snap_percent_left_enabled: true,
            snap_percent_right: "ctrl+alt+shift+right".to_string(),
            snap_percent_right_enabled: true,
            snap_percent_top: "ctrl+alt+shift+up".to_string(),
            snap_percent_top_enabled: true,
            snap_percent_bottom: "ctrl+alt+shift+down".to_string(),
            snap_percent_bottom_enabled: true,
            snap_third_left: "ctrl+alt+1".to_string(),
            snap_third_left_enabled: true,
            snap_third_middle: "ctrl+alt+2".to_string(),
            snap_third_middle_enabled: true,
            snap_third_right: "ctrl+alt+3".to_string(),
            snap_third_right_enabled: true,
            percent_left: crate::constants::DEFAULT_SNAP_PERCENT,
            percent_right: crate::constants::DEFAULT_SNAP_PERCENT,
            percent_top: crate::constants::DEFAULT_SNAP_PERCENT,
            percent_bottom: crate::constants::DEFAULT_SNAP_PERCENT,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            stack_width_percent: crate::constants::DEFAULT_STACK_WIDTH_PERCENT,
            stack_shortcut: "ctrl+alt+shift+s".to_string(),
            stack_shortcut_enabled: true,
            move_next_monitor_shortcut: "ctrl+alt+shift+enter".to_string(),
            move_next_monitor_shortcut_enabled: true,
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

    #[test]
    fn percent_snap_fields_roundtrip_through_toml() {
        let mut cfg = Config::default();
        cfg.snapping.snap_percent_left = "ctrl+alt+shift+left".to_string();
        cfg.snapping.snap_percent_right = "ctrl+alt+shift+right".to_string();
        cfg.snapping.snap_percent_top = "ctrl+alt+shift+up".to_string();
        cfg.snapping.snap_percent_bottom = "ctrl+alt+shift+down".to_string();
        cfg.snapping.percent_left = 70;
        cfg.snapping.percent_right = 30;
        cfg.snapping.percent_top = 25;
        cfg.snapping.percent_bottom = 75;

        let toml = cfg.to_toml_string().unwrap();
        let parsed = Config::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.snapping.snap_percent_left, "ctrl+alt+shift+left");
        assert_eq!(parsed.snapping.snap_percent_right, "ctrl+alt+shift+right");
        assert_eq!(parsed.snapping.snap_percent_top, "ctrl+alt+shift+up");
        assert_eq!(parsed.snapping.snap_percent_bottom, "ctrl+alt+shift+down");
        assert_eq!(parsed.snapping.percent_left, 70);
        assert_eq!(parsed.snapping.percent_right, 30);
        assert_eq!(parsed.snapping.percent_top, 25);
        assert_eq!(parsed.snapping.percent_bottom, 75);
    }

    #[test]
    fn third_snap_fields_roundtrip_through_toml() {
        let mut cfg = Config::default();
        cfg.snapping.snap_third_left = "ctrl+alt+shift+1".to_string();
        cfg.snapping.snap_third_middle = "ctrl+alt+shift+2".to_string();
        cfg.snapping.snap_third_right = "ctrl+alt+shift+3".to_string();

        let toml = cfg.to_toml_string().unwrap();
        let parsed = Config::from_toml_str(&toml).unwrap();
        assert_eq!(parsed.snapping.snap_third_left, "ctrl+alt+shift+1");
        assert_eq!(parsed.snapping.snap_third_middle, "ctrl+alt+shift+2");
        assert_eq!(parsed.snapping.snap_third_right, "ctrl+alt+shift+3");
    }

    #[test]
    fn action_enabled_fields_roundtrip_through_toml() {
        let mut cfg = Config::default();
        cfg.switcher.shortcut_enabled = false;
        cfg.switcher.fallback_shortcut_enabled = false;
        cfg.snapping.snap_half_left_enabled = false;
        cfg.snapping.snap_half_right_enabled = false;
        cfg.snapping.snap_half_top_enabled = false;
        cfg.snapping.snap_half_bottom_enabled = false;
        cfg.snapping.snap_maximize_enabled = false;
        cfg.snapping.snap_third_left_enabled = false;
        cfg.snapping.snap_third_middle_enabled = false;
        cfg.snapping.snap_third_right_enabled = false;
        cfg.layout.move_next_monitor_shortcut_enabled = false;
        cfg.snapping.snap_percent_left_enabled = false;
        cfg.snapping.snap_percent_right_enabled = false;
        cfg.snapping.snap_percent_top_enabled = false;
        cfg.snapping.snap_percent_bottom_enabled = false;
        cfg.layout.stack_shortcut_enabled = false;

        let toml = cfg.to_toml_string().unwrap();
        let parsed = Config::from_toml_str(&toml).unwrap();
        assert_eq!(cfg, parsed);
        assert!(!parsed.switcher.shortcut_enabled);
        assert!(!parsed.switcher.fallback_shortcut_enabled);
        assert!(!parsed.snapping.snap_half_left_enabled);
        assert!(!parsed.snapping.snap_half_right_enabled);
        assert!(!parsed.snapping.snap_half_top_enabled);
        assert!(!parsed.snapping.snap_half_bottom_enabled);
        assert!(!parsed.snapping.snap_maximize_enabled);
        assert!(!parsed.snapping.snap_third_left_enabled);
        assert!(!parsed.snapping.snap_third_middle_enabled);
        assert!(!parsed.snapping.snap_third_right_enabled);
        assert!(!parsed.layout.move_next_monitor_shortcut_enabled);
        assert!(!parsed.snapping.snap_percent_left_enabled);
        assert!(!parsed.snapping.snap_percent_right_enabled);
        assert!(!parsed.snapping.snap_percent_top_enabled);
        assert!(!parsed.snapping.snap_percent_bottom_enabled);
        assert!(!parsed.layout.stack_shortcut_enabled);
    }

    #[test]
    fn legacy_config_with_enable_overlapping_stack_false_seeds_disabled() {
        let toml = r#"
            [layout]
            enable_overlapping_stack = false
            stack_width_percent = 60
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.layout.stack_width_percent, 60);
        assert!(!cfg.layout.stack_shortcut_enabled);

        // Next save drops the retired key
        let saved_toml = cfg.to_toml_string().unwrap();
        assert!(!saved_toml.contains("enable_overlapping_stack"));
    }

    #[test]
    fn legacy_config_with_enable_overlapping_stack_true_or_absent_loads_enabled() {
        let toml_true = r#"
            [layout]
            enable_overlapping_stack = true
        "#;
        let cfg_true = Config::from_toml_str(toml_true).unwrap();
        assert!(cfg_true.layout.stack_shortcut_enabled);

        let toml_absent = r#"
            [layout]
            stack_width_percent = 50
        "#;
        let cfg_absent = Config::from_toml_str(toml_absent).unwrap();
        assert!(cfg_absent.layout.stack_shortcut_enabled);
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
        assert_eq!(cfg.snap_percent_left, "ctrl+alt+shift+left");
        assert_eq!(cfg.snap_percent_right, "ctrl+alt+shift+right");
        assert_eq!(cfg.snap_percent_top, "ctrl+alt+shift+up");
        assert_eq!(cfg.snap_percent_bottom, "ctrl+alt+shift+down");
        assert_eq!(cfg.snap_third_left, "ctrl+alt+1");
        assert_eq!(cfg.snap_third_middle, "ctrl+alt+2");
        assert_eq!(cfg.snap_third_right, "ctrl+alt+3");
        assert!(cfg.snap_half_left_enabled);
        assert!(cfg.snap_half_right_enabled);
        assert!(cfg.snap_half_top_enabled);
        assert!(cfg.snap_half_bottom_enabled);
        assert!(cfg.snap_maximize_enabled);
        assert!(cfg.snap_percent_left_enabled);
        assert!(cfg.snap_percent_right_enabled);
        assert!(cfg.snap_percent_top_enabled);
        assert!(cfg.snap_percent_bottom_enabled);
        assert!(cfg.snap_third_left_enabled);
        assert!(cfg.snap_third_middle_enabled);
        assert!(cfg.snap_third_right_enabled);
        assert_eq!(cfg.percent_left, 50);
        assert_eq!(cfg.percent_right, 50);
        assert_eq!(cfg.percent_top, 50);
        assert_eq!(cfg.percent_bottom, 50);
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
            &cfg.snapping.snap_percent_left,
            &cfg.snapping.snap_percent_right,
            &cfg.snapping.snap_percent_top,
            &cfg.snapping.snap_percent_bottom,
            &cfg.snapping.snap_third_left,
            &cfg.snapping.snap_third_middle,
            &cfg.snapping.snap_third_right,
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
            &cfg.snapping.snap_percent_left,
            &cfg.snapping.snap_percent_right,
            &cfg.snapping.snap_percent_top,
            &cfg.snapping.snap_percent_bottom,
            &cfg.snapping.snap_third_left,
            &cfg.snapping.snap_third_middle,
            &cfg.snapping.snap_third_right,
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
        let layout = LayoutConfig::default();
        assert_eq!(layout.stack_shortcut, "ctrl+alt+shift+s");
        assert!(layout.stack_shortcut_enabled);
        assert_eq!(layout.move_next_monitor_shortcut, "ctrl+alt+shift+enter");
        assert!(layout.move_next_monitor_shortcut_enabled);
    }

    #[test]
    fn frozen_switcher_defaults() {
        let switcher = SwitcherConfig::default();
        assert_eq!(switcher.shortcut, "win+backtick");
        assert!(switcher.shortcut_enabled);
        assert_eq!(switcher.fallback_shortcut, "alt+backtick");
        assert!(switcher.fallback_shortcut_enabled);
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
        assert_eq!(cfg.layout.stack_shortcut, "ctrl+alt+shift+s");
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

    #[test]
    fn mouse_config_roundtrips_through_toml() {
        let mut cfg = Config::default();
        cfg.mouse.enabled = false;
        cfg.mouse.thumb_back = "cycle_forward".to_string();
        cfg.mouse.thumb_forward = "maximize".to_string();
        cfg.mouse.tilt_left = "snap_left".to_string();
        cfg.mouse.tilt_right = "snap_right".to_string();

        let toml = cfg.to_toml_string().unwrap();
        let parsed = Config::from_toml_str(&toml).unwrap();
        assert_eq!(cfg.mouse, parsed.mouse);
        assert!(!parsed.mouse.enabled);
        assert_eq!(parsed.mouse.thumb_back, "cycle_forward");
        assert_eq!(parsed.mouse.thumb_forward, "maximize");
        assert_eq!(parsed.mouse.tilt_left, "snap_left");
        assert_eq!(parsed.mouse.tilt_right, "snap_right");
    }

    #[test]
    fn missing_mouse_section_defaults_safely() {
        let toml = r#"
            [general]
            auto_start = true
        "#;
        let cfg = Config::from_toml_str(toml).unwrap();
        assert_eq!(cfg.mouse, MouseConfig::default());
        assert!(cfg.mouse.enabled);
        assert_eq!(cfg.mouse.thumb_back, "prev_virtual_desktop");
        assert_eq!(cfg.mouse.thumb_forward, "next_virtual_desktop");
        assert_eq!(cfg.mouse.tilt_left, "show_desktop");
        assert_eq!(cfg.mouse.tilt_right, "task_view");
    }

    #[test]
    fn default_tilt_directions_are_inverted() {
        let def = MouseConfig::default();
        assert_eq!(def.tilt_left, "show_desktop");
        assert_eq!(def.tilt_right, "task_view");
    }

    #[test]
    fn mouse_action_preset_slug_parsing() {
        assert_eq!(
            MouseActionPreset::parse_slug("next_virtual_desktop"),
            Some(MouseActionPreset::NextVirtualDesktop)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("prev_virtual_desktop"),
            Some(MouseActionPreset::PrevVirtualDesktop)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("task_view"),
            Some(MouseActionPreset::TaskView)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("show_desktop"),
            Some(MouseActionPreset::ShowDesktop)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("cycle_forward"),
            Some(MouseActionPreset::CycleForward)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("snap_left"),
            Some(MouseActionPreset::SnapLeft)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("snap_right"),
            Some(MouseActionPreset::SnapRight)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("maximize"),
            Some(MouseActionPreset::Maximize)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("passthrough"),
            Some(MouseActionPreset::Passthrough)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("default"),
            Some(MouseActionPreset::Passthrough)
        );
        assert_eq!(
            MouseActionPreset::parse_slug("  NEXT_VIRTUAL_DESKTOP "),
            Some(MouseActionPreset::NextVirtualDesktop)
        );
        assert_eq!(MouseActionPreset::parse_slug("unknown_preset"), None);
        assert_eq!(MouseActionPreset::parse_slug(""), None);

        // Verify roundtrip through as_str
        for preset in MouseActionPreset::ALL {
            assert_eq!(MouseActionPreset::parse_slug(preset.as_str()), Some(preset));
            assert!(!preset.display_label().is_empty());
        }
    }

    #[test]
    fn expanded_mouse_presets_roundtrip_and_parse() {
        assert_eq!(MouseActionPreset::ALL.len(), 20);

        let expected_categories = [
            "Virtual Desktops",
            "Windows Shell",
            "Switching",
            "Snap to Half",
            "Snap to Third",
            "Snap to Custom",
            "Arrange & Move",
            "Passthrough",
        ];

        for (i, preset) in MouseActionPreset::ALL.iter().enumerate() {
            assert_eq!(preset.index(), i);
            assert_eq!(MouseActionPreset::from_index(i), Some(*preset));
            assert_eq!(
                MouseActionPreset::parse_slug(preset.as_str()),
                Some(*preset)
            );
            assert!(!preset.display_label().is_empty());
            assert!(
                expected_categories.contains(&preset.category()),
                "unexpected category {} for {:?}",
                preset.category(),
                preset
            );

            // Serialization roundtrip
            let serialized = serde_json::to_string(preset).unwrap();
            let deserialized: MouseActionPreset = serde_json::from_str(&serialized).unwrap();
            assert_eq!(*preset, deserialized);
        }

        assert_eq!(MouseActionPreset::from_index(20), None);
        assert_eq!(MouseActionPreset::parse_slug("invalid_preset_xyz"), None);
    }
}
