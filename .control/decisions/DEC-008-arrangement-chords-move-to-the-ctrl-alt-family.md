---
type: decision
id: DEC-008
status: applied
touches:
  - .what/_prd/wira-desk/prd.md
  - .control/registry/requirements.yaml
  - .what/window-management/04-usecases/UC-2-snap-window-half.md
  - .what/window-management/SRS-window-management.md
  - .what/settings/04-usecases/UC-4-change-shortcut.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/05-model/data-model.md
supersedes: null
superseded_by: null
created: '2026-08-26'
accepted: '2026-08-26'
accepted_by: Product Owner (in session)
applied: '2026-08-26'
---

# DEC-008 — The shipped arrangement chords are the Ctrl+Alt family, and Ctrl+Win+Arrow is returned to Windows

## Decision

Two halves, and they are one act. Doing only the first leaves a known gap in the reserved catalogue
open; doing only the second makes the shipped defaults fail their own validation.

**First: every shipped arrangement default moves from the `Ctrl+Win` family to the `Ctrl+Alt`
family.** `Ctrl+Alt` means *place the active window on this monitor*; adding `Shift` means *the
variant that reaches wider than one window or one screen*.

| Field | Was | Is |
|---|---|---|
| `snapping.snap_half_left` | `ctrl+win+left` | `ctrl+alt+left` |
| `snapping.snap_half_right` | `ctrl+win+right` | `ctrl+alt+right` |
| `snapping.snap_half_top` | — | `ctrl+alt+up` |
| `snapping.snap_half_bottom` | — | `ctrl+alt+down` |
| `snapping.snap_maximize` | `ctrl+win+enter` | `ctrl+alt+enter` |
| `layout.stack_shortcut` | `ctrl+win+down` | `ctrl+alt+shift+down` |
| `layout.move_next_monitor_shortcut` | — | `ctrl+alt+shift+enter` |

The switcher chords do not move. `win+backtick` and `alt+backtick` are the product's identity and
`DEC-003` already carves out the second one by name.

**Second: `Win + Ctrl + Left` and `Win + Ctrl + Right` are added to the reserved catalogue** in
`shared::shortcut::reservation`, as `Reservation::ShellOwned`, owner `"switch between your virtual
desktops"`. From that point a user cannot configure them, and the refusal explains itself the way
every other `ShellOwned` refusal does.

The migration of existing installations is deliberately **nothing**. Every field carries
`#[serde(default)]`, so a `config.toml` written before this decision keeps every value it already
holds and gains only the two new fields. A user who is happy on `Ctrl+Win` stays on `Ctrl+Win`; only
a fresh install, or a user who resets, meets the new family. The one collision this creates is
handled by `DEC-009`, not here.

## Why

The `Ctrl+Win` family was not chosen carelessly — `DEC-003` cites `snap_half_left` being
`ctrl+win+left` rather than `win+left` as evidence that "the defaults already had this instinct" of
staying out of the shell's way. That instinct was right and its execution was wrong, and the reason is
one fact that was never checked: **`Win + Ctrl + Left` and `Win + Ctrl + Right` are Windows' own
chords for switching virtual desktops.** The shipped default takes a shell function, which is exactly
what `DEC-003` forbids, and it takes it silently because the low-level hook sees the chord first and
swallows it.

The reserved catalogue proves the omission is an omission rather than a considered exception: it
already lists `win+ctrl+d` and `win+ctrl+f4` — create and close a virtual desktop — and skips the two
arrow chords that *navigate* between them. Three quarters of one feature is catalogued and the
quarter our own default sits on is not.

This decision is therefore not a change of taste. It closes a `DEC-003` violation that has shipped,
and the family move is what makes closing it possible: while `ctrl+win+left` remains a shipped
default, adding it to the catalogue would make the default fail its own validation at load — the exact
carve-out already written into `reservation()` for `ctrl+win+enter`, which the source calls out by
name. Carve-outs are affordable one at a time. Three would turn the catalogue into a list of
exceptions.

`Ctrl+Alt` was verified to be free of the two things that could disqualify it. It appears nowhere in
the reserved catalogue, and it is not a shell chord: the escape hatches the product refuses to take
(`Alt+Tab`, `Alt+F4`, `Alt+Shift+Tab`) are all guarded on `alt && !ctrl`, so adding `Ctrl` leaves
them untouched by construction rather than by luck.

The owner has also been running this exact family in their own `config.toml` for the whole life of the
product, which is not proof but is the only field evidence available.

## Cost

