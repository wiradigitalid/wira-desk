//! Embeds `wiradesk-settings.rc` — icon and version metadata — into the Settings
//! binary.
//!
//! Unlike the daemon there is no manifest here and no skip switch, and both follow from
//! the same fact: Settings does not request elevation. The daemon needs
//! `WIRADESK_SKIP_MANIFEST` because its `requireAdministrator` manifest applies to every
//! target of the crate, including the test harness, which then cannot launch. Nothing in
//! this resource script affects how a process starts, so the Settings tests are
//! unaffected and the resources are always embedded.
//!
//! # Why the outcome is checked
//! There is no manifest here, but the same silent-failure shape applies to what there is:
//! a resource compiler that is missing or that fails would produce a successful build of a
//! binary with no icon and no version metadata, and nothing would report it. The version
//! test in `main.rs` compares the resource script against `Cargo.toml`, which keeps the two
//! *texts* honest — it cannot know whether the script was ever compiled in.
//! `manifest_required()` is used despite its name because the requirement being asserted is
//! that the resource compiled at all, and `NotAttempted` is a failure here for that reason.

/// Reported on every path, and consumed by `main.rs` via `env!` so a path that forgets to
/// set it is a compile error rather than a silent gap.
fn report(state: &str) {
    println!("cargo:rustc-env=WIRADESK_SETTINGS_RESOURCE_STATE={state}");
}

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        report("not-windows");
        return;
    }

    println!("cargo:rerun-if-changed=wiradesk-settings.rc");
    // The icon lives in the daemon crate and is shared rather than duplicated, so a
    // change there must rebuild this resource too.
    println!("cargo:rerun-if-changed=../daemon/wiradesk.ico");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let rc_path = std::path::Path::new(&manifest_dir).join("wiradesk-settings.rc");

    if let Err(e) = embed_resource::compile(rc_path, embed_resource::NONE).manifest_required() {
        panic!(
            "failed to embed wiradesk-settings.rc: {e:?}\n\
             Without it the Settings binary ships with no icon and no version metadata, and \
             nothing at runtime would report the omission."
        );
    }
    report("embedded");
}
