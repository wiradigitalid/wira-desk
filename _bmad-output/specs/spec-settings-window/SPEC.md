---
id: SPEC-settings-window
title: Settings Window (Fluent 2 Mica Shell) Specification
status: ready-for-dev
created: 2026-08-24
updated: 2026-08-24
companions:
  - prototype.html
  - ../../../.how/_platform/design-system.md
  - ../../../.what/settings/04-usecases/UC-4-change-shortcut.md
  - ../../../.what/settings/04-usecases/UC-6-toggle-auto-start.md
  - ../../../.what/settings/04-usecases/EXPERIENCE.md
  - ../../../.how/settings/01-ux/DESIGN.md
  - ../../../.how/settings/04-components/LC-settings-shell.md
  - ../../../.how/settings/04-components/LC-shortcut-capturer.md
  - ../../../.how/settings/04-components/LC-config-writer.md
sources:
  - ../../../.what/_prd/wira-desk/prd.md
  - ../../../.control/registry/requirements.yaml
satisfies:
  - FR-7
  - FR-13
  - FR-14
  - FR-15
  - FR-16
  - FR-18
  - FR-19
  - FR-20
  - FR-21
---

# SPEC — Settings Window (Fluent 2 Mica Shell)

## 1. Why & Problem Statement

Wira Desk configuration requires an accessible, responsive, and intuitive graphical settings interface for customizing global shortcuts, managing startup integration, setting up window snapping/stacking parameters, and configuring VM/RDP exceptions. 

To maintain the background daemon's memory target (<2MB RAM), the Settings window is an episodic executable (`wiradesk-settings.exe`) running Fluent 2 Mica styling with zero persistent background memory overhead.

---

## 2. Capabilities & Acceptance Criteria

### CAP-SET-1: Frameless Mica Window & Vertical Sidebar Navigation
* **Intent:** Provide a clean, frameless, scalable, and keyboard-accessible vertical sidebar navigation structure conforming to Windows 11 Fluent 2 Design Guidelines with native OS chrome disabled (`with_decorations(false)`).
* **Success Criteria:**
  1. Window dimensions: 620 × 560 px (resizable with min size 500 × 380 px).
  2. Frameless titlebar with integrated custom caption controls (`—` Minimize via `ViewportCommand::Minimized(true)` and `✕` Close via `ViewportCommand::Close`).
  3. Sidebar contains 5 distinct navigation items: `General`, `Shortcuts`, `Layout & Snapping`, `VM & Exceptions`, `About`.
  4. Supports seamless linear Tab navigation with high-contrast (2.0 pt) active focus indicators.

### CAP-SET-2: Interactive Physical Shortcut Capturer
* **Intent:** Intercept physical keyboard chords directly into a specialized `Listening` mode instead of accepting typed Unicode text.
* **Success Criteria:**
  1. Clicking a shortcut button transitions the control to `Listening` mode with visual pulse feedback and AccessKit announcement.
  2. Requires at least one modifier key (`Win`, `Ctrl`, `Alt`, or `Shift`) and exactly one main key.
  3. Rejects invalid combinations (e.g. bare keys, modifier-only, multiple main keys) with precise inline feedback.
  4. Pressing `Escape` cancels listening and restores the previous valid chord.

### CAP-SET-3: Collision-Free Layout & Snapping Configuration (v1.3 Alignment)
* **Intent:** Provide DPI-aware window snapping and Overlapping Stack layout shortcuts with 100% collision immunity against Windows OS and Virtual Desktop hotkeys.
* **Success Criteria:**
  1. Configures **`Ctrl + Alt + Win`** as the default multi-modifier cluster:
     * Snap Left 50%: `Ctrl + Alt + Win + ←`
     * Snap Right 50%: `Ctrl + Alt + Win + →`
     * Maximize / Next Monitor: `Ctrl + Alt + Win + Enter`
     * Overlapping Stack: `Ctrl + Alt + Win + ↓`
  2. Provides an interactive width slider for Overlapping Stack (range 30% to 80%, default 50%).

