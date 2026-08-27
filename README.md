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

Through winget:

```powershell
winget install WiraDigitalIndonesia.WiraDesk
```

Or download `WiraDesk-<version>-x64-setup.exe` from the [releases page](https://github.com/kodesh87/wira-desk/releases) and run it. Verify it against the published `SHA256SUMS` first:

```powershell
Get-FileHash .\WiraDesk-0.1.0-x64-setup.exe -Algorithm SHA256
```

The installer needs Administrator, installs to `%ProgramFiles%\Wira Desk`, and offers no per-user location. That is deliberate rather than an omission - auto-start runs the daemon elevated at every logon with no prompt, so a directory only administrators can write is the whole thing protecting it. See `SECURITY.md`.

It does **not** switch auto-start on. That stays yours to enable from the tray menu or Settings.

### Without the installer

The loose binaries are published beside it, so you can run the program without one:

1. Place `wiradesk.exe` and `wiradesk-settings.exe` in the same folder - one only administrators can write.
2. Run `wiradesk.exe` as Administrator.

Wira Desk will warn you, in the log and on the tray icon, if you turn auto-start on from a folder a normal user could overwrite.

## Update

```powershell
winget upgrade WiraDigitalIndonesia.WiraDesk
```

Or run the newer setup executable over the old install; it stops the running daemon, replaces the files in place, and keeps your settings. There is no update check inside the application, and no network path in it at all - see `SECURITY.md`.

## Uninstall

Through winget (`winget uninstall WiraDigitalIndonesia.WiraDesk`), or from Add/Remove Programs.

Uninstalling removes the program and the auto-start scheduled task. It leaves `%APPDATA%\WiraDesk\` alone - delete that folder by hand if you want your settings and log gone, and read Factory reset below first, because that folder does not behave the way you might expect.

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

## Built with Slint

[![#MadeWithSlint](https://raw.githubusercontent.com/slint-ui/slint/master/logo/MadeWithSlint-logo-light.svg)](https://slint.dev)

The Settings window is built with [Slint](https://slint.dev). Slint is offered under a choice of licences, and Wira Desk uses it under the **Slint Royalty-free License 2.0** - which requires that this use be disclosed on a public page where the binaries can be found. That is what this section is, so it belongs to the licence rather than to courtesy: it is not decoration to be tidied away.

## License

MIT - see [LICENSE](LICENSE). Third-party dependency licences are listed in [NOTICE](NOTICE), generated from `cargo metadata`.