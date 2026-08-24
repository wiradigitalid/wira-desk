---
id: SPEC-first-run-tutorial
title: First-Run Onboarding Simulation Specification
status: ready-for-dev
created: 2026-08-24
updated: 2026-08-24
companions:
  - prototype.html
  - ../../../.how/_platform/design-system.md
  - ../../../.what/settings/04-usecases/UC-5-first-run-tutorial.md
  - ../../../.what/settings/04-usecases/EXPERIENCE.md
  - ../../../.how/settings/01-ux/DESIGN.md
  - ../../../.how/settings/04-components/LC-settings-shell.md
sources:
  - ../../../.what/_prd/wira-desk/prd.md
  - ../../../.control/registry/requirements.yaml
satisfies:
  - FR-17
  - FR-19
  - FR-20
  - FR-21
---

# SPEC — First-Run Tutorial (Interactive Onboarding)

## 1. Why & Problem Statement

Windows users transitioning from macOS frequently miss the native app-specific cycling workflow (`Cmd + ~` / `Win + \``). Additionally, users are accustomed to global window switchers like `Alt+Tab` that span all monitors and applications. 

The First-Run Tutorial (`wiradesk-settings.exe --onboarding`) provides an interactive, episodic sandbox that builds immediate muscle memory for same-application spatial cycling without burdening users with static manual documentation.

---

## 2. Capabilities & Acceptance Criteria

### CAP-OB-1: First-Run Detection & Invocation
* **Intent:** The system detects when Wira Desk is launched without an existing `%APPDATA%\WiraDesk\config.toml` file or with the CLI flag `--onboarding`, automatically presenting the onboarding wizard instead of remaining silently in the system tray.
* **Success Criteria:**
  1. If `config.toml` is absent from `%APPDATA%\WiraDesk`, `wiradesk-settings.exe` launches with `--onboarding`.
  2. If launched normally with an existing configuration, onboarding is bypassed.

### CAP-OB-2: Step 1 — Welcome & Conceptual Grounding
* **Intent:** Explain the fundamental difference between Wira Desk (same-app, per-monitor) and Windows `Alt+Tab` (all apps, all monitors) in a frameless modal dialog shell.
* **Success Criteria:**
  1. Displays frameless modal window (520 × 400 px, `with_decorations(false)`) centered on screen with Fluent 2 Mica material.
  2. Integrated custom titlebar with title and clean `✕` Close caption control (`ViewportCommand::Close`).
  3. Top step indicator highlights Step 1 of 3.
  4. Header reads: `Welcome to Wira Desk`.
  5. Actions: `Skip Tutorial` (dismisses wizard and persists baseline config) and `Next →` (advances to Step 2).

### CAP-OB-3: Step 2 — Interactive Same-App Cycling Simulation
* **Intent:** Provide a live interactive simulation within the onboarding UI that shifts focus between simulated dummy windows when `Win + \`` is pressed.
* **Success Criteria:**
  1. Displays 2 simulated dummy window widgets: *"Document 1"* (initially focused with blue stroke) and *"Document 2"*.
  2. Captures the physical `Win + \`` keypress while the onboarding window is active.
  3. On keypress, visual focus shifts smoothly from Document 1 to Document 2, displaying a confirmation message (*"Great! Focus shifted instantaneously"*).
  4. The `Next →` button activates to proceed to Step 3.
  5. An explicit `Skip Tutorial` action remains available and functional.

### CAP-OB-4: Step 3 — Completion & Persistence
* **Intent:** Confirm successful setup and atomically write baseline configuration so onboarding does not recur.
* **Success Criteria:**
  1. Step 3 displays confirmation that Wira Desk is running in the background System Tray.
  2. Clicking `Finish & Start Using Wira Desk` (or `Skip Tutorial` at any stage) writes `config.toml` atomically to disk.
  3. The onboarding process terminates cleanly and releases all GUI memory.

---

## 3. Interaction & State Machine Contract

```text
[Launch Wira Desk]
        │
        ▼
[Check %APPDATA%\WiraDesk\config.toml existence]
        │
        ├─► File Missing OR Flag --onboarding ──► [Launch wiradesk-settings.exe --onboarding]
        │                                                                │
        └─► File Present ──────────────────────► [Run Headless Daemon in Tray]
                                                                         │
        ┌────────────────────────────────────────────────────────────────┘
        ▼
[Step 1: Welcome] ──(Next)──► [Step 2: Try Switching] ──(Keypress Win+`)──► [Step 3: Done]
        │                                  │                                      │
   (Skip Clicked)                     (Skip Clicked)                       (Finish Clicked)
        │                                  │                                      │
        └──────────────────────────────────┴──────────────────────────────────────┘
                                           ▼
                   [Atomically Write Baseline config.toml to Disk]
                                           ▼
                                [Exit UI Process (0 RAM)]
                                           ▼
                            [Daemon Runs Invisibly in Tray]
```

---

## 4. Accessibility & UI Automation (UIA) Contract

1. **Keyboard Navigation:** Every control is reachable via `Tab` / `Shift+Tab` and actionable via `Space` / `Enter`.
2. **Escape Key Handling:** Pressing `Escape` triggers immediate tutorial dismissal (equivalent to `Skip Tutorial`).
3. **Screen Reader Support (AccessKit):**
   * Announces step changes: *"Step 1 of 3: Welcome to Wira Desk"*, *"Step 2 of 3: Try switching windows"*, *"Step 3 of 3: You are all set"*.
   * Announces simulated window focus shift: *"Simulated focus moved to Document 2"*.

---

## 5. Constraints & Invariants

* **AD-11:** Onboarding GUI is hosted entirely in `wiradesk-settings.exe` (`egui`/`eframe`) as a frameless dialog (`with_decorations(false)`), never inside the background daemon.
* **AD-11a:** Uses `accesskit` feature and `Segoe UI Variable` system typography with custom titlebar caption controls.
* **BR-3:** Completing or skipping onboarding writes a valid default configuration, preventing recurrence on next boot.
