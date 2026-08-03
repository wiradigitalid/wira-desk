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

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        println!("cargo:rerun-if-changed=wiradesk.rc");
        println!("cargo:rerun-if-changed=wiradesk.manifest");
        println!("cargo:rerun-if-env-changed=WIRADESK_SKIP_MANIFEST");

        if std::env::var_os("WIRADESK_SKIP_MANIFEST").is_some() {
            // Loud on purpose: this must never pass unnoticed in a real build.
            println!(
                "cargo:warning=WIRADESK_SKIP_MANIFEST is set - building without the \
                 elevation manifest. Test-only; the daemon will refuse to start."
            );
            return;
        }

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let rc_path = std::path::Path::new(&manifest_dir).join("wiradesk.rc");
        embed_resource::compile(rc_path, embed_resource::NONE);
    }
}
