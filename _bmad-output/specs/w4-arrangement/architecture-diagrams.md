# Architecture diagrams — W4

Projected from `.how/window-management/06-flows/flow-monitor-move.md` and the spine. Diagrams live
here because the SPEC kernel holds prose only.

## Where each piece of W4 lands

```mermaid
graph TD
    subgraph shared["crates/shared"]
        CMD["commands.rs<br/>+SnapTop 6, SnapBottom 7, MoveToNextMonitor 8"]
        CFG["config.rs<br/>+snap_half_top, snap_half_bottom,<br/>move_next_monitor_shortcut<br/>defaults move to Ctrl+Alt"]
        SC["shortcut.rs<br/>+win+ctrl+left/right reserved"]
    end

    subgraph daemon["crates/daemon"]
        HOOK["hook.rs<br/>Chords struct replaces the 6-tuple<br/>+3 match arms<br/>+duplicate detection at load"]
        DCFG["config.rs<br/>+RejectReason::DuplicateShortcut"]
        WRK["worker.rs<br/>+2 snap arms<br/>+execute_monitor_move"]
        SNAP["arrangement/snap.rs<br/>+split_y, plan_snap_top, plan_snap_bottom"]
        MON["arrangement/monitor.rs NEW<br/>next-monitor selection<br/>proportional remap"]
        SPAT["context/spatial.rs<br/>+EnumDisplayMonitors, per-monitor work area and DPI"]
    end

    subgraph settings["crates/settings"]
        APP["app.rs<br/>ShortcutField 6 to 9"]
        PERS["persistence.rs<br/>+3 fields in validate_config"]
        UI["ui/panes/shortcuts_pane.slint<br/>1 card to 3 labelled cards, 9 rows<br/>ui/main_window.slint pane label"]
    end

    CMD --> HOOK
    CMD --> WRK
    CFG --> HOOK
    CFG --> DCFG
    CFG --> APP
    SC --> PERS
    HOOK --> WRK
    WRK --> SNAP
    WRK --> MON
    MON --> SPAT
    APP --> UI
    APP --> PERS
```

## Story order and why it is serial

```mermaid
graph LR
    S1["S1 Chords struct<br/>no behaviour delta"] --> S2["S2 vertical halves<br/>FR-22"]
    S2 --> S3["S3 monitor move<br/>FR-23"]
    S2 --> S4["S4 duplicate chord<br/>DEC-009"]
    S3 --> S5["S5 Ctrl+Alt family<br/>DEC-008"]
    S5 --> S6["S6 Shortcuts pane<br/>nine rows, three groups"]
```

Five of the six stories touch `daemon/src/hook.rs`, the one file every arrangement command passes
through. Parallel worktrees would collide there, so the wave runs serial. S1 goes alone first
because its shape decision — chord configuration travelling as one struct instead of a six-tuple
and six positional parameters — is what keeps the five behaviour diffs readable. At nine chords,
`clippy::too_many_arguments` fires in a build that runs `-D warnings`, so the refactor is a
prerequisite rather than a courtesy.

## The monitor-move sequence

```mermaid
sequenceDiagram
    participant User
    participant Hook as LC-hook-thread
    participant Ring as ring.rs
    participant Worker as LC-worker-thread
    participant Spatial as context/spatial.rs
    participant Plan as LC-arrangement-engine
    participant Win as Win32

    User->>Hook: Ctrl + Alt + Shift + Enter
    Hook->>Hook: throttle >=50 ms, bypass check
    Hook->>Ring: push(8)
    Hook-->>User: return 1 (swallowed)
    Hook->>Worker: WM_APP_COMMAND_READY
    Worker->>Ring: pop() -> 8
    Worker->>Spatial: resolve_context()
    alt Target is Wira Desk's own window
        Worker->>Worker: Tier-2 diagnostic, nothing moves
    else Eligible
        Worker->>Spatial: EnumDisplayMonitors (fresh, never cached)
        alt One monitor
            Worker->>Worker: empty plan, success, nothing logged
        else Two or more
            opt Window is maximized
                Worker->>Win: restore to normal first
            end
            Worker->>Plan: source work area, destination work area, rect
            Plan->>Plan: rect as share of source, mapped onto destination
            Plan-->>Worker: PlacementPlan
            Worker->>Win: SetWindowPos (SWP_NOACTIVATE | SWP_NOZORDER)
        end
        Worker->>Worker: suppress_start_menu()
    end
```

Enumeration sits on the worker, never in the callback: the callback is budgeted under 10 ms and must
not allocate, and the display set is only needed once a command has been accepted. The hook answers
*whose chord is this*; the worker answers *what is a legal target and where does it go* — the same
split `DEC-006` drew.

## The Shortcuts pane, before and after

```mermaid
graph TD
    subgraph before["Before — 1 card, 6 rows"]
        B1["Switch windows of the same application"] --- B2["Fallback switch shortcut"] --- B3["Snap to left half"] --- B4["Snap to right half"] --- B5["Maximize"] --- B6["Overlapping stack"]
    end
```

```mermaid
graph TD
    subgraph after["After — 3 labelled cards, 9 rows"]
        subgraph g1["Switching"]
            A1["Switch windows of the same application"] --- A2["Fallback switch shortcut"]
        end
        subgraph g2["Snap and resize"]
            A3["Snap to left half"] --- A4["Snap to right half"] --- A5["Snap to top half"] --- A6["Snap to bottom half"] --- A7["Maximize"]
        end
        subgraph g3["Move and arrange"]
            A8["Move to next monitor"] --- A9["Overlapping stack"]
        end
    end
```

The three groups are the three configuration sections already on disk — `[switcher]`, `[snapping]`,
`[layout]` — so what a user sees and what the product stores stop telling different stories. A
fourth pane was refused: the capture lease is armed from which pane is showing, so chord fields in
two panes means two panes arming the observe lease, which regresses the key check.
