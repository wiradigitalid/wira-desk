# Wira Desk Experience

## Foundation

Wira Desk is an ultra-lightweight Windows desktop utility running silently in the background. To preserve strict resource efficiency, the application is split across two decoupled binaries:
- `wiradesk.exe`: Ultra-lightweight background daemon with no GUI interface (<2MB RAM).
- `wiradesk-settings.exe`: Native-themed interactive UI window for First-Run Onboarding and on-demand Settings configuration running modern Fluent 2 Mica styling.

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
- **General**: Auto-start on boot toggle (Task Scheduler integration), Spatial Lock, Virtual Desktop isolation, and UX Honesty controls.
- **Shortcuts**: Every editable chord, and the only pane that holds any. Nine rows in three labelled groups that scroll, above a Key check readout pinned in place:

  | Group | Rows | Reads as |
  | --- | --- | --- |
  | Switching | Switch windows of the same application · Fallback switch shortcut | Which window has focus |
  | Snap & resize | Snap to left half · right half · top half · bottom half · Maximize | One window, one monitor, a fraction of it |
  | Move & arrange | Move to next monitor · Overlapping stack | More than one window, or more than one screen |

  The groups are the three configuration sections the product already keeps on disk, so what a user sees and what the product stores stop telling different stories. The Key check readout stays put while they scroll: it reports what the keyboard just did, so it has to be readable at the moment a chord is pressed rather than wherever the list happens to end. Inside *Snap & resize* the rows follow the arrow keys — left, right, top, bottom — with Maximize last as the "all of it" case; the order is the declared sequence, not the enum's numbering, and grouping never reorders it.
- **Layout**: The overlapping stack toggle and its width slider. No chord lives here — the pane was called *Layout & Snapping* while holding no snapping control at all.
- **VM & Exceptions**: Virtualization and Remote Desktop passthrough rules (`mstsc.exe`, `vmconnect.exe`, `VMwareUnityWindow`).
- **About**: Version information, project links, active typeface loader status, and diagnostic build metadata.

## Voice and Tone

- **Neutral & Direct**: As a pure system utility, microcopy across the Settings UI is concise, functional, and devoid of unnecessary casual jargon.
- **Solutive Microcopy**: Warning and error messages (such as keyboard hook detachment or elevation issues) directly present actionable remedies (e.g. "Run with Administrator privileges to interact with elevated windows").

## Accessibility Floor

- **Keyboard Navigation**: Every interactive element within the Settings dialog and onboarding flow supports complete keyboard navigation via Tab, Shift+Tab, Space, and Enter keys.
- **Screen Reader Support**: Toggle switches and shortcut input capturers announce their active/inactive and listening states explicitly to screen readers via Windows UI Automation (AccessKit).
- **Theme-Resilient Focus Rings**: Active widget focus strokes maintain high-contrast 2.0 pt outlines across both Light and Dark themes.

## Component Patterns

- **Frameless Window Shell**: The settings and onboarding windows feature a modern frameless custom titlebar (`with_decorations(false)`) with in-app minimize (`—`) and close (`✕`) caption controls and native window drag handling.
- **Fluent 2 Animated Toggle**: Interactive animated pill switches (`fluent_toggle_switch`) providing immediate tactile and visual state confirmation.
- **Scroll hint**: When a pane holds more than fits, a chevron appears at the bottom edge of the scroll area and bobs gently. It shows only while **both** are true — the area is genuinely scrollable, and the reader has not scrolled yet — and fades out the moment either stops holding. Both halves earn their place: without the first it invites a scroll that does nothing, and without the second it keeps nagging after the reader has already complied, which is what makes the same pattern irritating on a landing page. It is an addition to the scrollbar, not a replacement: Fluent's auto-hiding scrollbar is easy to miss, and the Shortcuts pane's nine rows are where that was first noticed.
- **Shortcut Capturer**: When a user focuses or activates a shortcut field, the UI does not accept regular text entry. Instead, it enters an active "Listening" state to intercept the physical key chord pressed next. Pressing Escape cancels listening without overwriting previous bindings.
- **Decoupled Architecture**: Activating "Settings..." from the tray spawns an isolated process (`wiradesk-settings.exe`). The core background daemon is never blocked by GUI rendering threads, guaranteeing uncompromised low-level keyboard hook responsiveness.

## State Patterns

The primary visual indicator is the System Tray icon, communicating runtime operational health:
- **Normal**: Clean default icon. Wira Desk daemon is active and responsive.
- **Warning / Logged (Tier 2)**: Icon with small red alert dot overlay (`#E81123`). Non-fatal errors or fallback events logged silently.
- **Critical / Dead (Tier 3)**: Icon with red cross overlay (`#E81123`). Initialization failure or keyboard hook silently detached by OS timeout. **Accompanied by a Windows Toast Notification** (since Windows 11 hides tray icons in the overflow menu by default, ensuring immediate user awareness when the utility is halted).

