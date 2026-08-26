---
artifact: .control/decisions/DEC-003-windows-shell-chords-are-refused-not-overridden.md
skill: orchestrator (no wrapper invoked)
date: 2026-08-26
---

# Memlog — reserved Windows hotkey policy, DEC-003 and DEC-004 drafted

Owner reported a user-testing finding on 2026-08-26: pressing `Win+1` or `Win+E` while a shortcut
field was listening let Explorer act and steal focus, with no message anywhere. Owner asked for the
architecture and UX policy to be decided and the documents written; coding is run separately.

`wdi-decision` was named to the owner and **not invoked** — the project rule forbids invoking a skill
automatically. The two `DEC-` files, the registry rows, and this memlog were written directly. If the
owner wants the wrapper's own numbering and verification pass, the files can be re-landed through it.

## What was verified before deciding

Read against the code rather than the corpus, because the corpus turned out to describe the intent
and not the ordering:

| Claim checked | Finding |
|---|---|
| The capture lease is where reserved-chord recording belongs | **False.** The lease is checked *after* the `match_shortcut` early return, so it engages only for a chord already configured. `Win+1` never reaches it |
| The lease can be extended to capture a shell chord | **Not as it stands.** An armed lease returns passthrough *by design*, so the toolkit can see the chord — and passthrough is exactly what lets Explorer act. The disposition has to invert |
| `Alt+Backtick` is a reserved OS chord | **False.** Windows does not claim it. It is the product's own `switcher.fallback_shortcut` default; only the recorder's text-derived key name fails on it. Denylisting it would fail the shipped default's own validation |
| `Alt+Tab` is un-overridable | **False.** A low-level hook can suppress it. Its place in `is_reserved_system_shortcut` is a policy choice, not a technical limit — and the denylist currently gives no way to tell those two apart |
| The denylist is enforced where config enters | **False.** It lives in `settings`, so the daemon never consults it. A hand-edited `config.toml` holding `win+l` is accepted today |
| The daemon has one config path to guard | **False.** Two, with deliberately different semantics: `config.rs::validate` rejects a reload atomically, `hook.rs::load_shortcuts` falls back per field so startup can come up. A reserved chord *parses*, so both need a new branch |

## Why the recommendation changed mid-consultation

An earlier pass in the same session recommended a soft warning with an explicit override for
shell-owned chords. The owner's framing — a daily tray utility — supplied the argument that
overturned it: an override lives only as long as the daemon, so one chord does two different things
depending on whether a background process is alive, and the user can see neither state at the moment
they press it. That asymmetry is now the load-bearing reason in `DEC-003`.

## Status and what is deliberately not written

Both decisions are `status: draft` with `touches: []`. Two rules forced this and neither is
negotiable: an agent MUST NOT accept its own `DEC-`, and `touches` is filled when a decision is
*applied*, from what actually changed. A draft changes nothing by design.

So the SRS, SDD, `rules-settings.md`, `UC-4`, and `SCN-01` were **left untouched**. Carrying a draft
into the documents it governs would forge the evidence that `applied` exists to record. Those edits
belong to the apply step, after the owner ratifies.

`OQ-5` and `OQ-9` in `assumptions.md` both anticipated this — `OQ-9` names `Win+Z` specifically as
the chord nothing tracked. Both stay open and close when `DEC-003` reaches `applied`. `OQ-16` was
added for the one thing that cannot be decided from a desk: which chords a low-level hook cannot
suppress at all, and the assumption that every unmeasured candidate defaults to the un-suppressible
kind.

## Correction, same day — DEC-004 widened and DEC-005 added

Owner asked for a live key-check card in the Shortcuts pane, which forced a correction to `DEC-004`
before it was ever ratified. Both files were still `draft`, so they were corrected in place; the
guide requires the correction be recorded here.

