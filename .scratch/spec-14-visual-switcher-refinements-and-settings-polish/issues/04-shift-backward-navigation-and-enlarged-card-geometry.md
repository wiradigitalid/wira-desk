# 04: Visual switcher Shift-held backward cycling

**What to build:**
Shift-held backward cycling, unblocked by `DEC-026` clause 2 and its §B-1…B-6.

The card-geometry half that used to live here as part 2 is now **`SPEC-14-06`**. The owner's Alt+Tab
screenshot turned it from "raise two constants and add a height bound" into a layout-engine
replacement that also rewrites spatial navigation; it shares no code and no risk with the Shift work,
and bundling the two in one registry row is the mistake that split ticket 02 earlier.

---

## Shift-held backward cycling — `DEC-026` accepted

**Read `DEC-026` §B-1 through §B-6 before writing a line.** The decision's own clause 2 is one
sentence; those six sub-clauses are what make it buildable without breaking something silently. Every
one was derived from a live code site, and five of the six have a failure mode that no existing test
catches.

**Half of this already works.** With the overlay open, re-pressing the main key with Shift held
already routes to `Command::SwitcherPrev` (`hook.rs:519-526`), and `Left`/`Right` arrows already
navigate (`:517-518`). None of that changes.

### 1a. Two-pass matching (`DEC-026` §B-1)

`match_shortcut` (`hook.rs:1023`) compares the live `ModifierState` to the configured `Shortcut` by
exact equality on all four modifier bits, so `Win+Shift+Backtick` against a `win+backtick` cycle
binding matches nothing and falls through to the foreground app.

Do **not** make Shift a blanket "don't care". `Cycle` is first in `Chords::in_declared_order`
(`hook.rs:822`) and the first match wins, so a relaxed first pass shadows every chord differing from
the cycle chord only by Shift — concretely, bind cycle to `ctrl+alt+s` and the shipped default stack
chord `ctrl+alt+shift+s` (`shared/src/config.rs:435`) becomes unreachable. The duplicate check
(`daemon/src/config.rs:196-206`) compares parsed `Shortcut` values, which differ, so it raises
nothing and the collision is silent.

Resolve in two passes instead:

1. **Exact pass** over all sixteen slots, exactly as today.
2. **Shift-relaxed pass**, reached only when pass 1 returns `None`, evaluated only against the two
   `Cycle` slots (`primary`, `fallback`) with `shift` cleared. A match resolves to the backward cycle
   and to nothing else.

BR-6 / DEC-009 holds verbatim: a chord that reaches a configured action never reaches pass 2.

### 1b. Direction travels in the ring, not in the modifier atomic (`DEC-026` §B-4)

There is no backward blind cycle today: no `Command::CyclePrev`, both cycle slots map to
`Command::Cycle`, and `cycle_order` (`cycling/mod.rs:228`) produces one direction only.

The obvious shortcut — let the Worker read direction from `SWITCHER_LAST_CYCLE_MODS`, the `AtomicU8`
the Hook writes at `hook.rs:626` and the Worker reads at `worker.rs:102` — **is a race, and this
ticket is what makes it visible.** The Hook overwrites that atomic on every cycle while commands queue
independently in the ring. A second chord press arriving before the Worker drains the first overwrites
the direction of a command already in flight: tap the chord, then tap it with Shift, and the **first**
tap cycles backward. Today the same race only mis-arms a timer, which is why nothing has caught it.

The owner confirmed on 2026-09-13 that a backward **blind** cycle is wanted, not only backward entry
into the overlay. So this lands at full scope.

Add `Command::CyclePrev` as its own byte, decided by the Hook at match time and immutable
thereafter. **Two existing sites compare against `Command::Cycle` by value and must be widened with
it. Neither is reachable from the behavioural criteria alone, and both fail silently:**

- `hook.rs:625` — `if cmd == Command::Cycle.as_u8()` guards `set_last_cycle_mods` *and the entire
  switcher-arming block*. Left untouched, a Shift-held chord cycles backward correctly and then never
  arms — the overlay never opens, and 1d below is unreachable while every backward-cycle assertion
  stays green.
- `worker.rs:99` — the drain `match` arm that calls `execute_cycle`. Left untouched, `CyclePrev`
  matches no arm and the command is dropped without a trace.

Throttle `CyclePrev` exactly as `Command::Cycle` is — do **not** add it to `is_exempt_from_throttle`
(`shared/src/commands.rs:105`); that exemption is for held switcher repeat ticks, and a cycle is not
one. `cycle_order` grows a direction parameter; the backward order is the same rotation without the
final `reverse()` at `cycling/mod.rs:238`.

### 1c. Shift never joins the chord-modifier sets (`DEC-026` §B-5)

Two sites capture `rt.mods` wholesale and would quietly make Shift a chord modifier:

- `hook.rs:628` — `rt.switcher_mods = rt.mods`. Commit fires on
  `!rt.mods.has_any_of(&rt.switcher_mods)` (`hook.rs:446`), true only when **every** shared modifier
  is up. With Shift in the set, releasing Alt while still holding Shift leaves the overlay open with
  no chord modifier down, and the watchdog (`worker.rs:215`) will not rescue it either — it commits
  only when *no* modifier at all is down, and Shift is one.
