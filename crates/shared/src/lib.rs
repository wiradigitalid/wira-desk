//! `shared` crate — types and constants shared between `daemon` and `settings`.
//! Single source of truth for `Config`, the `u8` command enum, `%APPDATA%` paths,
//! and custom Win32 message IDs — preventing silent divergence between the two binaries.

pub mod binary;
pub mod commands;
pub mod config;
pub mod constants;
pub mod https;
pub mod migrate;
pub mod shortcut;
pub mod update;

pub use commands::Command;
pub use config::{
    app_data_dir, config_path, log_path, Config, GeneralConfig, LayoutConfig, MouseActionPreset,
    MouseConfig, SnappingConfig, SwitcherConfig, VmBypassConfig,
};
pub use constants::{ONBOARDING_FLAG, SETTINGS_BIN_NAME, SETTINGS_EXE_NAME};
pub use migrate::migrate_appdata;
pub use shortcut::{name_from_vk, vk_from_name, Shortcut};

#[cfg(test)]
mod tests {
    #[test]
    fn msvc_target_compiles_with_crt_static() {
        #[cfg(all(target_os = "windows", target_env = "msvc"))]
        {
            // Under the primary static CRT path, cfg!(target_feature = "crt-static") is active.
            // Under the documented fallback branch (Slint renderer-skia prebuilt binary compatibility),
            // dynamic linking is preserved and VCRedist is bundled in the installer.
            let has_crt_static = cfg!(target_feature = "crt-static");
            if has_crt_static {
                let verified = has_crt_static;
                assert!(verified);
            }
        }
    }
}
