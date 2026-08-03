## Deferred from: code review (1-1-core-daemon-and-cargo-workspace-setup)
- Entry point daemon menggunakan println! [crates/daemon/src/main.rs] — deferred, pre-existing (setup only)
- build.ps1 gagal jika CARGO_TARGET_DIR aktif [build.ps1:34] — deferred, pre-existing

- println! digunakan pada detached subsystem windows (crates/shared/src/lib.rs:2). Output diabaikan oleh Windows, tapi saat ini hanya digunakan untuk scaffolding.

## Deferred from: code review of 1-2-administrator-elevation-and-uipi-bypass.md (2026-07-09)
- Unconditional build dependency on embed-resource [crates/daemon/Cargo.toml:13] — deferred, pre-existing

## Deferred from: code review of 1-3-system-tray-resident-and-auto-recovery.md (2026-07-09)
- HICON=0 fallback [crates/daemon/src/tray.rs:196-206] — deferred to Story 1.5 (AD-7 Tier-1 startup-fatal protocol handles GDI exhaustion via MessageBox + exit).
- State-machine transition guards [crates/daemon/src/tray.rs:150-160] — deferred to Story 1.5: no guard against Critical→Warning downgrade (late warning would overwrite Tier-3 X); no Critical→Normal recovery message defined.
- NIM_MODIFY return in set_state [crates/daemon/src/tray.rs:104] — deferred to Story 1.5. Silent no-op if the icon was already removed by an Explorer restart; 1.5 will re-NIM_ADD if MODIFY returns FALSE.
- DPI awareness (WM_DPICHANGED) [crates/daemon/src/tray.rs] — deferred, cross-cutting. Icons fixed at 32×32; not blocking any AC.
## Deferred from: code review story-2.1 (2026-07-24)
- [x] [Review][Defer] Thread \health\ menggunakan \	hread::sleep(15s)\ statis sehingga cek \shutdown\ telat. Terselamatkan karena OS exit process, namun kurang elegan. [\crates/daemon/src/health.rs\:55] — deferred, pre-existing limitation
- [x] [Review][Defer] ModifierState desync jika keyup event terlewat (Win+L / UAC / Ctrl+Alt+Del) — deferred to Story 2.2 polish [crates/daemon/src/hook.rs:328]

## Deferred from: code review of story 4-4 (2026-08-02)
- [x] [Review][Defer] Pre-existing `Rect`/`Placement` public-field validation bypass, frozen since Story 4.1 [crates/daemon/src/arrangement/mod.rs:29-34,126-129] — deferred, out of scope for the 4.4 adapter shard
- [x] [Review][Defer] Raw-`HWND` TOCTOU/recycling hazard — an HWND resolved at one point can be reassigned to a different window before it's used [crates/daemon/src/arrangement/win32.rs:48-75,96-121] — deferred, inherent codebase-wide Win32 pattern, not specific to this adapter
- [x] [Review][Defer] Minimized-window edge case degrades to a silent no-op [crates/daemon/src/arrangement/win32.rs:39-58] — deferred, low reachability since minimizing removes foreground status
- [x] [Review][Defer] `apply_plan` doesn't dedupe repeated `WindowId`s within one plan [crates/daemon/src/arrangement/win32.rs:129-143] — deferred, no current planner produces a plan with duplicate targets

## Deferred from: code review of story 5-2 (2026-08-02)
- [x] [Review][Defer] `windows-sys` resolves to 5 different versions in `Cargo.lock` — deferred, transitive dependency graph issue not fixable from the settings crate alone
- [x] [Review][Defer] Registry-read failures for the OS theme collapse silently to Light with no diagnostic — deferred, low severity
- [x] [Review][Defer] No manual theme override/escape hatch exists if OS-theme detection misreads — deferred, not required by any AC

## Deferred from: code review of stories 5-3/5-4/5-5/5-6 (2026-08-02)
- [x] [Review][Defer] Enter/Space may be uncapturable as shortcut keys because egui's own keyboard-activation of the focused button may consume them first — deferred, needs real-window runtime verification, not reproducible in unit tests
- [x] [Review][Defer] Tab may be uncapturable as a shortcut key because egui's focus-navigation may consume it before the capturer sees it — deferred, same runtime-verification caveat as above
- [x] [Review][Defer] The "no config on disk → show normal Settings with defaults" path has no reachable route through the real binary — it always redirects to onboarding — deferred, needs product confirmation on whether that path should exist at all

## Deferred from: code review (2026-08-03)
- [x] [Review][Defer] Silently restoring from legacy WinTick folder on reset � deferred, pre-existing. Reason: Legacy migration is temporary (remove in v0.3.0); rollback safety is more important, and users can reset by deleting config.toml instead of the folder.
- [x] [Review][Defer] Keterbatasan harness NFR10 dan skenario dua-jendela � deferred, pre-existing structural issues of the harness, not product defects.
- [x] [Review][Defer] Isu pada terjemahan spesifik S-07 � deferred, translation reviewed and Indonesian meanings preserved.
