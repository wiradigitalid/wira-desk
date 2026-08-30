# Wira Desk

Windows doesn't have macOS's `⌘+\`` same-app window cycling — Alt+Tab piles every app's windows
together instead. Wira Desk adds that, plus fast one-key window snapping and arranging, to
Windows 11 as one lightweight, keyboard-driven tray daemon.

## Why

- **Alt+Tab treats every window from every app as one pile.** With a dozen windows open, cycling
  to the one you want becomes a small tax on your attention, every time. Windows has no built-in
  way to cycle only the windows of the app you're currently using — that's what `Win + \`` does
  here.
- **Fast, one-key window snapping and arranging.** Windows already has window snapping; this is
  about speed and consistency — snap left/right/top/bottom, maximize, move to another monitor, or
  stack three windows, each with one dedicated keyboard shortcut instead of a mouse drag or menu.
- The one third-party tool that came close to same-app window cycling had been unmaintained for
  years, so this exists to fill that specific gap.

## Default shortcuts

All of these are remappable from the Settings app.

| Shortcut | Action |
|---|---|
| `` Win + ` `` | Cycle windows of the app you're currently using (e.g. Chrome window 1 → Chrome window 2 → ...) |
| `Ctrl+Alt+Left/Right/Up/Down` | Snap the window to that half of the screen (50%) |
| `Ctrl+Alt+Enter` | Maximize the window to full screen |
| `Ctrl+Alt+Shift+Enter` | Move the window to the next monitor (multi-monitor setups) |
| `Ctrl+Alt+Shift+Down` | Stack 3 windows at 50% width each — useful on a small monitor when you still want another window visible |

## About

Wira Desk began as **WinTick**, a private personal project born from missing macOS's `⌘+\``
same-app window cycling after moving to Windows — Alt+Tab doesn't do that job. It also bundles
fast, one-key window snapping and arranging: quick left/right/top/bottom snap, maximize, and
move-to-monitor, each on its own dedicated shortcut instead of a drag or a menu. The source is now
published under the Wira Desk name as its first public release.

**Wira Digital Indonesia** is the studio brand behind this project; it is still being established.
Wira Desk is built and maintained by [@kodesh87](https://github.com/kodesh87) - the studio name
and the maintainer are the same effort, not separate products.

If you used WinTick before, settings migrate automatically on first run (see Factory reset below).

## Install

Download `WiraDesk-<version>-x64-setup.exe` from the
[releases page](https://github.com/wiradigitalid/wira-desk/releases) (mirrored on
[SourceForge](https://sourceforge.net/projects/wira-desk/files/latest/download)) and run it.
Verify it against the published `SHA256SUMS` first:

```powershell
Get-FileHash .\WiraDesk-0.1.4-x64-setup.exe -Algorithm SHA256
```

The installer needs Administrator, installs to `%ProgramFiles%\Wira Desk`, and offers no per-user
location. That is deliberate rather than an omission - auto-start runs the daemon elevated at
every logon with no prompt, so a directory only administrators can write is the whole thing
protecting it. See `SECURITY.md`.

It does **not** switch auto-start on. That stays yours to enable from the tray menu or Settings.

### Without the installer

The loose binaries are published beside it, so you can run the program without one:

1. Place `wiradesk.exe` and `wiradesk-settings.exe` in the same folder - one only administrators
   can write.
2. Run `wiradesk.exe` as Administrator.

Wira Desk will warn you, in the log and on the tray icon, if you turn auto-start on from a folder
a normal user could overwrite.

## Update

Download the newer setup executable from the
[releases page](https://github.com/wiradigitalid/wira-desk/releases) and run it over the old
install; it stops the running daemon, replaces the files in place, and keeps your settings. There
is no update check inside the application, and no network path in it at all - see `SECURITY.md`.

## Uninstall

From Add/Remove Programs.

Uninstalling removes the program and the auto-start scheduled task. It leaves
`%APPDATA%\WiraDesk\` alone - delete that folder by hand if you want your settings and log gone,
and read Factory reset below first, because that folder does not behave the way you might expect.

## Build

Requires Rust (stable) and the MSVC toolchain on Windows.

```powershell
.\build.ps1 -Mode prod
```

Binaries: `target\release\wiradesk.exe` and `target\release\wiradesk-settings.exe`.

## Administrator and keyboard hook

Wira Desk requests Administrator because it installs `WH_KEYBOARD_LL` and must activate windows
across integrity levels (UIPI). The hook observes key events to match configured shortcuts; it
does not log keystroke contents to disk. See `SECURITY.md` and `PRIVACY.md`.

## Data on disk

Configuration and logs live under `%APPDATA%\WiraDesk\` (`config.toml`, `wiradesk.log`).

## Factory reset

To restore defaults (including first-run onboarding), delete the config file only:

```text
Delete   %APPDATA%\WiraDesk\config.toml
Do NOT   delete the %APPDATA%\WiraDesk\ folder
```

Migration from a prior WinTick install is triggered by the **presence** of the legacy
`%APPDATA%\WinTick\` directory, which is kept intact so rollback remains possible. Removing the
whole `WiraDesk` folder does not reset settings - the next start re-imports from WinTick. Deleting
only `config.toml` while leaving both directories in place skips re-migration and loads factory
defaults.

## Status

Pre-release (`0.1.4`). Behavior and packaging may change. Not code-signed yet, so Windows
SmartScreen and the UAC prompt will show an unverified publisher warning - verify the published
`SHA256SUMS` before running.

## Built with Slint

[![#MadeWithSlint](https://raw.githubusercontent.com/slint-ui/slint/master/logo/MadeWithSlint-logo-light.svg)](https://slint.dev)

The Settings window is built with [Slint](https://slint.dev). Slint is offered under a choice of
licences, and Wira Desk uses it under the **Slint Royalty-free License 2.0** - which requires that
this use be disclosed on a public page where the binaries can be found. That is what this section
is, so it belongs to the licence rather than to courtesy: it is not decoration to be tidied away.

## Contributing

Contributions are welcome via pull request - see [CONTRIBUTING.md](CONTRIBUTING.md) for the checks
CI runs and the conventions this repository follows.

## License

MIT - see [LICENSE](LICENSE). Third-party dependency licences are listed in [NOTICE](NOTICE),
generated from `cargo metadata`.
