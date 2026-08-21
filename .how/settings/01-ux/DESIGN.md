# Wira Desk Visual Design — Settings

## Brand & Style

Wira Desk adheres strictly to the native Windows 11 Fluent design language. The UI feels indistinguishable from a first-party Windows utility dialog. It prioritizes invisibility over presence: the primary cycling and snapping capabilities feature zero on-screen HUD or graphical switcher overlays.

## Theme & Colors

Settings inherits the active Windows Light or Dark mode setting dynamically via the OS personalization registry (`AppsUseLightTheme`).

- **OS Adaptive Theme**: Neutral surfaces, text contrast, borders, and widget backgrounds adapt dynamically to system Light or Dark mode.
- **Alert Overlay Color**: `#E81123` (used exclusively for System Tray health badges — red dot for Tier 2 warnings/logs, red cross for Tier 3 hook failure).

## Typography

Settings dialogs and onboarding surfaces utilize the documented Windows UI typeface hierarchy:
- **Primary Typeface**: `Segoe UI Variable`, falling back to `Segoe UI`, `Tahoma`, or bundled font as necessary.
- **Rendering**: Crisp subpixel font rendering with strict font validation (`ttf-parser`) before GPU atlas rasterization.

## Layout & Spacing

Follows standard Windows 11 dialog spacing and structured pane hierarchy within the decoupled executable (`wiradesk-settings.exe`):
- **Panes**: General, Shortcuts, Layout, About.
- **Modular Grouping**: Each configuration group contains:
  - Toggle switch (Enable/Disable state).
  - Shortcut capturer field (listening state for real-time key chord capture, rejecting raw text editing).
  - Inline validation feedback for invalid or ambiguous chords.

## UI Components

### 1. System Tray Icon & Context Menu
- **Asset Dimensions**: 16×16 and 32×32 pixel `.ico` formats supporting multi-DPI displays.
- **Menu Hierarchy**: Settings..., View Logs, Auto-Start (toggle), Check for Updates..., About, Exit.

### 2. Settings Window (`wiradesk-settings.exe`)
- **Structure**: Native dialog shell with accessible tab navigation across four panes (General, Shortcuts, Layout, About).
- **Shortcut Capturer Control**: Dedicated interactive listening widget with clear auditory/screen-reader announcements and Escape key cancellation.
- **Save Bar**: Sticky footer with Save button, validation error summary, and instantaneous IPC status reflection.

### 3. First-Run Onboarding Dialog
- **Modal Tutorial Shell**: Interactive practice arena invoked with `--onboarding`.
- **Simulated Windows**: Responsive dummy window widgets that alternate visual focus states when the user presses `Win + \``.
- **Skip Action**: Explicit "Skip Tutorial" button for experienced power users.

## Do's and Don'ts

- **Do** keep window cycling 100% invisible: zero delay, zero animation, zero visual HUD overlays.
- **Don't** embed heavyweight GUI runtimes (Electron, WPF, CEF) into the core background daemon. The UI must remain decoupled in `wiradesk-settings.exe` to enforce minimal daemon memory usage.
- **Do** respect UX Honesty: surface "Not Responding" windows during cycling rather than hiding them.
- **Do** provide high-contrast 2.0 pt keyboard focus rings across both Light and Dark themes for accessibility compliance.
