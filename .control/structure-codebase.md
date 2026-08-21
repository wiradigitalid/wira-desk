---
type: structure
scope: codebase
verified: 2026-08-21
commit: pending
---

# Codebase Structure

Written and refreshed by `wdi-init` intent `structure`. Rules live in
`.constitution/method/structure-guide.md`.

## Verified

2026-08-21 — Rust workspace, three crates, Windows-only.

## Top level

```text
crates/
  daemon/                     # wiradesk.exe — elevated tray daemon, keyboard hook, arrangement
  settings/                   # wiradesk-settings.exe — egui settings UI
  shared/                     # config, IPC constants, migration, paths
build.ps1                     # prod/dev build entry (MSVC via vcvars64)
scripts/
  verify-public-export.ps1    # Publication hygiene gate
docs/
  decisions.md                # Engineering rationale
  threat-model.md             # Trust boundaries
design-system/                # UI kits and brand assets
```

## Workspace

| Crate | Binary | Role |
| --- | --- | --- |
| `daemon` | `wiradesk.exe` | WH_KEYBOARD_LL hook, window cycling, tray, arrangement |
| `settings` | `wiradesk-settings.exe` | Config editor, onboarding, IPC reload |
| `shared` | (lib) | TOML config, WinTick→WiraDesk migration, paths, constants |

## Build outputs

```text
target/release/wiradesk.exe
target/release/wiradesk-settings.exe
```

## Config on disk (runtime)

```text
%APPDATA%\WiraDesk\config.toml
%APPDATA%\WiraDesk\wiradesk.log
%APPDATA%\WinTick\          # legacy — triggers one-time migration if present
```
