# Design System — Wira Desk

## Overview

Wira Desk adheres to native Windows 11 Fluent 2 Design and Mica Material design principles, matching first-party Windows background utilities and modern settings shells in appearance, accessibility, and behavior. The system interface is decoupled between an invisible background daemon (`wiradesk.exe`) and an on-demand episodic configuration shell (`wiradesk-settings.exe`).

## Core Principles

1. **Invisibility by Default**: Core window cycling and snapping interactions produce zero on-screen visual clutter, animations, or HUD overlays.
2. **Harmonious Native Surfaces (Mica Grounding)**: Multi-layered surface hierarchy: Page Canvas (`#121418`) → Mica Window Surface (`#191D23`) → Sidebar Canvas (`#15181E`) → Card Containers (`#20242B`) → Pill Controls.
3. **Deterministic Keyboard-First Navigation**: 100% accessible via keyboard navigation (Tab traversal, AccessKit/UIA announcements, and physical keycap chord interception).
4. **Decoupled Architecture**: High-memory UI components are strictly segregated into a standalone on-demand binary (`wiradesk-settings.exe`), preserving a minimal static footprint (<2MB RAM) for the background daemon (`wiradesk.exe`).

## Where Values Live

- **Theme Detection, Color Tokens & Font Loading**: `crates/settings/src/theme.rs`
- **Settings Shell, Navigation & Custom Widgets**: `crates/settings/src/app.rs`, `crates/settings/src/main.rs`
- **Icon Resources & Manifests**: `crates/daemon/wiradesk.ico`, `crates/daemon/wiradesk.rc`, `crates/settings/wiradesk-settings.rc`
- **Persistence & IPC Protocol**: `crates/settings/src/persistence.rs`, `crates/shared/src/config.rs`

## Design Tokens

### 1. Color Palette (Dark Theme — Default)

| Token | Hex Value | Scope | Purpose |
|---|---|---|---|
| `bg_page` | `#121418` | Settings UI | Outer window ground canvas / page background |
| `bg_mica` | `#191D23` | Settings UI | Main window surface (Mica Material container) |
| `bg_sidebar` | `#15181E` | Settings UI | Left vertical sidebar navigation column |
| `bg_card` | `#20242B` | Settings UI | Grouped settings card container surface |
| `bg_card_hover` | `#272C35` | Settings UI | Hover state for card rows and sidebar items |
| `bg_subtle` | `#1A1C21` | Settings UI | Sunken area / sandbox preview container |
| `bg_keycap` | `#2B303A` | Settings UI | Physical shortcut keycap button background |
| `stroke_card` | `rgba(255, 255, 255, 0.08)` | Settings UI | Subtle border outline on cards and dialog shells |
| `stroke_divider` | `rgba(255, 255, 255, 0.06)` | Settings UI | Fine horizontal divider line inside card groups |
| `accent_primary` | `#4CC2FF` | Settings UI | Active Windows 11 blue accent (Switch on, Save button, Selection) |
| `accent_hover` | `#60CBFF` | Settings UI | Primary accent hover state |
| `accent_subtle` | `rgba(76, 194, 255, 0.12)` | Settings UI | Soft accent background highlight |
| `text_primary` | `#F3F5F8` | Settings UI | Primary titles, headings, and high-contrast labels |
| `text_secondary` | `#A0A6B4` | Settings UI | Secondary descriptions, subheadings, and guide text |
| `text_tertiary` | `#6B7280` | Settings UI | Captions, shortcut hints, and metadata |
| `signal_success` | `#6CCB5F` | Global / Settings | Active daemon status dot indicator |
| `tray_alert` | `#E81123` | Global / Tray | Warning dot (Tier 2) and critical error cross (Tier 3) overlay badge |
| `focus_ring_stroke`| `2.0 pt` | Settings UI | High-contrast keyboard navigation focus outline |

### 2. Typography Hierarchy

* **Primary Font**: `Segoe UI Variable Text` (`SegUIVar.ttf`)
* **Secondary Font**: `Segoe UI` (`segoeui.ttf`)
* **Monospace Font**: `Cascadia Code` / `Consolas` / `JetBrains Mono`
* **Fallback Font**: `Tahoma` (`tahoma.ttf`) / Bundled proportional face