States a shortcut row can be in:

| State | What the user sees | What they can do |
| --- | --- | --- |
| Resting | The current chord, rendered as key names — `Ctrl + Alt + ↓` | Activate the row to rebind it |
| Listening | The row says it is listening instead of showing a chord, and announces that state to a screen reader rather than relying on the visual change | Press a chord, or Escape to cancel and keep the old one |
| Refused — grammar | An inline message on **this** row saying what is wrong: a missing modifier, an unknown key, more than one main key | Press a different chord. Save stays available |
| Refused — reserved | An inline message naming what Windows uses the chord for, and — where the chord is merely policy-refused rather than impossible — suggesting adding Ctrl | Press a different chord |
| Collides | Inline messages on **both** rows, each naming the other action, plus a Swap affordance on the row that was just changed | Swap the two, change one, or submit and be refused with both names |
| Empty | No shortcut row is ever empty. A chord field always holds a value; an action with no reachable chord is the **unbound** case below, which is a daemon-side condition and not something this pane can currently show | — |

An **unbound** action — one the daemon has left unreachable because another action ahead of it holds the same chord (`BR-6`) — has no representation in this pane today. The user sees the tray Warning dot and, if they open the log, a line naming both fields. `DEC-009` records this silence as a cost and names showing it here as the route out; it is deliberately not designed in this pass.

## Interaction Primitives

- **Same-App Window Cycling**: Pressing `Win + \`` instantly cycles focus to the next top-level window belonging exclusively to the active foreground application. Replicates native macOS behavior — zero animations, zero HUD overlays, zero visual transitions.
- **Spatial Boundary Locking**: Cycling is strictly confined to the active physical monitor and current virtual desktop. Peripheral monitors remain completely undisturbed.
- **Half-Screen Snapping**: `Ctrl + Alt` with an arrow key puts the active window in that half of the current monitor's work area — left, right, top, or bottom. Pressing the same chord twice changes nothing, because the division is recomputed from the work area rather than from where the window currently sits.
- **Deliberate Monitor Movement**: `Ctrl + Alt + Shift + Enter` moves the active window to the next monitor and it keeps the same *share* of the screen, not the same pixel size. This is the one command that crosses the monitor boundary on purpose; the virtual desktop boundary is never crossed. With one monitor attached it does nothing at all — no movement, no message.

## Key Flows

### Flow 1: First-Run Onboarding Simulation
1. User launches Wira Desk for the first time without an existing configuration.
2. Instead of staying hidden in the system tray, `wiradesk-settings.exe` launches automatically with `--onboarding`.
3. The interactive simulation presents a compact frameless modal dialog (`520 × 400 px`) guiding the user to practice the same-application cycling shortcut (`Win + \``).
4. Simulated dummy windows within the UI shift focus visually to build muscle memory.
5. A prominent "Skip Tutorial" action is available for power users (also bound to Escape).
6. Upon completion or skipping, onboarding state is persisted, the UI process exits, and Wira Desk operates invisibly in the tray.

### Flow 2: Settings Configuration and Dynamic Reload
1. User right-clicks the System Tray icon and selects "Settings...".
2. `wiradesk-settings.exe` launches on demand as a frameless 5-pane settings shell.
3. User alters window snapping shortcuts or adjusts the "Overlapping Stack" width ratio slider.
4. User clicks "Save Changes". Settings validates inputs, writes `config.toml` atomically, and signals the daemon via a `WM_APP_RELOAD_CONFIG` Win32 message.
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

### Flow 6: Building a Two-Screen Review Layout (Sari's Docked Scenario — UJ-4)
1. User (Sari) is on a 14-inch laptop with a larger external monitor to its right, the two running at different display scaling. A specification and a terminal are both on the laptop screen; a browser is on the external monitor.
2. Sari presses `Ctrl + Alt + Up` on the specification. It takes the top half of the laptop's work area — the useful division on a screen too short for left and right halves to help.
3. She focuses the terminal and presses `Ctrl + Alt + Down`. It takes the bottom half. The two halves meet exactly, with no gap and no overlap.
4. She decides the specification belongs on the larger display and presses `Ctrl + Alt + Shift + Enter`.
5. **Climax:** the specification arrives on the external monitor still occupying the top half — the same *share* of the work area, not the same pixel height — so the layout she just built survives the move. The browser and the terminal have not moved.
6. Sari works across both screens with a layout assembled from the keyboard in seconds, and rebuilds it the same way each time she docks.
7. Undocked, with only the laptop screen attached, step 4 does nothing at all: no window jump, no message, no error. The snap chords keep working.
