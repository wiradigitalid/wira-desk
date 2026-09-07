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

Follows Windows 11 dialog spacing with a structured 4-pane vertical sidebar within the decoupled executable (`wiradesk-settings.exe`), per `DEC-014`:
- **Panes (4)**:
  1. `General`: Startup integration (Task Scheduler), Spatial Lock, Virtual Desktop isolation, and UX Honesty controls.
  2. `Shortcuts`: **Every** editable chord, and the only pane that holds one — plus the one non-chord control that used to justify a `Layout` pane of its own (the Overlapping Stack width percentage; see below). Sixteen rows in five labelled card groups, taxonomy per `DEC-014` — *Switching* (2) · *Snap to half* (4) · *Snap to third* (3) · *Snap to custom* (4) · *Resize, move & arrange* (3: Maximize, Move to next monitor, Overlapping Stack) — which scroll, above a **pinned** Key check band that does not.
  3. `VM & Exceptions`: Passthrough rules for virtualization guests (`mstsc.exe`, `vmconnect.exe`, `VMwareUnityWindow`).
  4. `About`: Diagnostic build metadata, version info, active font rendering details, and memory footprint.
- **Modular Grouping**: Each configuration group is rendered within a rounded Card Container (`#20242B`, radius 8px) with fine divider lines.
- **Group Heading**: Inside the Shortcuts pane, each card carries a heading above it in the caption size, uppercase, letter-spaced, in the secondary text colour — the same treatment the About pane already uses for its metadata labels. A heading names a group; it is never itself interactive and never carries a chord. A heading also now carries meaning a row's title no longer has to repeat: under *Snap to custom*, a row reads `Snap to left edge`, not `Snap to left edge (custom %)` — the group name already says "custom", and restating it on every row is what the earlier three-group taxonomy needed and this one does not.
- **Scroll hint**: A 34 px veil at the bottom edge of the scroll area — `transparent` to `bg_mica` over the lower 78% — with a chevron centred in it in `accent_primary`, bobbing 3 px either side of its resting line on a 900 ms ease-in-out. It fades in and out over 200 ms and its timer does not run while it is hidden. The chevron is **drawn as a path, never a font glyph**: neither loaded font carries a chevron codepoint, so a glyph would render as a box. The veil is a gradient rather than a solid strip because a hard edge reads as "the list ends here", which is the opposite of what the hint says.
- **Pinned Key check band**: On the Shortcuts pane only, the Key check readout sits in its own band between the scroll area and the save footer, separated from the rows by a 1 px hairline in the card stroke colour. It carries **no card of its own** — no background, no border, no radius: it is the only thing in the band, so a boundary would be drawn against nothing, and its padding stacked on the band's cost roughly 30 px of height for no information. The band owns the inset; the readout owns none. It keeps its natural height and the scroll area gives up space to it, never the reverse. It is pinned because it is a *live* instrument: sixteen rows are taller than the window, so at the end of the pane it was below the fold exactly when a user was pressing chords to test them. The band appears on no other pane, because the daemon arms its observe lease from which pane is showing and a reading is only meaningful here.
- **Row description as tooltip, not a truncated line.** A row's one-line description used to draw permanently under its title, elided with `…` whenever it overflowed the row's width — which on several rows was always, so the ellipsis was the only thing some users ever saw of it. The description is now a hover/focus tooltip on the row's title-and-description block instead: it carries the full text every time, and the row itself draws nothing extra when the pointer or focus is elsewhere.
- **Row vertical centring.** A row's right-hand cluster — its percentage stepper (where present), its keycap, and its enable toggle — is one visually centred group against the row's own height, not three independently positioned children. The toggle in particular must sit on the same vertical middle as the keycap beside it; a toggle that reads higher than its neighbouring keycap is the defect this line exists to prevent.
- **No horizontal scroll.** Five groups and sixteen rows must fit the pane's working width without the shell growing a horizontal scrollbar. Where the row's right-hand cluster (stepper + keycap + toggle) is too wide for the window's default size, the fix is narrowing that cluster's own layout, not letting the pane overflow sideways — a Windows settings dialog scrolls vertically only.

**Why every chord sits in one pane.** An earlier draft of this document put the snapping chords under a `Layout & Snapping` pane and the switcher chords under `Shortcuts`, and the shipped build never did that — it has always drawn all of them in `Shortcuts`. The build is right and this document was wrong, for a reason outside taste: the daemon's capture lease is armed from *which pane is showing* (`DEC-004`), so shortcut fields living in two panes means two panes have to arm the observe lease. `DEC-004` and `DEC-005` are both built on the lease having one owning place, and splitting it is a regression in the key check rather than a tidier menu.

`DEC-014` carries this one step further: the `Layout` pane held exactly one control — `stack_width_percent`, never a chord — and once Overlapping Stack's own on/off toggle moved to `Shortcuts` (`SPEC-3-01`), a whole pane survived to hold one percentage field. That field moves inline onto the Overlapping Stack row instead, the same `has_percent` pattern the four `Snap to custom` rows already use, and the `Layout` pane is retired rather than kept alive for a control that fits its sibling row's own shape.

## UI Components

### 1. System Tray Icon & Context Menu
- **Asset Dimensions**: 16×16 and 32×32 pixel `.ico` formats supporting multi-DPI displays.
- **Menu Hierarchy**: Settings..., View Logs, Auto-Start (toggle), Check for Updates..., About, Exit.

### 2. Frameless Settings Window (`wiradesk-settings.exe`)
- **Structure**: Modern frameless shell (`with_decorations(false)`) with custom caption bar (`36 px`, `#15181E`) containing draggable region, minimize (`—`), and close (`✕`) buttons.
- **Vertical Sidebar**: Fixed-width navigation column (`175 px`) with active blue indicator pill (`#4CC2FF`, `3.5 × 20 px`).
- **Pill Toggle Switch (`fluent_toggle_switch`)**: Interactive animated toggle switch widget (`40 × 20 px`) with smooth state transitions.
- **Shortcut Capturer Control**: Dedicated interactive listening widget with clear auditory/screen-reader announcements and Escape key cancellation.
- **Shortcut Row**: One row per editable action — title, one-line description, the current chord rendered as key names joined by `+`, and, when the chord collides with another action, an inline refusal naming the other action plus a Swap affordance. Sixteen rows exist; the row is the unit, and an action never appears as two rows.
  - The keycap is `135 px` **minimum** and grows to its own label plus `12 px` either side. It is not a fixed width: `Ctrl + Alt + Shift + Enter` does not fit in 135 px, and the label does not clip, so a fixed keycap drew its text past both rounded ends.
  - A row is as tall as its description, so the buttons are centred on the row's vertical middle rather than pinned to a fixed offset. Two-line descriptions are normal and must not push the keycap off centre.
  - The listening state's dot is **drawn**, not an emoji, for the same reason the scroll chevron is: the fonts do not carry it, and this is the state a user reaches on every rebind.
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
- **Don't** reach for a font glyph for an icon or a symbol. Both loaded fonts cover little beyond Latin and the arrows already in use; verify against the font's cmap, or draw a path.