**`Ctrl + Alt + Arrow` is claimed by graphics drivers.** Intel, AMD, and some Nvidia control panels
bind screen rotation to those chords by default on a large share of Windows machines. The reserved
catalogue cannot help: it is a catalogue of chords *Windows* owns, and `DEC-002` forbids probing to
discover what any other application holds. In practice Wira Desk's low-level hook usually wins
because it sees the chord before the driver's own hook, but the ordering between two low-level hooks
is not guaranteed by anything. This is a real cost paid by a real population of users, and the only
mitigations are documentation and the `DEC-005` key check — neither of which is code that fixes it.
It is accepted because the alternative is worse: `Ctrl+Win+Arrow` breaks a Windows feature for
*everyone*, deterministically, while this breaks a driver feature for *some*, probabilistically, and
the driver feature is remappable in the driver's own control panel where the Windows one is not.

**Four frozen tests change.** The `ctrl+win+*` defaults are pinned in `shared/src/config.rs`
(`frozen_snapping_defaults`), `daemon/src/arrangement/mod.rs`
(`arrangement_shortcut_defaults_are_frozen`), `daemon/src/hook.rs` as fallback literals in
`load_shortcuts`, and `settings/src/persistence.rs`. They were pinned so the value could not drift
unnoticed, and they are doing their job right now by making this change loud. Changing them is
correct here and it spends the freeze.

**The product's documented chords and its users' chords diverge.** Because migration is deliberately
nothing, every existing install keeps `Ctrl+Win` while every document, tutorial, and README says
`Ctrl+Alt`. A support answer now has to start by asking which the user has. The alternative — rewriting
users' config files on upgrade — is worse, and this product does not do that.

**One argument in `DEC-006` loses a leg.** `DEC-006` refused passing an unhandled chord back to
Windows partly because "`Ctrl+Win+Left` … is Windows' own previous-virtual-desktop chord and
`reservation()` deliberately does not reserve `Ctrl+Win+Arrow`". After this decision that specific
premise is false — the chord is reserved and is no longer a default. `DEC-006` is `applied` and MUST
NOT be edited; its refusal still stands on its other and stronger leg, that passthrough would move a
window the user cannot see. Recorded here so the next reader of `DEC-006` is not misled by a premise
that has since moved.

## Alternatives

**Keep `Ctrl+Win` and add `win+ctrl+left/right` to the catalogue anyway, with a third carve-out.**
Refused. The carve-out for `ctrl+win+enter` is already documented as an uncomfortable exception in the
source; a second and third turn `reservation()` into a table whose entries mean "reserved unless we
wanted it", which is the erosion `DEC-003` exists to prevent.

**Keep `Ctrl+Win`, and leave the catalogue gap open.** Refused. It is a shipped violation of an
`applied` decision. Knowing about it and declining to close it is the failure mode this repository's
"verify claims against artifacts" rule was written for.

**Move to `Ctrl + Win + Alt + Arrow`, free of both collisions.** Refused on ergonomics. Four
modifiers on a chord meant for repeated, one-handed use during ordinary work is not a shortcut, and a
default nobody can press is a default nobody keeps.

**Move to `Ctrl + Shift + Arrow`.** Refused: it is text selection by word in essentially every editor
and terminal on the platform, which is a far more frequently used function than screen rotation.

**Migrate existing config files to the new family on upgrade.** Refused. Silently rewriting a file
the user owns, to change chords they have learned, is not a migration this product is entitled to
perform. `shared/src/migrate.rs` exists and copies rather than moves for the same instinct.

## Reversal trigger

Revisit the family if `Ctrl + Alt + Arrow` losing to a graphics driver is reported more than once by
different users on different hardware. At that point the probabilistic cost has become the common
case, and a family with no third-party claimant is worth four modifiers.

## Trace

Came from an owner request on 2026-08-26 to add vertical snapping and a monitor-move command, in
which the chords named were `Ctrl+Alt` rather than the shipped `Ctrl+Win` — the discrepancy is what
surfaced the catalogue gap. Every claim above was read from source rather than recalled:
`shared::shortcut::reservation` for what is and is not catalogued, the four frozen tests for what is
pinned, and `%APPDATA%\WiraDesk\config.toml` for the owner's own values.

Enforces `DEC-003` rather than contradicting it — it closes a case that decision's rule already
covered and its catalogue missed. Depends on `DEC-009` for the one collision the new defaults create
against an existing config. Reports, and does not edit, the premise it invalidates in `DEC-006`.
