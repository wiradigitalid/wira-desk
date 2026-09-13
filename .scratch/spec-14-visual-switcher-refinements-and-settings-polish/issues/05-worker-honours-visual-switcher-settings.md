# 05: Worker honours `visual_enabled` and `visual_hold_delay_ms`

**What to build:**
Disabling "Enable Visual Switcher" and saving does not stop the overlay opening. Before writing any
code, read `SPEC.md` §3 — the first draft of this ticket named the wrong root cause, and the wrong
cause has a **green test defending it**.

The short version: the Hook already holds `visual_enabled` (`config.rs:32`, `hook.rs:1881`) and
already refuses to arm on it (`hook.rs:627`). But `rt.switcher_armed` feeds only
`HookRuntime::check_switcher_deadline` (`hook.rs:1349`), which is `#[allow(dead_code)]` with exactly
one caller in the workspace — a test (`hook.rs:2676`). The overlay actually opens from the Worker:
`execute_cycle` (`worker.rs:466`) arms `TIMER_SWITCHER_HOLD` whenever a modifier is held
(`worker.rs:492-506`), hardcoded to `150` ms, with no `visual_enabled` check at all.

So:

1. **Carry the settings to the Worker.** Add `visual_enabled: bool` and `visual_hold_delay_ms: u32`
   to `WorkerSnapshot` (`config.rs:42`) and populate them at `config.rs:226`. Delivery plumbing
   already exists (`install_config_snapshot`, `worker.rs:801`) — nothing new is needed there.
   **Update `WorkerSnapshot`'s doc-comment**: it currently argues the Worker reads layout only and
   explains why carrying anything else would be "a field nothing reads". Leaving that in place makes
   it a lie the next reader will trust.

2. **Fix the cold-start fallback.** `worker_snapshot()` (`worker.rs:805-817`) rebuilds a snapshot
   from `Config::load_or_default` when no reload has arrived. Both new fields must be carried there
   too, or a daemon that has never taken a reload behaves as "enabled" whatever is on disk.

3. **Gate the live path.** In `execute_cycle`: when `visual_enabled` is false, neither arm
   `TIMER_SWITCHER_HOLD` nor set `SWITCHER_HOLD_ORIGIN`/`SWITCHER_CHORD_MODS`. When true, pass
   `visual_hold_delay_ms` to `SetTimer` in place of the literal `150`. Clamp defensively to
   `100..=500` the way `hook.rs:631-636` does — `config::validate` already rejects out-of-range
   values, but the cold-start path in step 2 does not go through `validate`.

4. **Give it a seam.** `execute_cycle` calls `SetTimer` inline and cannot be tested. Extract the
   decision as a pure function — outcome + modifier state + enabled flag + delay in, `Option<u32>`
   delay out — and test that.

5. **Resolve the dead Hook gate.** Either wire `check_switcher_deadline` into production or delete it
   along with `switcher_armed`/`switcher_deadline_ms` if nothing else reads them. Leaving a gated
   dead path next to an ungated live one is how this bug survived a test written specifically to
   catch it.

6. **Gate the timer that is already in flight.** Gating `execute_cycle` closes the door on *new*
   arming and nothing else. `install_config_snapshot` (`worker.rs:801`) is a plain assignment — it
   does not `KillTimer`. A user who holds the chord, then saves `visual_enabled = false` inside the
   hold window, still gets the overlay: `handle_timer`'s `TIMER_SWITCHER_HOLD` arm (`worker.rs:206`)
   checks only `are_chord_modifiers_down` and calls `open_visual_switcher` regardless. The window is
   100–500 ms wide, so this is rare rather than impossible — and rare-and-silent is the shape of the
   bug this whole ticket exists to fix. Either re-read the live snapshot inside the timer arm, or
   `KillTimer(TIMER_SWITCHER_HOLD)` on reload. Pick one and say which.

**See the guard fail before you fix it.** Per `CLAUDE.md`: write the new test against today's code,
watch it go red, then fix. `hook::tests::reload_with_visual_switcher_disabled_leaves_blind_cycling_unchanged`
is green **right now**, against the bug the owner reported — a test that asserts on
`rt.switcher_armed` proves nothing about whether the overlay opens. Run the suite with
`--no-fail-fast`.

**Blocked by:** None

**Status:** ready-for-agent

- [ ] `WorkerSnapshot` carries `visual_enabled: bool` and `visual_hold_delay_ms: u32`, and its doc-comment describes what it now holds.
- [ ] `worker_snapshot()`'s cold-start fallback populates both fields from the on-disk config.
- [ ] With `visual_enabled == false`, `execute_cycle` arms no hold timer and stores no switcher origin; blind cycling is byte-for-byte unchanged.
- [ ] With `visual_enabled == true`, the hold timer uses the configured `visual_hold_delay_ms`, clamped to `100..=500`.
- [ ] Saving `visual_enabled = false` from Settings stops the overlay opening without a daemon restart and without re-registering the keyboard hook.
- [ ] A reload arriving **between** arming and the hold timer firing does not open the overlay — asserted against the timer arm, not only against `execute_cycle`.
- [ ] Changing `visual_hold_delay_ms` while a hold timer is already armed resolves to one defined outcome, stated in the ticket: the in-flight timer keeps its original delay, or it is re-armed. Not left to whichever the code happens to do.
- [ ] The arming decision lives in a pure, Win32-free function that tests call directly.
- [ ] `check_switcher_deadline` is either called from production or removed; no `#[allow(dead_code)]` gate remains that a reader could mistake for the live one.
- [ ] The new disabled-path test was **observed red** against the unfixed code, and that is recorded in the ticket or commit message.
- [ ] Automated tests cover both the disabled path and the configured-delay path.
