//! Embeds `wiradesk-settings.rc` — icon and version metadata — into the Settings
//! binary.
//!
//! Unlike the daemon there is no manifest here and no skip switch, and both follow from
//! the same fact: Settings does not request elevation. The daemon needs
//! `WIRADESK_SKIP_MANIFEST` because its `requireAdministrator` manifest applies to every
//! target of the crate, including the test harness, which then cannot launch. Nothing in
//! this resource script affects how a process starts, so the Settings tests are
//! unaffected and the resources are always embedded.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        println!("cargo:rerun-if-changed=wiradesk-settings.rc");
        // The icon lives in the daemon crate and is shared rather than duplicated, so a
        // change there must rebuild this resource too.
        println!("cargo:rerun-if-changed=../daemon/wiradesk.ico");

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let rc_path = std::path::Path::new(&manifest_dir).join("wiradesk-settings.rc");
        embed_resource::compile(rc_path, embed_resource::NONE);
    }
}