**What was wrong in `DEC-004` as first written.** It carried reporting and swallowing as one
capability on one lease. That reads as simpler until a second reader needs the report while the pane
is merely open — and then one lease means the keyboard is swallowed for as long as a pane the user
may leave open stays open. That outcome is `DEC-004`'s own third rejected alternative, and merging
the two capabilities is what would have made it look like a small extension instead. The lease is now
two: **observe** reports and passes through, **record** reports and swallows.

`DEC-004`'s H1 title changed to match. The filename was deliberately **not** renamed — the guide's
stated reason for constraining filenames is that a rename breaks every link, and the slug is still
accurate as far as it goes. The registry `title` was updated instead, which is what
`.control/generated/decisions.md` renders.

**Why `DEC-005` is a decision and not a UI task.** The card's premise — no activity here means another
app is hijacking it — is false for one of the two interception mechanisms. An application holding a
chord via `RegisterHotKey` takes it out of the window's queue but does *not* beat a low-level hook, so
Wira Desk still gets it and the chord works. A card reading window events alone would tell the user to
abandon a shortcut that functions perfectly: the same wrong-in-both-directions failure `DEC-002`
refused a probe for. Two signals correlated, or no verdict at all.

**Costs newly written down rather than discovered later:** pass-through under the observe lease means
checking a chord another app owns will fire that app's action; the check keeps no history, so an
overlay that only takes a chord while a game runs reads as healthy; and the second correlation row
tells a user their chord works without saying that the other application's shortcut just stopped.

Three `DEC-` now stand as `draft` with `touches: []`, and `OQ-5`, `OQ-9`, `OQ-16` stay open. Nothing
in `.what/` or `.how/` was touched, for the same reason as the first pass.

## Second correction, same day — the lease had never armed

A follow-up consultation asked which of three arming conditions the capture lease should use. The
question turned out to rest on a premise nobody had checked: that the lease worked at all.

It does not. `WM_APP_CAPTURE_LEASE`'s `lParam` is documented in `crates/shared/src/constants.rs` as a
window handle, sent by `crates/settings/src/persistence.rs` as a process id, and converted back by
`crates/daemon/src/tray.rs` with `GetWindowThreadProcessId` — which fails on a value that is not a
handle and leaves its out-parameter at `0`, which the hook's own guard then rejects. Recorded as
`DEF-3`, `root_cause: architecture`, left `open` because it closes on `DEC-004` being accepted rather
than on a patch landing.

Three things are worth keeping from how it hid, because each is a habit rather than a fact about this
bug:

- **The SSOT was on the wrong side.** `constants.rs` is the contract, and it said handle. So the
  daemon was correct against the contract and Settings was the violator — meaning a review that read
  either file against the contract would have confirmed that file. A fix touching only the sender
  leaves the contract sanctioning the bug for the next reader.
- **The branch was unreachable from a test by construction**, not merely untested: it asks the desktop
  for the foreground process id, a harness gets `0`, and the guard demands non-zero. This is `DEF-1`
  one branch earlier — `DEF-1` introduced a seam for exactly this, and the lease check sits above it.
- **The debug trace read as a disarm.** `HOOK_LEASE: arm=true pid=0` blames the sender. A diagnostic
  that prints a derived value without printing what it was derived from points at the wrong file.

The consultation's own three options also turned out to be one axis where the hook has three: report,
suppress Wira Desk's own action, and swallow from Windows. The existing lease is `no / yes / no`. Once
separated, the observed symptom stops being an architecture question and becomes `DEF-3`, and the only
genuine decision left is whether observe suppresses — it must, or the key check switches windows out
from under the pane being read. That is what `DEC-004`'s table now carries.

Also written down rather than left to the implementation: process-id addressing, because the
alternative (window handle) is the shape that produced `DEF-3`; and `OQ-17`, because process-id reuse
is a real hole that disarm-plus-heartbeat narrows without closing.

None of the lease code is in `HEAD`. It is uncommitted work in the working tree, so `DEF-3` is a
finding about unfinished work, not a regression against a release.
