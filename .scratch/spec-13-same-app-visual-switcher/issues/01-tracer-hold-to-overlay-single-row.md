# 01: Tracer — hold the cycle chord, see one row of live previews, release to activate

**What to build:**
The whole path end to end, one row wide and with no chrome beyond a selection border. Holding
the cycle chord past 150 ms opens an opaque topmost overlay on the active monitor showing DWM
thumbnails of the same windows the blind cycle would visit; releasing the modifiers activates
the highlighted one; `Escape` dismisses and restores the origin window. No pagination, no card
headers, no settings — those are tickets 02, 03 and 04. The switcher is hard-enabled at a
constant threshold for this ticket.

This ticket exists to make the feature demoable on day one rather than after the other three.

**Blocked by:** None

**Status:** done

- [x] The eligible-collection half of `run_context_safe_cycle` is extracted into one function,
      and a test asserts the switcher's candidate set equals the blind cycle's eligible set for
      the same snapshot
- [x] Key-down still enqueues `Cycle` with no added delay (the `SM-1` path is untouched) and
      arms a hold timer on the Worker keyed to the **main key**, not the modifier
- [x] A main-key key-up before the deadline disarms; nothing is drawn and the tap is
      indistinguishable from today's behaviour
- [x] The deadline reached with every modifier in the matched chord still physically down opens
      the overlay, selection on the window the key-down already activated
- [x] Window class `WiraDeskVisualSwitcher`, `WS_POPUP` with
      `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, shown `SW_SHOWNOACTIVATE`, created
      and pumped on the Worker thread — it never takes the foreground, so
      `capture_active_context` still reads the user's application
- [x] `DwmThumbnailHandle` registers one source window and unregisters in `Drop`; a failed
      registration leaves an empty card and neither removes it nor aborts the overlay
- [x] Cards are ordered by `cycle_order`, so `` ` `` always moves the selection to the visually
      next card; a test asserts the two orders agree
- [x] New ring opcodes `SwitcherDisarm`, `SwitcherNext`, `SwitcherPrev`, `SwitcherCommit`,
      `SwitcherCancel` round-trip through `Command::from_u8`, and the navigation opcodes are
      exempt from `ANTI_MACRO_THROTTLE_MS`
- [x] Every modifier in the matched chord released commits through the existing
      `Win32Activator`, then `suppress_start_menu()`; an `InvalidTarget` falls through to the
      next candidate
- [x] `Escape` dismisses and re-activates the window that was foreground before the chord
- [x] While the overlay is open the hook swallows all keyboard input except the chord's own
      modifier releases, and `no_modifier_release_is_ever_swallowed` still passes
- [x] A chord with no modifier never arms the switcher (there is no release to commit on) —
      covered by a test over `Shortcut::has_modifier`
- [x] A VM/RDP bypassed chord never arms the switcher, because arming happens after the bypass
      decision
- [x] Watchdog: a `SetTimer` tick polling `GetAsyncKeyState` commits when the modifiers are
      found up without a key-up having been seen, and an absolute lifetime cap dismisses
      regardless — a topmost overlay stuck on screen with the hook swallowing keys is the worst
      failure this feature can have
- [x] Thumbnail registration and unregistration are counted through a `ThumbnailSink` trait with
      a counting fake; the leak guard is seen failing first (remove the `Drop` impl, watch the
      count go non-zero, restore it)
- [x] All `unsafe` blocks carry `SAFETY:` comments stating the precondition relied on
