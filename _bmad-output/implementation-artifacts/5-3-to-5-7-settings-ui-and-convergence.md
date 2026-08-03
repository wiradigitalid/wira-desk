---
baseline_commit: 9bc578f
workflow_id: story-5-3-to-5-7-settings-ui
covers: [5.3, 5.4, 5.5, 5.6, 5.7]
---

# Stories 5.3–5.7: Settings UI, Capturer, Onboarding, Bridge, Convergence

Status: in-progress

> **Combined artifact.** These five stories share `crates/settings/src/app.rs`
> and `main.rs`; splitting the record would duplicate the same evidence four
> times. Each story's status is called out individually below.

## Status by Story

| Story | State | Blocking gap |
| --- | --- | --- |
| 5.3 Modular Settings shell and safe editing | code complete, model verified | no rendered UI ever observed |
| 5.4 Accessible physical shortcut capturer | code complete, model verified | real key capture never exercised |
| 5.5 Interactive first-run shortcut training | code complete, model verified | the interactive practice exercise and the auto-start consent step are absent, not merely unrendered |
| 5.6 Daemon Settings bridge and first-run orchestration | partially complete | daemon-side launch of `--onboarding` not wired; the daemon also never handles the `WM_APP_RELOAD_CONFIG` it receives (see Review Findings, decision-needed) |
| 5.7 Convergence | **not started** | requires 5.2's acceptance gate, which is not met |

## Dev Notes

### The Model Is Separated From the Rendering On Purpose

`SettingsModel` holds every decision that matters — staged draft, capture state
machine, onboarding progression, save outcome — and `main.rs` is a thin drawing
layer over it. That is what makes 23 of these tests executable without a window.

Had the state lived inside the `App::ui` closure, none of it would be testable
at all, and this epic would rest entirely on manual inspection.

### Staged Editing

The user edits `draft`; `saved` only changes after a successful write. A
rejected save leaves **both** untouched, so an invalid entry can be corrected
instead of losing the rest of the user's edits.
`a_rejected_save_reports_an_error_and_does_not_promote_the_draft` pins it.

### The Capturer Stays Open On Rejection

`accept_capture()` validates before touching the draft. On failure it returns
the error **and remains in `Listening`**, so the user can simply press a
different combination. Closing the capturer on failure would force them to
re-open it to fix a typo.

Holding modifiers alone never commits: `captured_combination()` returns `None`
until a non-modifier key is down and at least one modifier is present.

### Skip Tutorial Is Not a Shortcut Around Configuration

Both "Finish" and "Skip Tutorial" reach `OnboardingStep::Done` and write a valid
configuration. That is what stops onboarding repeating on the next launch
(AC-5.1-006). `skip_reaches_the_same_terminal_state_as_completing` asserts the
two paths converge.

### Onboarding Teaches the Spatial Philosophy

PRD §6 requires WinTick to explain how this differs from Alt+Tab; the
implementation-readiness report flagged it as covered by no story. The Welcome
copy states the contrast explicitly, and a test asserts the word "Alt+Tab"
survives future copy edits — without it, the feature reads as a broken Alt+Tab.

### What Story 5.6 Is Missing

The Settings→daemon direction only posts the signal: `signal_reload()` sends
`WM_APP_RELOAD_CONFIG` after an atomic write, but the daemon does not yet
handle that message — see Review Findings' decision-needed item on
`WM_APP_RELOAD_CONFIG` for the detail. The daemon→Settings direction is
**not** done either: nothing in the daemon launches `wintick-settings.exe
--onboarding` on a missing configuration. `persistence::resolve_launch_intent`
implements the Settings side of that contract, so the remaining work is a call
site in the daemon's startup path.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- **`cargo test -p settings` PASS 48/48 — really executed**, covering staged
  editing, the capture state machine, focus-order determinism, onboarding
  progression, and save feedback.
- Builds clean on egui 0.35 in debug and release.
- **No window has ever been rendered.** Every rendering decision — layout,
  contrast, focus visibility, whether the shortcut button is even reachable by
  `Tab` — is unverified. `cargo build` proves the code type-checks against
  0.35's API, nothing more.
- `focus_order()` is declared and tested but **not consumed by the renderer**.
  The panes happen to draw in that order because they iterate `Pane::ALL` and
  `ShortcutField::ALL`, but nothing enforces the match.
- Story 5.7 is **not started**: AC-5.2-007 makes dependent UI stories ineligible
  until the accessibility gate is demonstrated.

