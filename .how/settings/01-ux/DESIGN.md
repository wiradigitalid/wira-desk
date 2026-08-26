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
  2. `Shortcuts`: **Every** editable chord, and the only pane that holds one. Nine rows in three labelled card groups — *Switching* (2), *Snap & resize* (5), *Move & arrange* (2) — which scroll, above a **pinned** Key check band that does not.
  3. `Layout`: The overlapping stack toggle and its width slider. No chord lives here.
  4. `VM & Exceptions`: Passthrough rules for virtualization guests (`mstsc.exe`, `vmconnect.exe`, `VMwareUnityWindow`).
  5. `About`: Diagnostic build metadata, version info, active font rendering details, and memory footprint.
- **Modular Grouping**: Each configuration group is rendered within a rounded Card Container (`#20242B`, radius 8px) with fine divider lines.
- **Group Heading**: Inside the Shortcuts pane, each card carries a heading above it in the caption size, uppercase, letter-spaced, in the secondary text colour — the same treatment the About pane already uses for its metadata labels. A heading names a group; it is never itself interactive and never carries a chord.
- **Pinned Key check band**: On the Shortcuts pane only, the Key check card sits in its own band between the scroll area and the save footer, separated from the rows by a 1 px hairline in the card stroke colour. It keeps its natural height and the scroll area gives up space to it, never the reverse. It is pinned because it is a *live* instrument: nine rows are taller than the window, so at the end of the pane it was below the fold exactly when a user was pressing chords to test them. The band appears on no other pane, because the daemon arms its observe lease from which pane is showing and a reading is only meaningful here.

**Why every chord sits in one pane.** An earlier draft of this document put the snapping chords under a `Layout & Snapping` pane and the switcher chords under `Shortcuts`, and the shipped build never did that — it has always drawn all of them in `Shortcuts`. The build is right and this document was wrong, for a reason outside taste: the daemon's capture lease is armed from *which pane is showing* (`DEC-004`), so shortcut fields living in two panes means two panes have to arm the observe lease. `DEC-004` and `DEC-005` are both built on the lease having one owning place, and splitting it is a regression in the key check rather than a tidier menu. The pane's third name also stops being a lie: `Layout` holds layout settings, and it holds no chords.

## UI Components

### 1. System Tray Icon & Context Menu
- **Asset Dimensions**: 16×16 and 32×32 pixel `.ico` formats supporting multi-DPI displays.
- **Menu Hierarchy**: Settings..., View Logs, Auto-Start (toggle), Check for Updates..., About, Exit.

### 2. Frameless Settings Window (`wiradesk-settings.exe`)
- **Structure**: Modern frameless shell (`with_decorations(false)`) with custom caption bar (`36 px`, `#15181E`) containing draggable region, minimize (`—`), and close (`✕`) buttons.
- **Vertical Sidebar**: Fixed-width navigation column (`175 px`) with active blue indicator pill (`#4CC2FF`, `3.5 × 20 px`).
- **Pill Toggle Switch (`fluent_toggle_switch`)**: Interactive animated toggle switch widget (`40 × 20 px`) with smooth state transitions.
- **Shortcut Capturer Control**: Dedicated interactive listening widget with clear auditory/screen-reader announcements and Escape key cancellation.
- **Shortcut Row**: One row per editable action — title, one-line description, the current chord rendered as key names joined by `+`, and, when the chord collides with another action, an inline refusal naming the other action plus a Swap affordance. Nine rows exist; the row is the unit, and an action never appears as two rows.
- **Save Bar**: Sticky footer (`48 px`) with Revert and Save Changes buttons, validation error summary, and instantaneous IPC status reflection.

### 3. First-Run Onboarding Modal Dialog
- **Modal Tutorial Shell**: Frameless interactive practice arena (`580 × 380 px`, `with_decorations(false)`, `.with_transparent(true)`) with top 48 px drag area and symmetric 28 px padding.
- **Progress Bar**: 3-segment balanced indicator displaying *Welcome*, *Try Switching*, and *Done*.
- **Simulated Windows**: Dual responsive dummy window cards (50% split with 12 px gap) that alternate visual focus states when the user presses `Win + \``.
- **Navigation Flow**: Step 1 (Skip / Next), Step 2 (Back / Next), Step 3 (Back / Start Using Wira Desk), with buttons pinned at exact bottom margin 24 px.

## Do's and Don'ts

- **Do** keep window cycling 100% invisible: zero delay, zero animation, zero visual HUD overlays.
- **Don't** embed heavyweight GUI runtimes (Electron, WPF, CEF) into the core background daemon. The UI must remain decoupled in `wiradesk-settings.exe` to enforce minimal daemon memory usage.
- **Do** respect UX Honesty: surface "Not Responding" windows during cycling rather than hiding them.
- **Do** provide high-contrast 2.0 pt keyboard focus rings across both Light and Dark themes for accessibility compliance.
- **Do** render every chord through the single display formatter, so `down` shows as `↓` in every readout including the Key check. Building a second display path is what once made the Key check print the word `down` while every row above it showed the glyph.
- **Don't** reorder rows to suit a group heading. The row order is one declared sequence that also decides which action wins a chord collision (`LBR-ST-14`); a heading may gather rows, never move them.
- **Don't** put a chord field in any pane other than `Shortcuts`. See the note under Layout & Navigation Hierarchy.
