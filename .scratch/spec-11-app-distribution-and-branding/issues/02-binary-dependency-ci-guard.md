---
id: SPEC-11-02
component: window-management
satisfies: []
blocked_by: [SPEC-11-01]
status: done
tests:
  - shared::binary::tests::pe_import_scanner_detects_dynamic_msvc_imports
  - shared::binary::tests::pe_import_scanner_passes_clean_static_binary
---

# 02: Release binary dependency scanner and CI regression guard

**What to build:** Create an automated verification script `scripts/verify-release-binary.ps1` that scans Windows PE import tables of release executables to ensure no dynamic CRT dependencies (`VCRUNTIME*.dll`, `MSVCP*.dll`) are imported, backed by pure in-memory unit tests in `crates/shared/src/binary.rs`. Integrate this gate into `.github/workflows/release.yml`. Additionally, update release note generation in `release.yml` with a concrete `WINGET_PACKAGE_LIVE` variable to accurately govern `winget install` instructions.

**Blocked by:** SPEC-11-01

## Acceptance Criteria

- [ ] In `crates/shared/src/binary.rs`:
      - Implement pure PE import scanner logic that inspects PE headers in-memory with unit tests:
        - `shared::binary::tests::pe_import_scanner_detects_dynamic_msvc_imports`
        - `shared::binary::tests::pe_import_scanner_passes_clean_static_binary`
- [ ] Create `scripts/verify-release-binary.ps1`:
      - Accepts `-Path` (directory or binary files) and optional `-AllowDynamicCrtIfBundled` switch (for the fallback branch).
      - Verifies that target binaries were written after the script invocation began (preventing stale artifact false positives).
      - Inspects PE import tables; fails with non-zero exit code if dynamic CRT imports are detected without the bundled fallback switch.
      - If `-AllowDynamicCrtIfBundled` is passed, verifies that the installer script bundles `vcredist_x64.exe` and that the binary exists in staging.
- [ ] In `.github/workflows/release.yml`:
      - Add a build verification step after compilation and before packaging executing `scripts/verify-release-binary.ps1`.
      - In the release note generation step, check repository variable `vars.WINGET_PACKAGE_LIVE`:
        - If `'true'`, render:
          ```markdown
          ### Install via WinGet
          ```powershell
          winget install WiraDigitalIndonesia.WiraDesk
          ```
          ```
        - Else, render:
          ```markdown
          ### Install via WinGet (Pending Approval)
          *WinGet package submission pending official approval on microsoft/winget-pkgs (PR #426321).*
          ```
