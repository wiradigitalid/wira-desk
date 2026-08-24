# Wira Desk Visual Design — Settings

## Brand & Style

Wira Desk adheres strictly to the native Windows 11 Fluent 2 design language with Mica material styling. The UI feels indistinguishable from a modern, first-party Windows utility dialog. It prioritizes invisibility over presence: the primary cycling and snapping capabilities feature zero on-screen HUD or graphical switcher overlays.

## Theme & Surfaces

Settings inherits the active Windows Light or Dark mode setting dynamically via the OS personalization registry (`AppsUseLightTheme`).

- **OS Adaptive Theme**: Neutral surfaces, text contrast, borders, and widget backgrounds adapt dynamically to system Light or Dark mode.
- **Layered Grounding**: Outer Canvas (`#121418`) → Mica Window Surface (`#191D23`) → Sidebar Navigation (`#15181E`) → Settings Card Surface (`#20242B`).
- **Alert Overlay Color**: `#E81123` (used exclusively for System Tray health badges — red dot for Tier 2 warnings/logs, red cross for Tier 3 hook failure).

## Typography

Settings dialogs and onboarding surfaces utilize the documented Windows UI typeface hierarchy:
- **Primary Typeface**: `Segoe UI Variable Text` (`SegUIVar.ttf`), falling back to `Segoe UI`, `Tahoma`, or bundled font as necessary.
- **Rendering**: Crisp subpixel font rendering with strict font validation (`ttf-parser`) before GPU atlas rasterization.

## Layout & Navigation Hierarchy

Follows Windows 11 dialog spacing with a structured 5-pane vertical sidebar within the decoupled executable (`wiradesk-settings.exe`):
- **Panes (5)**: 
  1. `General`: Startup integration (Task Scheduler), Spatial Lock, Virtual Desktop isolation, and UX Honesty controls.
  2. `Shortcuts`: Core same-app window switcher (`Win + \``), Alt-fallback switcher, and precise modifier matching.
  3. `Layout & Snapping`: Collision-free `Ctrl + Alt + Win` snapping combinations (Left 50%, Right 50%, Maximize) and Overlapping Stack slider / shortcut.
  4. `VM & Exceptions`: Passthrough rules for virtualization guests (`mstsc.exe`, `vmconnect.exe`, `VMwareUnityWindow`).
  5. `About`: Diagnostic build metadata, version info, active font rendering details, and memory footprint.
- **Modular Grouping**: Each configuration group is rendered within a rounded Card Container (`#20242B`, radius 8px) with fine divider lines.

## UI Components

### 1. System Tray Icon & Context Menu
- **Asset Dimensions**: 16×16 and 32×32 pixel `.ico` formats supporting multi-DPI displays.
- **Menu Hierarchy**: Settings..., View Logs, Auto-Start (toggle), Check for Updates..., About, Exit.

### 2. Frameless Settings Window (`wiradesk-settings.exe`)
- **Structure**: Modern frameless shell (`with_decorations(false)`) with custom caption bar (`36 px`, `#15181E`) containing draggable region, minimize (`—`), and close (`✕`) buttons.
- **Vertical Sidebar**: Fixed-width navigation column (`175 px`) with active blue indicator pill (`#4CC2FF`, `3.5 × 20 px`).
- **Pill Toggle Switch (`fluent_toggle_switch`)**: Interactive animated toggle switch widget (`40 × 20 px`) with smooth state transitions.
- **Shortcut Capturer Control**: Dedicated interactive listening widget with clear auditory/screen-reader announcements and Escape key cancellation.
- **Save Bar**: Sticky footer (`48 px`) with Revert and Save Changes buttons, validation error summary, and instantaneous IPC status reflection.

### 3. First-Run Onboarding Modal Dialog
- **Modal Tutorial Shell**: Frameless interactive practice arena (`520 × 400 px`) invoked with `--onboarding`.
- **Progress Bar**: 3-step indicator displaying *Welcome*, *Try Switching*, and *Done*.
- **Simulated Windows**: Responsive dummy window widgets that alternate visual focus states when the user presses `Win + \``.
- **Skip Action**: Explicit "Skip Tutorial" button for experienced power users (also bound to `Escape`).

## Do's and Don'ts

- **Do** keep window cycling 100% invisible: zero delay, zero animation, zero visual HUD overlays.
- **Don't** embed heavyweight GUI runtimes (Electron, WPF, CEF) into the core background daemon. The UI must remain decoupled in `wiradesk-settings.exe` to enforce minimal daemon memory usage.
- **Do** respect UX Honesty: surface "Not Responding" windows during cycling rather than hiding them.
- **Do** provide high-contrast 2.0 pt keyboard focus rings across both Light and Dark themes for accessibility compliance.
