# Wira Desk

Lightweight desktop tools for Windows.

Wira Desk is a pre-release suite that runs as an elevated system-tray daemon with a global low-level keyboard hook for same-app window switching and window arrangement.

## About

Wira Desk began as **WinTick**, a private personal project. The source is now published under the Wira Desk name as its first public release.

**Wira Digital Indonesia** is the studio brand behind this project; it is still being established. Wira Desk is built and maintained by [@kodesh87](https://github.com/kodesh87) - the studio name and the maintainer are the same effort, not separate products.

If you used WinTick before, settings migrate automatically on first run (see Factory reset below).

## Build

Requires Rust (stable) and the MSVC toolchain on Windows.

```powershell
.\build.ps1 -Mode prod
```

Binaries: `target\release\wiradesk.exe` and `target\release\wiradesk-settings.exe`.

## Install

1. Build or obtain the release binaries.
2. Place `wiradesk.exe` and `wiradesk-settings.exe` in the same folder.
3. Run `wiradesk.exe` once as Administrator so the daemon can install its hook and migrate settings from a prior installation if present.

## Administrator and keyboard hook

Wira Desk requests Administrator because it installs `WH_KEYBOARD_LL` and must activate windows across integrity levels (UIPI). The hook observes key events to match configured shortcuts; it does not log keystroke contents to disk. See `SECURITY.md` and `PRIVACY.md`.

## Data on disk

Configuration and logs live under `%APPDATA%\WiraDesk\` (`config.toml`, `wiradesk.log`).

## Factory reset

To restore defaults (including first-run onboarding), delete the config file only:

```text
Delete   %APPDATA%\WiraDesk\config.toml
Do NOT   delete the %APPDATA%\WiraDesk\ folder
```

Migration from a prior WinTick install is triggered by the **presence** of the legacy `%APPDATA%\WinTick\` directory, which is kept intact so rollback remains possible. Removing the whole `WiraDesk` folder does not reset settings - the next start re-imports from WinTick. Deleting only `config.toml` while leaving both directories in place skips re-migration and loads factory defaults.

## Status

Pre-release (`0.1.0`). Behavior and packaging may change.

## License

MIT - see [LICENSE](LICENSE).