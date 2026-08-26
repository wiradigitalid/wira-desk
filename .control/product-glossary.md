# Product Glossary — Wira Desk

Alphabetical. Terms cite their authoritative source.

| Term | Definition | Source |
| --- | --- | --- |
| Actor / Message-Passing | Software architecture paradigm where each execution unit (hook thread, worker thread, settings process) exclusively owns its mutable state and communicates solely through asynchronous, non-blocking messages. | `ARCHITECTURE-SPINE.md` AD-1 |
| Ghost window | A hidden, transparent, or utility window (e.g. `WS_EX_TOOLWINDOW`, background surrogate, or shell container) that is excluded from the cycling loop. | `FR-5`, `PRD §3` |
| Next monitor | The monitor that follows the active window's own in one fixed enumeration order, wrapping from the last back to the first. Not the nearest monitor and not a monitor named by position, because coordinate order is undefined for stacked or L-shaped arrangements. | `PRD §4.11`, `DEC-007` |
| Hook thread | A dedicated OS thread running at `THREAD_PRIORITY_TIME_CRITICAL` executing the low-level keyboard hook callback (`WH_KEYBOARD_LL`); must complete all work within 10 ms. | `NFR-2`, `AD-2`, `PRD §3` |
| LowLevelHooksTimeout | Windows operating system threshold (~300 ms) after which an unresponsive `WH_KEYBOARD_LL` hook is silently removed from the global hook chain by the OS. | `PRD §3` |
| Product Component | A user-named capability boundary in the WDI Method architecture representing a distinct promise to the user (`window-management`, `settings`). | `AGENTS.md`, `WDI Method` |
| Ring buffer | A lock-free, fixed-size 16-slot circular queue of `u8` command bytes bridging the hook thread to the worker thread without heap allocations. | `AD-2`, `NFR-3`, `PRD §3` |
| Spatial preservation | The core guarantee that window cycling and snapping remain strictly confined to the physical monitor and virtual desktop hosting the active foreground window. | `FR-2`, `CAP-7`, `PRD §3` |
| Proportional placement | Placing a window on a destination monitor by the share of the work area it occupied on the monitor it left, rather than by copying its pixel width and height. What makes an arrangement survive a move between monitors of different size or display scaling. | `FR-23`, `DEC-007` |
| Three-tier error handling protocol | Structured error triage: Tier 1 (fatal startup displays ≤1 modal and exits), Tier 2 (runtime warning logs silently and adds red dot to tray icon), and Tier 3 (runtime hook death renders red cross on tray icon and delivers exactly one desktop notification toast). | `FR-11`, `AD-7`, `PRD §4.7` |
| Unbound action | An arrangement or switching action that currently has no chord that reaches it, because the chord it was configured with is already claimed by an action ahead of it in the fixed precedence order. It stays unreachable, and no other action fires in its place, until the user changes one of the two. | `DEC-009`, `BR-6` |
| UIPI | User Interface Privilege Isolation, the Windows security mechanism preventing standard-integrity processes from sending messages or manipulating windows of higher-integrity (elevated) processes. | `FR-8`, `PRD §3` |
| UX honesty | The design principle requiring unresponsive ("Not Responding") windows to be brought to the foreground during a cycle rather than silently skipped, providing transparent feedback to the user. | `FR-4`, `PRD §3` |
| Virtual desktop isolation | Filtering mechanism leveraging `IVirtualDesktopManager` ensuring cycling candidates exist exclusively on the active Windows Virtual Desktop. | `FR-2`, `AD-9`, `CAP-7` |
| Work area | The usable region of one monitor, with the taskbar and any reserved appbar already excluded. Every arrangement is planned inside it, never inside full monitor bounds — a maximized window that reached monitor bounds would cover the taskbar. | `FR-14`, `PRD §4.5`, `daemon/arrangement/mod.rs` |
| Worker thread | The background daemon thread executing live `EnumWindows` traversal, spatial boundary filtering, and focus manipulation off the critical input path. | `AD-2`, `PRD §3` |
| Z-order | The dynamic front-to-back stacking order of desktop windows managed by the Windows Desktop Window Manager (DWM). | `PRD §3`, `AD-3` |
