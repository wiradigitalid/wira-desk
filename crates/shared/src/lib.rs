//! `shared` crate — types and constants shared between `daemon` and `settings`.
//! Single source of truth for `Config`, the `u8` command enum, `%APPDATA%` paths,
//! and custom Win32 message IDs — preventing silent divergence between the two binaries.

pub mod commands;
pub mod config;
pub mod constants;
pub mod migrate;
pub mod shortcut;

pub use commands::Command;
pub use config::{
    app_data_dir, config_path, log_path, Config, GeneralConfig, LayoutConfig, SnappingConfig,
    SwitcherConfig, VmBypassConfig,
};
pub use constants::{ONBOARDING_FLAG, SETTINGS_BIN_NAME, SETTINGS_EXE_NAME};
pub use migrate::migrate_appdata;
pub use shortcut::{name_from_vk, vk_from_name, Shortcut};
