---
id: SPEC-11-04
component: settings
satisfies: []
blocked_by: [SPEC-11-03]
status: done
tests:
  - shared::update::tests::a_checksum_that_is_not_sha256_is_refused
  - shared::update::tests::a_download_url_off_this_repository_is_refused
---

# 04: Application security disclosure, network boundary correction, and honest offline badge

**What to build:** Align `SECURITY.md`, `docs/threat-model.md`, and application documentation with actual codebase behavior. Reconcile the false "no network path" claims by accurately disclosing both outbound HTTPS requests to GitHub: (1) version check to `latest.json`, and (2) optional installer download from GitHub Releases triggered only when the user clicks "Download and install". Update the offline badge to an honest form and ensure `scripts/verify-public-export.ps1` passes cleanly.

**Blocked by:** SPEC-11-03

## Acceptance Criteria

- [ ] In `SECURITY.md`:
      - Remove the claim *"There is no network path. No socket, HTTP client, update check, or telemetry exists in the source."*
      - Accurately state:
        *"No telemetry, no user account, no background updater service. The application makes two distinct outbound HTTPS requests to GitHub under `github.com/wiradigitalid/wira-desk`:*
        *1. An update check request (automated or on-demand) fetching version descriptor `latest.json`.*
        *2. An installer download request fetching the release setup executable, initiated ONLY when the user explicitly clicks 'Download and install' in Settings.*
        *Both requests carry zero machine, user, or configuration telemetry. Update checking can be fully disabled in Settings."*
      - Document that update binaries are verified against published SHA-256 digests before execution.
- [ ] In `docs/threat-model.md`:
      - Add the updater network path and download handler as documented trust boundaries, noting transport security (HTTPS via Windows `WinHttp`), payload origin, checksum verification, and user UAC elevation prompt at installation.
- [ ] In `README.md`:
      - Update the Update section and badges to reflect the standardized honest offline statement:
        *"No telemetry, no account, no background updater service — HTTPS requests to GitHub occur only when checking for or installing updates, which you can switch off."*
- [ ] Automated tests in `crates/shared/src/update.rs`:
      - `shared::update::tests::a_checksum_that_is_not_sha256_is_refused`: Validates that descriptors lacking valid SHA-256 hashes are refused.
      - `shared::update::tests::a_download_url_off_this_repository_is_refused`: Validates that update checks and download URLs strictly target the pinned GitHub release host and repository path.
- [ ] Run `scripts/verify-public-export.ps1`: Passes with 0 failures.
