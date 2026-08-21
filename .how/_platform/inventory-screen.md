# Screen & UI Surface Inventory

This document catalogues all graphical user interface surfaces, dialogs, menus, and transient visual states across Wira Desk.

## Top-Level Windows & Dialogs

| Screen / Dialog | ID | Owner | Container | Entry Point | Technology | Description |
| --- | --- | --- | --- | --- | --- | --- |
| **Settings Dialog** | `SCR-SETTINGS` | `settings` | `settings` (`wiradesk-settings.exe`) | Tray Menu → "Settings", or manual CLI launch | Rust, `eframe` / `egui` + `accesskit` | Tabbed/modular configuration UI: General (auto-start), Switcher shortcuts, Snapping bindings, VM/RDP bypass lists, About & typography display. |
| **First-Run Onboarding** | `SCR-ONBOARDING` | `settings` | `settings` (`wiradesk-settings.exe`) | Daemon launch with `--onboarding` on first run (no `config.toml`) | Rust, `eframe` / `egui` + `accesskit` | Interactive multi-step tutorial featuring dummy test windows to practice same-app cycling and snapping before entering production usage. |
| **About Dialog / View** | `SCR-ABOUT` | `settings` | `settings` (`wiradesk-settings.exe`) | Settings UI tab or Tray Menu → "About" | Rust, `egui` | Version info, licensing notice, website link, and active typography resolution (Segoe UI vs fallback). |
| **Startup Error Dialog** | `SCR-FATAL-POPUP` | `window-management` | `daemon` (`wiradesk.exe`) | Tier 1 fatal startup failure (e.g. hook initialization failed after 5 retries) | Win32 `MessageBoxW` | Modal native message box with Error icon. Closes process immediately on dismissal. |

## Shell & System Surfaces

| Surface | ID | Owner | Container | Presentation | Trigger / Behavior |
| --- | --- | --- | --- | --- | --- |
| **Tray Context Menu** | `MNU-TRAY` | `window-management` | `daemon` (`wiradesk.exe`) | Native Win32 popup menu (`TrackPopupMenuEx`) | Right-click on tray icon. Items in order: **Settings**, **View Logs**, **Run at Startup** (toggle check), **Check for Updates**, **About**, **Exit**. |
| **Critical Toast Notification** | `NOTIF-TOAST` | `window-management` | `daemon` (`wiradesk.exe`) | Native Windows Toast Notification | Dispatched exactly 1x upon escalating to Tier 3 Critical (keyboard hook dead). Informs user that cycling is temporarily paused. |

## Tray Icon Visual States (AD-7 Protocol)

The system tray icon reflects the 3-Tier error protocol via visual overlays:

| State | Visual Asset / Badge | Condition | User Action |
| --- | --- | --- | --- |
| **Normal** | Default Wira Desk icon (clean monochrome / brand mark) | Hook active, no unread warnings. | Normal cycling operation. |
| **Warning (Tier 2)** | Default icon + **Amber / Red Dot overlay** | One or more runtime warnings logged to `wiradesk.log` (e.g. non-fatal API refusal). | Clicking "View Logs" in the tray context menu opens log file and clears the dot. |
| **Critical (Tier 3)** | Default icon + **Red X overlay** | Keyboard hook died and could not be recovered after 3 heartbeat retries. | User inspects logs or restarts utility; toast alert fired once. |
