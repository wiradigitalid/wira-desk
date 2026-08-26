# Coding brief — DEC-003, DEC-004, DEC-005 (and DEF-3)

Scratch. Not authority: where this disagrees with the corpus, the corpus wins.

Gate: **G5 release** (`wdi-build`). All three decisions are `applied`, so they are frozen — an
implementation that needs one changed opens a new `DEC-`, it does not edit these.

Binding documents: `.control/decisions/DEC-003…`, `DEC-004…`, `DEC-005…`;
`.what/settings/02-rules/rules-settings.md` (LBR-ST-10..12);
`.what/settings/04-usecases/UC-4-change-shortcut.md`; `.what/settings/05-scenarios/SCN-01-…`;
`.how/settings/SDD-settings.md`; `.how/window-management/SDD-window-management.md`;
`.how/_platform/ARCHITECTURE-SPINE.md` (its cross-actor channel decision).

## Order matters — DEF-3 is first, and nothing else works before it

**1. Fix the lease contract (`DEF-3`).** Three places disagree about `WM_APP_CAPTURE_LEASE`'s
`lParam`; the lease has never armed once.

| File | Now | Must become |
|---|---|---|
| `crates/shared/src/constants.rs:141,145` | doc says `lParam` = Settings window HWND | `wParam` = lease level `0` none / `1` observe / `2` record; `lParam` = Settings **process id** |
| `crates/settings/src/persistence.rs:176` | sends `std::process::id()` | unchanged in value; `arm: bool` becomes the lease level |
| `crates/daemon/src/tray.rs:399-406` | casts `lParam` to `HWND`, calls `GetWindowThreadProcessId` | forwards the process id unchanged; the conversion is deleted |
| `crates/daemon/src/hook.rs:883` | `capture_lease_pid = lParam as u32` if `wParam != 0` | stores level **and** pid |

Fixing only `persistence.rs` is not a fix: the contract in `constants.rs` would still sanction the
wrong shape, and the next reader restores the bug.

**2. Give the lease decision a seam.** `hook.rs:274` calls `rt.identity.foreground_pid()` directly,
which is `GetForegroundWindow` (`crates/daemon/src/context/vm_bypass.rs:119`). A test harness with no
foreground window gets `0` while the guard demands non-zero, so **the branch is unreachable from a
test by construction** — that is why a dead feature got this far. `DEF-1` already solved this shape
one branch lower: `handle_key_event` splits into a thin wrapper plus `handle_key_event_with_bypass`
taking `FnOnce(&mut HookRuntime) -> bool`. Do the same for the lease. This is a prerequisite, not a
follow-up.

**3. Move the lease comparison above `match_shortcut`.** Today it sits after the early return at
`hook.rs:256-270`, so it is reached only by a chord that is *already* one of the six configured — and
`Win+1` never reaches it. Recording an unconfigured chord requires the comparison to run first.

## The lease is three decisions, not one switch

| Lease | Armed while | report | suppress own action | swallow from Windows |
|---|---|---|---|---|
| none | — | no | no | a matched chord only, as today |
| observe | Shortcuts pane visible **and** Settings foreground | yes | yes | no |
| record | a field listening **and** Settings foreground | yes | yes | yes |

Both fire only on a **non-modifier key-down carrying at least one modifier**. Neither reports a
modifier-only press or any key release. These bounds are load-bearing, not tuning.

Four bounds that are not optional: one owning place per lease (derive the level from
`(pane, capture)` and post only on change — today four `signal_capture_lease` calls sit in
`main.rs:425,428,622,677` and `app.rs::set_pane:387` disarms none of them); fail closed when Settings
is not foreground; reap a dead holder on the existing heartbeat, never in the callback; never swallow
a chord Windows keeps regardless.

## Reporting replaces guessing

The daemon posts the raw virtual-key code plus modifiers. Three heuristics in `crates/settings/src/main.rs`
then come out: the `GetAsyncKeyState` meta-flag patch (`is_win_key_down`, ~line 68), the
`GetAsyncKeyState(0xC0)` backtick rescue (~line 641), and the text-to-key-name ladder. The
text-derived path stays as a **visibly marked** fallback for when no daemon is running — it is not
deleted, and it is not used silently.

A reported vk with no canonical name (`shared::shortcut::name_from_vk` returns `None`, e.g.
`Win+Semicolon`) MUST be an explicit refusal with its own message. Silence here reproduces the
unresponsive recorder in a narrower place.

## Reserved-chord catalogue

Data in `crates/shared/src/shortcut.rs`, each entry naming the Windows function and its kind:
*Windows keeps it regardless* (no alternative offered — none would be true) vs *Wira Desk could take
it and will not* (offer a free chord).

`Alt+Space`, `Alt+Esc`, and **`Alt+Backtick` stay allowed on purpose** — `Alt+Backtick` is this
product's own `switcher.fallback_shortcut` default, so adding it would make the shipped default fail
its own validation. `Alt+Tab` is suppressible; its place in the catalogue is policy, not a technical
limit.

Two enforcement paths in the daemon, and both need a branch, because a reserved chord *parses* fine:
`crates/daemon/src/config.rs:113` (`validate`, atomic reject) and `crates/daemon/src/hook.rs:642-720`
(`load_shortcuts`, per-field warn + fallback). The current denylist lives in
`crates/settings/src/persistence.rs:35` where the daemon can never see it.

Suggestion ladder needs two filters, each proven by a counter-example: `Win+D` → naive `Ctrl+Win+D`
is itself a Windows hotkey (new virtual desktop); `Win+Left` → naive `Ctrl+Win+Left` is already the
user's SnapLeft.

## Key check

Two correlated signals or no verdict. Four rows, and the second is the whole point:

| Hook saw | Window saw | Verdict | Usable |
|---|---|---|---|
| yes | yes | nothing intercepts it | yes |
| yes | no | another app claimed it, our hook is earlier | **yes** |
| no | no | an earlier low-level hook or Windows consumed it | no |
| no | yes | the daemon is not running, or its hook is dead | not for now |

Never a badge predicting availability for a chord nobody pressed. Never a perpetual pulse — it would
prove the renderer is alive, not that keys arrive, and it is the only element that would burn CPU
while idle. One beat per observed keystroke.

`crates/settings/ui/main_window.slint:113-127` wires `key-pressed` only. **There is no key-release
signal at all**, so modifier pills cannot be released without adding one.

## Done means

- `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace` green.
- The lease guard **seen failing**: break it, watch it go red with `--no-fail-fast`, restore.
- Every new `unsafe` block carries a `SAFETY:` comment stating the precondition relied on.
- `scripts/verify-public-export.ps1` clean — fix the source, never widen a pattern.
- `3p.md` updated (code tracker, not `docs/3p.md`).
- `DEF-3` moved to `fixed` **only** with the failing-test evidence; its `decision: DEC-004` gate is
  already cleared.
