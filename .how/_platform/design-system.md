# Design System — Wira Desk

## Overview

Wira Desk adheres to native Windows 11 Fluent design principles, matching first-party Windows background utilities in appearance, accessibility, and behavior. The system interface is split between an invisible background daemon and an on-demand configuration shell (`wiradesk-settings.exe`).

## Core Principles

1. **Invisibility by Default**: Core window cycling and snapping interactions produce zero on-screen visual clutter, animations, or HUD overlays.
2. **Native Consistency**: UI surfaces adopt the user's active Windows Light/Dark theme and native system typography seamlessly.
3. **Decoupled Architecture**: High-memory UI components are strictly segregated into a standalone on-demand binary (`wiradesk-settings.exe`), preserving a minimal footprint for the background daemon (`wiradesk.exe`).
4. **Accessible First**: Full keyboard navigation, high-contrast focus rings, and complete UI Automation / screen-reader accessibility semantics across all interactive controls.

## Where Values Live

- **Theme Detection & Font Loading**: `crates/settings/src/theme.rs`
- **Settings Shell & Controls**: `crates/settings/src/app.rs`
- **Icon Resources & Manifests**: `crates/daemon/wiradesk.ico`, `crates/daemon/wiradesk.rc`, `crates/settings/wiradesk-settings.rc`
- **Persistence & IPC Protocol**: `crates/settings/src/persistence.rs`, `crates/shared/src/config.rs`

## Design Tokens

### Color Tokens

| Token | Value | Scope | Purpose |
| --- | --- | --- | --- |
| `tray_alert` | `#E81123` | Global / Tray | Warning dot (Tier 2) and critical error cross (Tier 3) overlay badge |
| `theme_mode` | Dynamic (`Light` / `Dark`) | Settings UI | Resolved via registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme` |
| `focus_ring_stroke` | `2.0 pt` | Settings UI | High-contrast keyboard navigation focus outline applied across both Light and Dark visuals |

### Typography Tokens

| Token | Value | Scope | Purpose |
| --- | --- | --- | --- |
| `font_family_primary` | `Segoe UI Variable` / `Segoe UI` | Settings UI | Documented primary Windows 11 UI font loaded directly from `%SystemRoot%\Fonts\segoeui.ttf` |
| `font_family_fallback` | `Tahoma` | Settings UI | Fallback system proportional typeface loaded from `%SystemRoot%\Fonts\tahoma.ttf` |
| `font_family_bundled` | egui default proportional | Settings UI | Fallback typeface if system font files are absent or unparseable |

## Base UI Elements

| Element | States Supported | Implementation / Container | Notes |
| --- | --- | --- | --- |
| **Tray Icon** | Normal, Warning (red dot), Critical (red cross) | `crates/daemon/src/icon.rs` | 16×16 and 32×32 `.ico` resources dynamically badged with GDI overlay |
| **Tray Context Menu** | Active, Dismissed, Disabled | `crates/daemon/src/menu.rs` | Standard Win32 popup menu (`CreatePopupMenu`) responding to right-click |
| **Settings Navigation Tab** | Default, Focused, Selected, Hovered | `crates/settings/src/app.rs` | Accessible tab bar iterating General, Shortcuts, Layout, and About panes |
| **Shortcut Capturer** | Idle, Focused, Listening, Invalid Chord | `crates/settings/src/app.rs` | Intercepts keyboard chords; Escape cancels; validates modifiers + main key |
| **Toggle Switch / Checkbox** | Checked, Unchecked, Focused, Disabled | `crates/settings/src/app.rs` | Controls Auto-Start and Overlapping Stack options with UI Automation semantics |
| **Action Button** | Normal, Hovered, Active, Disabled | `crates/settings/src/app.rs` | Save, Cancel, Reset, and Skip Tutorial buttons |
| **Toast Notification** | Dispatched | `crates/daemon/src/error.rs` | Windows Shell Toast triggered on Tier 3 hook detachment |

## Global Design Rules

| Rule | Prevents |
| --- | --- |
| Zero animation / zero HUD during window switching | Prevents visual distraction and latency during high-speed window cycling |
| Decoupled Settings executable | Prevents GUI runtime memory consumption (>20 MB) from degrading daemon performance |
| Dual-theme focus stroke width enforcement (`2.0 pt`) | Prevents keyboard focus rings from disappearing when the OS theme changes mid-session |
| Validate-before-save atomic TOML write | Prevents invalid shortcut syntax from corrupting daemon configuration |
