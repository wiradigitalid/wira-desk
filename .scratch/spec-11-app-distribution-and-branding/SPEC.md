# SPEC-11: Application Distribution, Static C-Runtime, and About Pane Branding

## Problem Statement

Wira Desk v0.2.0 encounters distribution, UI completeness, and security documentation blockers:
1. **Dynamic C-Runtime Linkage Blocker (WinGet Rejection):** Both compiled binaries (`wiradesk.exe` and `wiradesk-settings.exe`) dynamically link MSVC runtime libraries (`VCRUNTIME140.dll` and `MSVCP140.dll`). On clean Windows installations without pre-installed Visual C++ Redistributables, the executables crash with `STATUS_DLL_NOT_FOUND (0xC0000135)`. This caused Microsoft's automated package validation bot on `microsoft/winget-pkgs#426321` to reject the submission.
2. **Untracked Binary Import Drift & Premature WinGet Release Notes:** The CI release workflow lacks an automated gate ensuring that release binaries are standalone. In addition, release notes prematurely print `winget install WiraDigitalIndonesia.WiraDesk` while PR #426321 is still pending approval, with no tracked mechanism to toggle between pending and live states.
3. **Incomplete About Pane Application Surface:** The Settings About pane (`crates/settings/ui/panes/about_pane.slint`) displays version and typeface, but lacks official publisher attribution (Wira Digital Indonesia), links to the source repository and issue tracker, license disclosure, and a developer support navigation entry point.
4. **Network Boundary Inconsistency:** `SECURITY.md` and `docs/threat-model.md` claim "no network path" exists in the source. However, the application contains an updater module (`WinHttp` in `crates/daemon/src/updatecheck.rs` and `crates/settings/src/update.rs`) which performs two specific HTTPS requests to GitHub: fetching version metadata (`latest.json`) and downloading installer binaries upon explicit user confirmation. This discrepancy undermines user trust in elevated software.

## Solution Architecture

1. **Static MSVC C-Runtime Linking (`+crt-static`):**
   Configure `.cargo/config.toml` to inject `rustflags = ["-C", "target-feature=+crt-static"]` for the `x86_64-pc-windows-msvc` target. This statically links CRT routines into both executables, producing standalone binaries for bare Windows installations.
   * **Skia Linking Risk & Fallback Branch:** `crates/settings` pulls in Slint's native C++ `renderer-skia`. If `+crt-static` encounters unresolvable linker conflicts with Skia (`LNK2038 RuntimeLibrary mismatch`), an explicit fallback branch is activated: bundle `Microsoft.VCRedist.2015+.x64.exe` in `installer/wiradesk.iss` with silent install flags (`/install /quiet /norestart`), declare `Microsoft.VCRedist.2015+.x64` in the WinGet manifest, and adapt the CI binary guard accordingly.
2. **Automated Binary Dependency Gate & Configurable WinGet Release Notes:**
   Implement `scripts/verify-release-binary.ps1` (backed by pure in-memory PE parser unit tests in `crates/shared/src/binary.rs`) and integrate it into `.github/workflows/release.yml`. The gate scans release executables, verifies non-stale file timestamps, and fails if dynamic MSVC CRT imports exist (unless the bundled fallback flag is active). For release notes, wire a repository variable `WINGET_PACKAGE_LIVE` (default: false) that prints pending status notice until set to true.
3. **About Pane Metadata & Strict URL Navigation:**
   Update `about_pane.slint` and `crates/settings/src/main.rs` to render publisher identity (*"An open-source utility by Wira Digital Indonesia"* linking to `https://wiradigital.id`), repository & issue links (`https://github.com/wiradigitalid/wira-desk`), license information (`GPL-3.0`), and a *"Support development"* button navigating to `https://wiradigital.id/wira-desk`.
   * **Strict Navigation Security:** Re-use and extend `crates/settings/src/update.rs::open_in_browser`. Ensure strict path prefix validation over HTTPS: only allow exact URLs `https://wiradigital.id`, `https://wiradigital.id/wira-desk`, or URLs starting with `https://github.com/wiradigitalid/wira-desk/`. Disallow arbitrary subpaths or wildcards. All navigation uses `ShellExecuteW` with documented `SAFETY:` comments.
   * **URL Availability:** The canonical URL `https://wiradigital.id/wira-desk` is retained per owner confirmation. A pre-release checklist confirms HTTP 200 availability before release tags are published.
4. **Transparent Security & Full Network Boundary Alignment:**
   Reconcile `SECURITY.md`, `docs/threat-model.md`, and `README.md` to accurately document both outbound network paths: (1) update check request to GitHub for `latest.json`, and (2) installer download request to GitHub Releases initiated only when the user clicks "Download and install". Document that zero telemetry or personal data is collected and that checks can be disabled in Settings. Standardize the honest offline badge across surfaces and verify with `scripts/verify-public-export.ps1`.

## User Stories & Scope

- **US-1 (Clean OS Execution):** As a Windows user with a clean OS installation, I can run `wiradesk.exe` and `wiradesk-settings.exe` without installing third-party VC++ runtime packages.
- **US-2 (Distribution Validation):** As a package maintainer, my release binaries pass WinGet automated execution validation without DLL dependency errors, and release notes accurately track package approval state.
- **US-3 (About Pane Transparency):** As an application user, I can open the About pane in Settings to verify publisher credentials, inspect open-source licensing, navigate to the issue tracker, and access official project pages through safe browser links.
- **US-4 (Informed Security Trust):** As a security-conscious administrator inspecting elevated software, I can read accurate documentation detailing the full network boundary, updater mechanism, and privacy posture.
