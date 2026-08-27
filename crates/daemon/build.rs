//! Embeds `wiradesk.rc` — icon, version info, and the `requireAdministrator`
//! manifest — into the daemon binary.
//! # Why the manifest can be skipped
//! A build script's output applies to every target of the crate, and all of the
//! daemon's tests live in the `wiradesk` bin target, so the test harness was
//! linked with the same `requireAdministrator` manifest as the daemon. Cargo
//! launches that harness like any other process, so `cargo test -p daemon`
//! aborted before running a single test:
//! ```text
//! could not execute process ... wiradesk-<hash>.exe (never executed)
//! The requested operation requires elevation. (os error 740)
//! ```
//! The suite could only be run from an Administrator shell, and never in CI.
//! Nothing under test needs elevation — they are unit tests plus a few
//! read-only desktop queries. Only the daemon does, to install a low-level
//! keyboard hook.
//! Setting `WIRADESK_SKIP_MANIFEST=1` omits the manifest so the harness can
//! launch. It is safe against misuse: `main.rs` checks the process token at
//! startup and refuses to run unelevated regardless of what the manifest says,
//! so a binary built this way fails loudly instead of misbehaving quietly.
//!
//! Note that the switch skips the whole resource script, so a binary built with it
//! also carries no icon and no version metadata. That is acceptable because the
//! switch exists for the test harness, and `build.ps1` and the release gate both
//! build without it — but it is the reason the name understates what is dropped.
//!
//! # Why the outcome is checked rather than assumed
//! Until `embed-resource` 3.0 this script called `compile` and discarded the result,
//! because there was no result to discard: the 2.x signature returned nothing. So a
//! resource compiler that was missing or that failed produced a **successful build of a
//! binary with no manifest**, and nothing anywhere would say so. That failure is silent
//! all the way to the user's machine — tests pass because they are built with
//! `WIRADESK_SKIP_MANIFEST` set, clippy is clean, the release build succeeds, CI is
//! green, and the daemon then refuses to start because it cannot elevate.
//!
//! That is not hypothetical. A release binary was built manifest-less during this
//! project's own release verification, and it was caught by reading bytes out of the
//! `.exe` by hand rather than by any check.
//!
//! Two things close it. `manifest_required()` turns a missing or failed resource
//! compilation into a build failure — `NotAttempted` counts as failure here, because a
//! daemon without its manifest cannot do the one thing it exists to do. And every path
//! through this script reports what it did in `WIRADESK_RESOURCE_STATE`, which `main.rs`
//! reads with `env!`, so a path that returns without embedding *and* without saying so
//! fails to compile the crate.

/// Reported on every path, and consumed by `main.rs` via `env!` so that adding a path
/// which forgets to set it is a compile error rather than a silent gap.
fn report(state: &str) {
    println!("cargo:rustc-env=WIRADESK_RESOURCE_STATE={state}");
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

    println!("cargo:rerun-if-changed=wiradesk.rc");
    println!("cargo:rerun-if-changed=wiradesk.manifest");
    println!("cargo:rerun-if-env-changed=WIRADESK_SKIP_MANIFEST");
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

    if std::env::var_os("WIRADESK_SKIP_MANIFEST").is_some() {
        // Loud on purpose: this must never pass unnoticed in a real build.
        println!(
            "cargo:warning=WIRADESK_SKIP_MANIFEST is set - building without the \
             elevation manifest. Test-only; the daemon will refuse to start."
        );
        report("skipped");
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let rc_path = std::path::Path::new(&manifest_dir).join("wiradesk.rc");

    // Panicking here is the point: the alternative is shipping a daemon that cannot
    // elevate, which fails at the user rather than at the build.
    if let Err(e) = embed_resource::compile(rc_path, version_macros()).manifest_required() {
        panic!(
            "failed to embed wiradesk.rc: {e:?}\n\
             The daemon requires its requireAdministrator manifest to install a low-level \
             keyboard hook, and refuses to start without it. Building on without the \
             resource would produce a binary that fails on the user's machine instead of \
             here. If you are building for a test harness, set WIRADESK_SKIP_MANIFEST=1 \
             deliberately."
        );
    }
    report("embedded");
}