- `worker.rs:497` — `SWITCHER_CHORD_MODS`, read by `are_chord_modifiers_down` (`worker.rs:162`),
  which requires **every** recorded modifier to still be physically down when the hold timer fires.
  With Shift recorded, a user who releases Shift a moment early — the natural motion — silently gets
  no overlay at all.

Clear `shift` from the state stored in both. Commit and hold-gate semantics then stay byte-for-byte
what they are today.

### 1d. Backward entry opens on the last card (`DEC-026` §B-6)

`open_visual_switcher` calls `SWITCHER.open(origin, work_area, eligible_ordered, 0)`
(`worker.rs:295`) — the selected index is a hardcoded `0`. A backward entry opens at `len - 1`.

Because `card_order_for_candidates` (`switcher/mod.rs:168`) *is* `cycle_order`, index `len - 1` is
exactly the window a backward blind cycle would have activated — overlay and blind path agree by
construction, not by coincidence. This only holds once 1b lands: without it a Shift-held chord runs a
**forward** blind activation and then opens an overlay claiming to be backward, so the first thing
the user sees is the opposite of what they asked for. **1b and 1d ship together or neither ships.**

### 1e. Validation: the Shift ban, and the derived variant's reservation (`DEC-026` §B-2, §B-3)

- `Shift` is refused as a modifier in the **cycle** chord. `settings::persistence::validate_shortcut`
  takes only the input string and cannot carry a field-specific rule; the rule goes in
  `validate_config` (`persistence.rs:109`), which knows the field name.
- **Do not add it to the daemon's reload validation as a rejection.** `daemon::config::validate`
  refuses a reload **wholesale** — a user who legitimately bound cycle to `ctrl+shift+backtick` before
  this decision would lose every other setting to a rule written after the fact. On reload such a chord
  is accepted and matched exactly, and simply receives no shift-relaxed variant.
- **The derived Shift variant must be reservation-checked.** `shared::shortcut::reservation`
  (`shortcut.rs:172`) is modifier-exact, so the variant becomes reachable without ever being checked.
  This is reachable, not theoretical: `Ctrl+Escape` is **not** in the catalogue and is a legal cycle
  binding today, while `Ctrl+Shift+Escape` **is** catalogued `Reservation::Immutable`, owner
  *"Task Manager"*. Binding cycle to `ctrl+escape` would swallow the user's Task Manager escape hatch.
  Validation must refuse a cycle chord whose Shift variant carries a reservation, naming that
  reservation's owner — on both the Settings path and the daemon reservation walk.

**Blocked by:** None. Touches `open_visual_switcher`'s `SWITCHER.open(...)` call, which ticket 03
also edits — sequence them or expect a conflict there.

**Status:** ready-for-agent

- [ ] `match_shortcut` resolves exact-first; the shift-relaxed pass runs only on a `None` from the exact pass, and only against the two `Cycle` slots.
- [ ] Binding cycle to `ctrl+alt+s` leaves the default `ctrl+alt+shift+s` stack chord reachable — asserted by a test that fails if the relaxed pass runs first or covers non-cycle slots.
- [ ] Pressing the cycle chord with Shift held cycles backward; the window activated is the one a forward cycle would have reached *last*.
- [ ] Direction is carried as `Command::CyclePrev` in the ring, not read from `SWITCHER_LAST_CYCLE_MODS`. A test enqueues `Cycle` then `CyclePrev` before either is drained and asserts each keeps its own direction.
- [ ] `Command::CyclePrev` is **not** in `is_exempt_from_throttle`, and is throttled identically to `Command::Cycle`.
- [ ] `cycle_order`'s backward direction is the forward rotation without the final `reverse()` — asserted against a fixture, both directions, same candidate set.
- [ ] A rapid Shift-held tap (no hold) activates the previous same-app window on the active monitor, and is still monitor-locked — the blind path keeps `SpatialScope::SameMonitor`.
- [ ] `hook.rs:625`'s arming guard covers `CyclePrev`: a Shift-held chord *held* past the delay opens the overlay. Asserted, because a backward cycle that never arms passes every other criterion here.
- [ ] `worker.rs:99`'s drain arm covers `CyclePrev`: the command is executed, not silently dropped.
- [ ] `shift` is cleared from both `rt.switcher_mods` and `SWITCHER_CHORD_MODS`.
- [ ] Releasing the non-Shift chord modifier while Shift is still held **commits** the overlay — the case that hangs it open today, asserted.
- [ ] Releasing Shift during the hold window while the chord modifier stays down still opens the overlay — the case `are_chord_modifiers_down` silently swallows today, asserted.
- [ ] A backward entry opens with `selected_index == len - 1`, and that index names the same window the backward blind cycle activated.
- [ ] Saving a cycle chord containing `shift` is refused in Settings, naming the field.
- [ ] An on-disk config that already binds `shift` in the cycle chord reloads **successfully**, keeps every other setting, matches that chord exactly, and gets no shift-relaxed variant.
- [ ] Binding cycle to `ctrl+escape` is refused because its Shift variant is `Ctrl+Shift+Escape` (Task Manager) — asserted on both the Settings path and the daemon reservation walk, with the owner string in the message.
- [ ] In-overlay Shift reversal (`hook.rs:519-526`) is unchanged.
