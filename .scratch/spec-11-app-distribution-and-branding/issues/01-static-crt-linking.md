---
id: SPEC-11-01
component: window-management
satisfies: []
blocked_by: []
status: done
tests:
  - shared::tests::msvc_target_compiles_with_crt_static
---

# 01: Static C-Runtime linking via `.cargo/config.toml`

**What to build:** Configure the Cargo workspace build environment to link the Microsoft Visual C++ runtime statically (`target-feature=+crt-static`) for the `x86_64-pc-windows-msvc` compilation target. Ensure both `wiradesk.exe` (daemon) and `wiradesk-settings.exe` (settings UI) compile as standalone executables without external dynamic dependencies on `VCRUNTIME140.dll` or `MSVCP140.dll`.

**Blocked by:** none

## Acceptance Criteria

- [ ] Create `.cargo/config.toml` in the repository root with:
      ```toml
      [target.x86_64-pc-windows-msvc]
      rustflags = ["-C", "target-feature=+crt-static"]
      ```
- [ ] Compile release binaries (`cargo build --release`) for `crates/daemon` and `crates/settings`.
- [ ] **Skia Linkage Compatibility & Fallback Rule:**
      - Verify that `crates/settings` (which pulls in Slint's C++ `renderer-skia`) links successfully with `+crt-static`.
      - If unresolvable linker conflicts occur with Skia C++ runtime (`LNK2038`), activate the documented fallback: bundle `Microsoft.VCRedist.2015+.x64.exe` in `installer/wiradesk.iss` with silent install flags (`/install /quiet /norestart`) and declare `Microsoft.VCRedist.2015+.x64` in the WinGet manifest.
- [ ] Binary inspection of fresh release executables (validated by timestamp) confirms zero dynamic imports of `VCRUNTIME*.dll` and `MSVCP*.dll` on the primary static linking path (or proper bundling under the fallback path).
- [ ] Compile-time unit test in `crates/shared/src/lib.rs`:
      - `shared::tests::msvc_target_compiles_with_crt_static`: Asserts `cfg!(target_feature = "crt-static")` when compiled for the `windows-msvc` target family.
- [ ] Workspace verification: `cargo test --workspace` (with `$env:WIRADESK_SKIP_MANIFEST = '1'`) and `cargo clippy --workspace --all-targets -- -D warnings` pass cleanly.
- [ ] Manual Owner Verification Checklist: Prior to public release tag creation, owner verifies installation and launch of the installer package on a clean Windows VM without pre-installed VC++ Redistributables.