### File List
- `crates/settings/src/app.rs` (new)
- `crates/settings/src/main.rs` (rewritten for eframe 0.35)
- `crates/daemon/src/menu.rs` (touched by commit 428604d for daemon-side onboarding launch)
- `crates/daemon/src/tray.rs` (touched by commit 428604d for daemon-side onboarding launch)
- `crates/shared/src/constants.rs` (touched by commit 428604d for daemon-side onboarding launch)
- `crates/shared/src/lib.rs` (touched by commit 428604d for daemon-side onboarding launch)

### Review Findings

_Adversarial code review of `git diff 9bc578f..HEAD` on `crates/settings/src/{app.rs,main.rs,persistence.rs}`,
`crates/daemon/src/{menu.rs,tray.rs}`, `crates/shared/src/{constants.rs,lib.rs}` — Blind Hunter +
Edge Case Hunter + Acceptance Auditor (against Stories 5.3/5.4/5.5/5.6 ACs in `epics.md`). Story 5.7 out of scope._

**Decision-needed**

- [ ] [Review][Decision] Daemon never handles `WM_APP_RELOAD_CONFIG` — Save's "applied" message is unverified — [5-6, cross 5-3] `signal_reload()` (`crates/settings/src/persistence.rs:132-150`) posts `WM_APP_RELOAD_CONFIG` to the daemon's hidden window, but grepping the entire `crates/daemon/src/` tree finds no handler for that message anywhere (not in `tray.rs`'s `wndproc_impl`, not `worker.rs`, not `hook.rs` — only a comment reference at `crates/daemon/src/tray.rs:57`). It falls through to `DefWindowProcW`. Yet `crates/settings/src/main.rs` (`actions()`, `SaveFeedback::Saved` branch) unconditionally shows "Settings saved and applied." whenever `reload_signalled` is true — which only means a window was *found*, not that the config was actually re-read. Decision needed: is daemon-side reload handling in scope for 5.6 (its own AC says "the daemon receives `WM_APP_RELOAD_CONFIG`... reads and validates the completed file exactly in response to that message"), or intentionally deferred to Story 5.7's declared ownership of "daemon Hook/Worker wiring"? Either way, the spec's "Settings→daemon direction is done" claim (Dev Notes, "What Story 5.6 Is Missing") needs correcting — it conflates "message posted" with "message handled."
- [ ] [Review][Decision] No cross-field shortcut collision validation — [5-3, cross 5-4] `validate_config` (`crates/settings/src/persistence.rs:84-96`) validates each of the 6 shortcut fields independently for parseability only; `accept_capture` (`crates/settings/src/app.rs:224-231`) likewise never checks a newly captured value against the other 5 fields. A user can save Switcher and SnapLeft both bound to `ctrl+win+left` with no warning. Decision needed: hard-block duplicates at capture/Save time, warn-but-allow, or leave unconstrained by design?
- [ ] [Review][Decision] Daemon's global keyboard hook can swallow the exact combination Settings is trying to capture — [5-4] `crates/daemon/src/hook.rs` `handle_key_event`/`match_shortcut` (unmodified in this diff) swallows any keystroke matching the *currently configured* Switcher `primary`/`fallback` shortcut system-wide, regardless of foreground window — including the Settings window itself while `Listening`. Concretely: trying to set field A to the value currently held by field B (e.g. swapping Switcher ↔ Fallback) is consumed by the hook before `captured_combination()` (`crates/settings/src/main.rs:251-277`) ever sees it — the button silently stays on "Listening…" forever, no error. Decision needed: does the daemon need to suspend/relax hook matching while Settings is capturing (new control message), or is this an accepted limitation to document?
- [ ] [Review][Decision] Story 5.5's interactive practice exercise is not implemented — [5-5] The AC requires "an interactive dummy-window exercise for Win+Backtick," "simulated foreground state advances visibly between dummy windows," and "handles practice input only within the onboarding process." `OnboardingStep::TrySwitching` (`crates/settings/src/app.rs:148-160` heading/body) is static instructional text only, paired with a manual "Next" button (`crates/settings/src/main.rs:80-82`) — there is no dummy window, no simulated switching, and no detection of the user pressing anything. This is unimplemented functionality, not a rendering gap; the spec's "code complete, model verified" status for 5.5 substantially overstates what exists.
- [ ] [Review][Decision] Story 5.5's auto-start consent step is missing from onboarding — [5-5] AC: "onboarding offers Start with Windows... user accepts or declines... `general.auto_start` reflects the explicit choice." None of `OnboardingStep::{Welcome,TrySwitching,Done}` (`crates/settings/src/app.rs:126-160`) present an auto-start toggle — it only exists in the normal Settings General pane (`crates/settings/src/main.rs:134-139`), which onboarding never shows. `finish_onboarding()` (`crates/settings/src/main.rs:92-95`) persists whatever `general.auto_start` already defaults to (`false`) without ever asking. Decision needed on where/how to add this step.
- [ ] [Review][Decision] "Skip Tutorial" requires a second "Finish" click to actually persist — [5-5, cross 5-6] `crates/settings/src/main.rs:83-85` (`Skip Tutorial` click) only calls `skip_onboarding()`, fast-forwarding to `OnboardingStep::Done` — it does not persist. Persistence only happens if the user then also clicks "Finish" on the resulting screen (`finish_onboarding()`, `crates/settings/src/main.rs:76-78,92-95`). Closing the window instead (plausible, since "Skip Tutorial" reads as a completing action) leaves no `config.toml`, so the next daemon start/Settings launch reopens onboarding — contradicting the AC "the next daemon start does not reopen onboarding solely because it was skipped." Decision needed: should Skip Tutorial persist+close in one step?
- [ ] [Review][Decision] Pane grouping doesn't match the AC's feature-oriented wording — [5-3] AC calls for "feature-oriented groups for Core Switcher, Window Snapping, Stack Layout, and applicable advanced settings," but the actual panes (`crates/settings/src/app.rs:17-23`) are `General`/`Shortcuts`/`Layout`/`About`, with all six Switcher/Snapping/Stack shortcut fields flattened into one undifferentiated "Shortcuts" tab (`crates/settings/src/main.rs:141-188`) instead of grouped under their owning feature area. Decision needed: accepted simplification, or reorganize before acceptance?

**Patch**

- [x] [Review][Patch] `validate_shortcut` never requires a modifier — bare keys like `"a"` validate as a legal shortcut [crates/settings/src/persistence.rs:33-73]
- [x] [Review][Patch] Physical-capture validation errors are silently discarded — no accessible error shown per AC [crates/settings/src/main.rs:184]
- [x] [Review][Patch] `focus_order()` is declared/tested but not actually consumed by the renderer; the only cross-check is a debug-only cosmetic label, so release builds have no structural guarantee of AC-5.2-003 keyboard order [crates/settings/src/app.rs:283-327]
- [x] [Review][Patch] Onboarding Welcome copy omits the "same-monitor and same-virtual-desktop preservation" explanation the AC requires [crates/settings/src/app.rs:148-160]
- [x] [Review][Patch] Switching Settings pane away from Shortcuts does not cancel an in-progress capture; `Listening` state silently persists and resumes when the user returns [crates/settings/src/main.rs:104-109]
- [x] [Review][Patch] Spec doc drift beyond what the review briefing already flagged: 5.5's "tutorial never rendered" understates that the interactive exercise and auto-start consent are simply unimplemented; 5.6's "Settings→daemon direction is done" ignores that the daemon never processes the reload message; "File List" still only names `app.rs`/`main.rs` [_bmad-output/implementation-artifacts/5-3-to-5-7-settings-ui-and-convergence.md — Status by Story table, Dev Notes, File List]

**Defer**

- [x] [Review][Defer] Enter/Space may be uncapturable as shortcut main keys — egui's default keyboard-activation of a focused `Button` on Enter/Space can register as `response.clicked()` (toggling capture off) before `captured_combination()` sees the key as "down" in the same frame; both are explicitly listed as supported in `key_name()` [crates/settings/src/main.rs:141-198,251-277,284-333] — deferred, pre-existing egui widget-activation interaction; needs manual runtime verification (the UI has never been rendered per the Dev Notes) before it can be confirmed as a real bug or patched, story 5-4.
- [x] [Review][Defer] Tab may be uncapturable as a shortcut main key — egui's default focus-navigation typically consumes Tab before app code observes it via `keys_down`, despite `key_name()` listing `Key::Tab => "tab"` as supported [crates/settings/src/main.rs:284-333] — deferred, pre-existing egui behavior; same runtime-verification caveat as above, story 5-4.
- [x] [Review][Defer] "No configuration exists → normal Settings mode displays shared defaults" has no reachable path through the actual binary — any launch with a missing config file redirects to onboarding via `launch_intent`'s fallback regardless of the `--onboarding` flag [crates/settings/src/persistence.rs:170-186] — deferred, pre-existing/likely-intentional interaction with Story 5.6's first-run design; worth confirming it isn't accidental, story 5-3 cross 5-6.

_Dismissed as noise (2): `is_modifier_key` always returning `false` regardless of its parameter (`crates/settings/src/main.rs:278-282`) — intentional per its own doc comment, matches egui's actual modifier/key-event model, not a bug. `ShortcutError::Unrepresentable` appears unreachable in practice since `vk_from_name`/`name_from_vk` are exact inverses for every value the parser can produce — defensive dead code, not a bug._