| Role | Size | Weight | Color Token |
|---|---|---|---|
| **Window Title** | `13.5 pt` | Bold (700) | `text_primary` (`#F3F5F8`) |
| **Section Heading** | `18.0 pt` | Bold (700) | `text_primary` (`#F3F5F8`) |
| **Card Item Title** | `13.0 pt` | Semi-Bold (600) | `text_primary` (`#F3F5F8`) |
| **Card Item Description** | `11.5 pt` | Regular (400) | `text_secondary` (`#A0A6B4`) |
| **Keycap Badge** | `12.0 pt` | Bold Monospace | `text_primary` (`#F3F5F8`) |
| **Status Bar Footer** | `12.0 pt` | Medium (500) | `text_secondary` (`#A0A6B4`) |

## Standard Component Specifications

### A. Frameless Window Shell & Custom Titlebar
* **Window Mode**: Frameless (`egui::ViewportBuilder::default().with_decorations(false)`).
* **Titlebar Height**: `36 px` with ground `#15181E`.
* **Window Dragging**: Left-click on drag area triggers native window repositioning (`ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag)`).
* **Caption Buttons**:
  - `Minimize` (`—`): Dimensions `30 × 24 px`, radius `4 px`, hover `#272C35`, dispatches `ViewportCommand::Minimized(true)`.
  - `Close` (`✕`): Dimensions `30 × 24 px`, radius `4 px`, hover `#C42B1C` (Windows caption red) with white glyph, dispatches `ViewportCommand::Close`.
* **Outer Window Border**: `1.0 px`, `rgba(255, 255, 255, 0.08)`.

### B. Fluent 2 Animated Toggle Switch (`fluent_toggle_switch`)
* **Dimensions**: Width `40 px`, Height `20 px`.
* **Corner Radius**: `CornerRadius::same(10)` (Pill shape).
* **Slider Thumb**: Diameter `12 px`, Margin `4 px`.
* **Animation**: Responsive smooth transition via `ui.ctx().animate_bool_responsive(id, on)`.
* **Active State (ON)**: Track `#4CC2FF`, thumb `#101216` (or pure white).
* **Inactive State (OFF)**: Track `#282A30`, border `rgba(255, 255, 255, 0.20)`, thumb `#A0A6B4`.

### C. Vertical Sidebar Navigation
* **Column Width**: `175 px` – `180 px`.
* **Background**: `#15181E`.
* **Navigation Item**: Width `165 px`, Height `36 px`, Radius `6 px`.
* **Active Indicator Bar**: Vertical pill bar (Width `3.5 px`, Height `20 px`, Radius `2 px`, Color `#4CC2FF`) positioned at the left edge of the active item.
* **Panes (5)**: `General`, `Shortcuts`, `Layout & Snapping`, `VM & Exceptions`, `About`.

### D. Settings Card Container
* **Background**: `#20242B`.
* **Corner Radius**: `CornerRadius::same(8)`.
* **Stroke**: `1.0 px`, `rgba(255, 255, 255, 0.08)`.
* **Padding**: Horizontal `14 px`, Vertical `12 px`.
* **Dividers**: Fine horizontal line `rgba(255, 255, 255, 0.06)`.

### E. Fixed Status Footer Bar
* **Bar Height**: `48 px`.
* **Background**: `#20242B` with top border `1.0 px`.
* **Status Dot**: Solid green dot `#6CCB5F` (Diameter `8 px`).
* **Action Buttons**:
  - `Revert`: Background `#2B303A`, text `#F3F5F8`, dimensions `80 × 30 px`.
  - `Save Changes`: Background `#4CC2FF`, text `#101216`, dimensions `110 × 30 px`.

### F. Onboarding Wizard Modal
* **Dialog Width**: `520 px` – `540 px`, Height `400 px` – `420 px`.
* **Corner Radius**: `CornerRadius::same(12)`.
* **Background**: `#20242B` with subtle border `#323640`.
* **Progress Bar (3-Step)**: Balanced segment width, thickness `3.5 px`, color `#4CC2FF` (Active) / `#2B303A` (Inactive).
* **Interactive Sandbox**: Simulated dual-window focus toggle responding to physical `Win + \`` keypresses.

## Global Design Rules & Invariants

| Rule | Prevents |
| --- | --- |
| Zero animation / zero HUD during window switching | Prevents visual distraction and latency during high-speed window cycling |
| Decoupled Settings executable | Prevents GUI runtime memory consumption (>20 MB) from degrading daemon performance |
| Frameless window with custom titlebar | Eliminates dated OS caption chrome, achieving seamless modern Windows 11 aesthetics |
| Dual-theme focus stroke width enforcement (`2.0 pt`) | Prevents keyboard focus rings from disappearing when the OS theme changes mid-session |
| Validate-before-save atomic TOML write | Prevents invalid shortcut syntax from corrupting daemon configuration |