### CAP-SET-4: Staged Draft Editing & Atomic IPC Reload Pipeline
* **Intent:** Stage all user edits in memory; persist atomically and signal the daemon only when "Save Changes" is explicitly triggered.
* **Success Criteria:**
  1. Making any change marks the draft dirty and enables the `Revert` button.
  2. Clicking `Save Changes` validates all fields in memory, writes atomically to `config.toml.tmp` → `config.toml`, and sends `WM_APP_RELOAD_CONFIG` (0x8001) to `WiraDeskDaemonHiddenWindow`.
  3. If daemon is running, status updates to *"Settings saved and applied."*
  4. If daemon is absent, status updates to *"Settings saved. Applies on next launch."*

---

## 3. UI Pane Architecture & Content Register

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ [🪟] Wira Desk — Settings                                        [─]   [✕]  │
├───────────────┬─────────────────────────────────────────────────────────────┤
│ ⚙️ General    │ [General Settings Pane]                                     │
│ ⌨️ Shortcuts  │ - Auto-start on boot (Toggle Task Scheduler)                │
│ 🗂️ Layout &   │ - Spatial Preservation / Per-Monitor Lock (Toggle)          │
│    Snapping   │ - Virtual Desktop Isolation (Toggle)                        │
│ 🖥️ VM &       │ - UX Honesty Mode (Toggle)                                  │
│    Exceptions ├─────────────────────────────────────────────────────────────┤
│ ℹ️ About      │ [Shortcuts Pane]                                            │
│               │ - Primary Switcher: [ Win + ` ]                             │
│               │ - Fallback Switcher: [ Alt + ` ]                            │
│               │ - Precise Modifier Matching (Toggle)                        │
│               ├─────────────────────────────────────────────────────────────┤
│               │ [Layout & Snapping Pane]                                    │
│               │ - Snap Left: [ Ctrl + Alt + Win + ← ]                       │
│               │ - Snap Right: [ Ctrl + Alt + Win + → ]                      │
│               │ - Maximize: [ Ctrl + Alt + Win + Enter ]                    │
│               │ - Enable Overlapping Stack (Toggle)                         │
│               │ - Stack Width Ratio: [ Slider 30%-80% (50%) ]               │
│               │ - Stack Shortcut: [ Ctrl + Alt + Win + ↓ ]                  │
│               ├─────────────────────────────────────────────────────────────┤
│               │ [VM & Exceptions Pane]                                      │
│               │ - Virtual Machine Passthrough (Toggle)                      │
│               │ - Remote Desktop Passthrough (Toggle)                       │
│               │ - Process Exclusion Rules                                   │
│               ├─────────────────────────────────────────────────────────────┤
│               │ [About Pane]                                                │
│               │ - Version, Loaded Typeface, RAM footprint diagnostics       │
├───────────────┴─────────────────────────────────────────────────────────────┤
│ 🟢 Daemon Running Elevated (PID: 10428)        [ Revert ]  [ Save Changes ] │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Accessibility & UI Automation (UIA) Contract

1. **Tab Navigation Sequence:**
   Sidebar Pane Selector → Active Pane Controls (Top-to-Bottom) → Revert Button → Save Changes Button.
2. **AccessKit Screen Reader Announcements:**
   * Active tab selection: *"Shortcuts tab, selected, 2 of 5"*.
   * Listening state: *"Listening for shortcut chord. Press desired keys or Escape to cancel."*.
   * Toggle controls: *"Auto-start on boot, switch, checked"*.
   * Save outcomes: *"Settings saved and applied to daemon."*.

---

## 5. Architectural Decisions & Constraints

* **AD-1 & AD-11:** Standalone executable running `egui`/`eframe` with `accesskit` feature and frameless viewport (`with_decorations(false)`).
* **AD-5:** Atomic disk write (`config.toml.tmp` → rename) prior to IPC signal via `PostMessageW(WM_APP_RELOAD_CONFIG)`.
* **AD-13:** Auto-start toggle manages `shared::Config.general.auto_start` and synchronizes with Task Scheduler via daemon.
* **AD-11a:** Adaptive theming using system font `Segoe UI Variable` with automatic dark/light registry synchronization and custom frameless titlebar caption controls.
