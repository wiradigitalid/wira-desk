# Wira Desk Experience

## Foundation

Wira Desk is an ultra-lightweight Windows desktop utility running silently in the background. To preserve strict resource efficiency, the application is split across two decoupled binaries:
- `wiradesk.exe`: Ultra-lightweight background daemon with no GUI interface.
- `wiradesk-settings.exe`: Native-themed interactive UI window for First-Run Onboarding and on-demand Settings configuration.

## Information Architecture

Primary interaction is exclusively driven through global keyboard shortcuts.
Secondary interaction is accessed via right-clicking the System Tray icon to summon the Context Menu:
- Settings...
- View Logs
- Auto-Start *(toggle)*
- Check for Updates...
- About
- Exit

Structure of the Settings Window (`wiradesk-settings.exe`):
- **General**: Auto-start on boot toggle and operational status.
- **Shortcuts (Core Switcher & Snapping)**: Primary same-app switching shortcut (`Win + \``), Alt-fallback shortcut, and snapping combinations (Left/Right/Maximize).
- **Layout (Stack Layout)**: Optional toggle to enable overlapping cascading stack (50% offset) tailored for compact or single-monitor setups, plus stack shortcut configuration.
- **About**: Version information, project links, and diagnostic build metadata.

## Voice and Tone

- **Neutral & Direct**: As a pure system utility, microcopy across the Settings UI is concise, functional, and devoid of unnecessary casual jargon.
- **Solutive Microcopy**: Warning and error messages (such as keyboard hook detachment or elevation issues) directly present actionable remedies (e.g. "Run with Administrator privileges to interact with elevated windows").

## Accessibility Floor

- **Keyboard Navigation**: Every interactive element within the Settings dialog and onboarding flow supports complete keyboard navigation via Tab, Shift+Tab, Space, and Enter keys.
- **Screen Reader Support**: Toggle switches and shortcut input capturers announce their active/inactive and listening states explicitly to screen readers via Windows UI Automation (AccessKit).
- **Theme-Resilient Focus Rings**: Active widget focus strokes maintain high-contrast 2.0 pt outlines across both Light and Dark themes.

## Component Patterns

- **Shortcut Capturer**: When a user focuses or activates a shortcut field, the UI does not accept regular text entry. Instead, it enters an active "Listening" state to intercept the physical key chord pressed next. Pressing Escape cancels listening without overwriting previous bindings.
- **Decoupled Architecture**: Activating "Settings..." from the tray spawns an isolated process (`wiradesk-settings.exe`). The core background daemon is never blocked by GUI rendering threads, guaranteeing uncompromised low-level keyboard hook responsiveness.

## State Patterns

The primary visual indicator is the System Tray icon, communicating runtime operational health:
- **Normal**: Clean default icon. Wira Desk daemon is active and responsive.
- **Warning / Logged (Tier 2)**: Icon with small red alert dot overlay (`#E81123`). Non-fatal errors or fallback events logged silently.
- **Critical / Dead (Tier 3)**: Icon with red cross overlay (`#E81123`). Initialization failure or keyboard hook silently detached by OS timeout. **Accompanied by a Windows Toast Notification** (since Windows 11 hides tray icons in the overflow menu by default, ensuring immediate user awareness when the utility is halted).

## Interaction Primitives

- **Same-App Window Cycling**: Pressing `Win + \`` instantly cycles focus to the next top-level window belonging exclusively to the active foreground application. Replicates native macOS behavior — zero animations, zero HUD overlays, zero visual transitions.
- **Spatial Boundary Locking**: Cycling is strictly confined to the active physical monitor and current virtual desktop. Peripheral monitors remain completely undisturbed.

## Key Flows

### Flow 1: First-Run Onboarding Simulation
1. User launches Wira Desk for the first time without an existing configuration.
2. Instead of staying hidden in the system tray, `wiradesk-settings.exe` launches automatically with `--onboarding`.
3. The interactive simulation guides the user to practice the same-application cycling shortcut (`Win + \``).
4. Simulated dummy windows within the UI shift focus visually to build muscle memory (analogous to macOS gesture tutorials).
5. A prominent "Skip Tutorial" action is available for power users.
6. Upon completion or skipping, onboarding state is persisted, the UI process exits, and Wira Desk operates invisibly in the tray.

### Flow 2: Settings Configuration and Dynamic Reload
1. User right-clicks the System Tray icon and selects "Settings...".
2. `wiradesk-settings.exe` launches on demand, allocating RAM temporarily for GUI rendering.
3. User alters window snapping shortcuts or toggles "Overlapping Stack".
4. User clicks "Save". Settings validates inputs, writes `config.toml` atomically, and signals the daemon via a `WM_APP_RELOAD_CONFIG` Win32 message.
5. The background daemon reloads configuration immediately without restarting.
6. User closes the Settings window. The UI process terminates and system memory is reclaimed.

### Flow 3: Pure Window Cycling (Rian's Multi-Monitor Scenario)
1. User (Rian) presses `Cmd + \`` on an external Mac keyboard (mapped as `Win + \``).
2. Wira Desk captures the keystroke at low-level without input lag.
3. Operating system focus shifts instantly to the adjacent application window on the current physical display.
4. No animations, thumbnails, or task switcher HUDs appear.

### Flow 4: Unresponsive Application Confrontation (Maya's UX Honesty Scenario)
1. User (Maya) presses `Win + \`` to cycle to the next window of a heavy application.
2. The next target window is in a "Not Responding" state.
3. Applying the *UX Honesty* principle, Wira Desk brings the frozen window to the foreground rather than silently skipping it.
4. Maya observes the "(Not Responding)" title bar and can take appropriate action (e.g. terminating the hung task or pressing `Win + \`` again to advance to the next window).

### Flow 5: Elevated UIPI Focus Transfer (Budi's SysAdmin Scenario)
1. User (Budi) is actively working in Task Manager or an Administrator Command Prompt.
2. Budi presses `Win + \``.
3. Because the Wira Desk background daemon runs with Administrator privileges (`requireAdministrator`), it successfully captures the shortcut and transfers focus across UIPI privilege boundaries without operating system blocks.
