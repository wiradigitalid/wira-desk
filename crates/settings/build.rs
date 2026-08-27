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

/// The version, as the preprocessor macros the resource script expects.
///
/// The `.rc` holds no version literal; it receives these. `CARGO_PKG_VERSION` comes from
/// `[workspace.package]` in the root manifest, so there is one place to edit and no
/// duplicate to keep honest.
///
/// Passed as separate digits rather than one comma-joined value on purpose: a macro
/// definition carrying commas has to survive a command line into the resource compiler,
/// and this sidesteps the question entirely. Any pre-release suffix is trimmed off the
/// patch digit, because `FILEVERSION` takes four integers and `0.2.0-beta` would
/// otherwise reach the compiler as the integer `0-beta`.
fn version_macros() -> Vec<String> {
    let version = std::env::var("CARGO_PKG_VERSION").expect("cargo always sets this");
    let mut digits = version.split('.');
    let mut next = |name: &str| -> String {
        let raw = digits.next().unwrap_or("0");
        let numeric = raw.split(['-', '+']).next().unwrap_or("0");
        if numeric.is_empty() || !numeric.bytes().all(|b| b.is_ascii_digit()) {
            panic!("CARGO_PKG_VERSION {version:?} has a non-numeric {name}: {raw:?}");
        }
        numeric.to_owned()
    };
    let major = next("major");
    let minor = next("minor");
    let patch = next("patch");
    vec![
        format!("WD_MAJOR={major}"),
        format!("WD_MINOR={minor}"),
        format!("WD_PATCH={patch}"),
        format!("WD_VERSION={version}"),
    ]
}

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        report("not-windows");
        return;
    }

    println!("cargo:rerun-if-changed=wiradesk-settings.rc");
    // The version comes from `[workspace.package]` in the ROOT manifest, so that file is
    // what this script depends on. `cargo:rerun-if-env-changed=CARGO_PKG_VERSION` was tried
    // first and is NOT a reliable trigger: cargo injects that value rather than reading it
    // from the environment, so the check compares "unset" against "unset" and never fires.
    //
    // The failure it left behind was one-directional and therefore easy to miss. Raising
    // the version rebuilt correctly, because a new version also rewrites `Cargo.lock` and
    // that dirties the crate. LOWERING it did not: the manifest said one version while the
    // embedded resource kept another, `cargo build` reported success, and the only place
    // the disagreement showed was the properties dialog of the built binary. CI is immune
    // because it always builds from a clean checkout, which is exactly why this would have
    // survived as a local-only trap.
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    // The icon lives in the daemon crate and is shared rather than duplicated, so a
    // change there must rebuild this resource too.
    println!("cargo:rerun-if-changed=../daemon/wiradesk.ico");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let rc_path = std::path::Path::new(&manifest_dir).join("wiradesk-settings.rc");

    if let Err(e) = embed_resource::compile(rc_path, version_macros()).manifest_required() {
        panic!(
            "failed to embed wiradesk-settings.rc: {e:?}\n\
             Without it the Settings binary ships with no icon and no version metadata, and \
             nothing at runtime would report the omission."
        );
    }

    let slint_ui_path = std::path::Path::new(&manifest_dir).join("ui/main_window.slint");
    slint_build::compile(slint_ui_path).expect("Slint build failed");

    report("embedded");
}
