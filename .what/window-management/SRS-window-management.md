---
type: srs
component: window-management
status: reviewed
created: 2026-08-21
updated: 2026-08-21
satisfies: [FR-1, FR-2, FR-3, FR-4, FR-5, FR-6, FR-8, FR-9, FR-10, FR-11, FR-12, FR-14, FR-15]
reviewed:
  date: '2026-08-21'
  sha: pending
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — window-management

## Decision Summary

The `window-management` component delivers instant, overlay-free same-application window cycling and DPI-aware window arrangement on Windows 10 and 11. Running as an elevated background system tray utility (`wiradesk.exe`), it intercepts global keyboard shortcuts, strictly enforces monitor and virtual desktop boundaries, brings unresponsive windows honestly to the foreground, and recovers the system tray icon automatically after shell restarts without cloud dependencies or background CPU bloat.

## Why

Users manage multiple windows within the same application (multiple browser sessions, code editors, document drafts) and expect immediate, muscle-memory cycling without the cognitive noise of full task switchers or multi-monitor focus jumps. Isolating core window cycling and snapping inside a dedicated, headless daemon container protects input latency (<10 ms hook duration) and guarantees a static RAM footprint under 2 MB.

## Actor Register

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Power User | Desktop user managing multiple windows of the same application across multi-monitor or virtual desktop workspaces. | Trigger same-app cycling, snap active windows, access tray menu, open diagnostic logs. |
| New User | First-time user running Wira Desk on Windows. | Experience default cycling and snapping shortcuts without opening configuration. |
| Sysadmin | System administrator operating standard and elevated command shells or administrative tools. | Cycle seamlessly between standard and elevated administrator windows without UIPI refusal. |

## UC Catalogue

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-1 | Cycle to the next window of the same app on this monitor | Power User | FR-1, FR-2, FR-3, FR-4, FR-5, FR-6 | no |
| UC-2 | Snap the active window to the left or right half of the screen | Power User | FR-14 | no |
| UC-3 | See the tray icon return after Windows Explorer restarts | Power User | FR-10, FR-11, FR-12 | no |

## Constraints

- Must adhere strictly to Architecture Spine invariants AD-1 through AD-10 and AD-12.
- Must execute live, stateless Z-order window enumeration via `EnumWindows` on every keypress without caching Z-order (AD-3).
- Must restrict window enumeration to the non-blocking kernel APIs named in the spine's sterilization convention (`IsWindowVisible`, `GetWindowLongPtrW`, `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`, `GetClassNameW`) and never a blocking `SendMessage` or `GetWindowText`, executing off the hook thread on the worker thread (NFR-4, AD-2).
- Must run with elevated Administrator privileges via application manifest (`requireAdministrator`) to guarantee UIPI focus control (FR-8).
- Must maintain a static RAM footprint under 2 MB idle (NFR-1) and release binary size under 500 KB (NFR-5).
- Must not watch configuration files on disk; configuration reload occurs exclusively via explicit `WM_APP_RELOAD_CONFIG` IPC message (BR-1, AD-5).
- Must bypass shortcut interception when foreground window is a known virtual machine or remote desktop client (FR-3, AD-6).

## Non-Goals

- Providing visual switcher HUDs, thumbnail previews, or overlay window task bars (explicitly invisible switching).
- Modifying keyboard shortcuts or configuring onboarding tutorial settings (delegated to `settings` component).
- Automated tiling window management (e.g. auto-tiling tree layouts like i3 or Komorebi).
- Cross-machine cloud synchronization or remote telemetry collection.

## Prerequisite

- Supported 64-bit Windows 10 (1809+) or Windows 11 operating system environment.
- Platform runtime data structures and entities available (`app-config`, `ipc-reload-signal`, `runtime-paths`).
- Process elevated with Administrator privileges.

## Success Signal

Pressing `Win + \`` immediately shifts keyboard focus to the next visible, same-application window on the active physical monitor and virtual desktop, with the hook callback returning in under 10 ms (NFR-2, NFR-3), zero visual flicker, zero cross-monitor jump, and full support for elevated console windows.

## Assumptions, Risks, and To Be Confirmed

### Assumptions
- Users accept granting Administrator elevation upon installation to achieve seamless UIPI traversal across administrative windows.
- Standard virtual machine and remote desktop clients expose recognizable executable or class names (`mstsc.exe`, `vmconnect.exe`) suitable for passthrough detection.

### Risks
- Third-party security software or antivirus suites could flag low-level keyboard hooks (`WH_KEYBOARD_LL`), handled via Tier 1 startup fatal protocol.
- Windows OS `LowLevelHooksTimeout` (~300 ms) unhooking the hook thread if blocked, mitigated by sub-10 ms hook execution and off-thread worker processing.

### To Be Confirmed
- None; all architectural invariants and functional requirements are stated and traced. Ratification is the owner's act at G3, recorded in `gates_passed`.

## Gate Checklist · [G3]

- ★ Every functional requirement (FR-1..6, FR-8..12, FR-14..15) mapped to a usecase or carries explicit `no_uc:` justification? Yes.
- ★ All use case titles phrased as natural user sentences? Yes.
- ★ Actor Register complete and aligned with PRD journeys? Yes.
- ★ Invariants AD-1..10 and cross-component business rules BR-1..5 respected? Yes.

## Design Reference · [G3]

Paired SDD: `.how/window-management/SDD-window-management.md`. Binds invariants AD-1 through AD-10 and AD-12.

---

## Slots

- `02-rules/rules-window-management.md`: Local component rules (LBR-WM-1..5).
- `03-domain/domain-model.md`: Conceptual domain entities (`hook-command`, `window-focus-state`, `tray-health-state`, `arrangement-command`).
- `03-domain/state-machines.md`: Hook lifecycle and tray health state machine transitions.
- `04-usecases/`: Detailed step-by-step flows (`UC-1-cycle-same-app-window.md`, `UC-2-snap-window-half.md`, `UC-3-tray-recovery-after-explorer.md`).
- `05-scenarios/`: Edge-case branching scenarios (`SCN-01-vm-bypass-during-cycle.md`, `SCN-02-hook-death-recovery-attempt.md`).

## Open Items

*(No open items; questions resolved in `.control/questions/answered.md`)*
