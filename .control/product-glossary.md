# Product Glossary — Wira Desk

Alphabetical. Terms cite their authoritative source.

| Term | Definition | Source |
| --- | --- | --- |
| Actor / Message-Passing | Software architecture paradigm where each execution unit (hook thread, worker thread, settings process) exclusively owns its mutable state and communicates solely through asynchronous, non-blocking messages. | `ARCHITECTURE-SPINE.md` AD-1 |
| Ghost window | A hidden, transparent, or utility window (e.g. `WS_EX_TOOLWINDOW`, background surrogate, or shell container) that is excluded from the cycling loop. | `FR-5`, `PRD §3` |
| Hook thread | A dedicated OS thread running at `THREAD_PRIORITY_TIME_CRITICAL` executing the low-level keyboard hook callback (`WH_KEYBOARD_LL`); must complete all work within 10 ms. | `NFR-2`, `AD-2`, `PRD §3` |
| LowLevelHooksTimeout | Windows operating system threshold (~300 ms) after which an unresponsive `WH_KEYBOARD_LL` hook is silently removed from the global hook chain by the OS. | `PRD §3` |
| Product Component | A user-named capability boundary in the WDI Method architecture representing a distinct promise to the user (`window-management`, `settings`). | `AGENTS.md`, `WDI Method` |
| Ring buffer | A lock-free, fixed-size 16-slot circular queue of `u8` command bytes bridging the hook thread to the worker thread without heap allocations. | `AD-2`, `NFR-3`, `PRD §3` |
| Spatial preservation | The core guarantee that window cycling and snapping remain strictly confined to the physical monitor and virtual desktop hosting the active foreground window. | `FR-2`, `CAP-7`, `PRD §3` |
| Three-tier error handling protocol | Structured error triage: Tier 1 (fatal startup displays ≤1 modal and exits), Tier 2 (runtime warning logs silently and adds red dot to tray icon), and Tier 3 (runtime hook death renders red cross on tray icon and delivers exactly one desktop notification toast). | `FR-11`, `AD-7`, `PRD §4.7` |
| UIPI | User Interface Privilege Isolation, the Windows security mechanism preventing standard-integrity processes from sending messages or manipulating windows of higher-integrity (elevated) processes. | `FR-8`, `PRD §3` |
| UX honesty | The design principle requiring unresponsive ("Not Responding") windows to be brought to the foreground during a cycle rather than silently skipped, providing transparent feedback to the user. | `FR-4`, `PRD §3` |
| Virtual desktop isolation | Filtering mechanism leveraging `IVirtualDesktopManager` ensuring cycling candidates exist exclusively on the active Windows Virtual Desktop. | `FR-2`, `AD-9`, `CAP-7` |
| Worker thread | The background daemon thread executing live `EnumWindows` traversal, spatial boundary filtering, and focus manipulation off the critical input path. | `AD-2`, `PRD §3` |
| Z-order | The dynamic front-to-back stacking order of desktop windows managed by the Windows Desktop Window Manager (DWM). | `PRD §3`, `AD-3` |
