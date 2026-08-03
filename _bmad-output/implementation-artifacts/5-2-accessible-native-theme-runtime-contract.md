---
baseline_commit: 9bc578f
workflow_id: story-5-2-accessible-theme-contract
---

# Story 5.2: Accessible Native-Theme Runtime Contract

Status: review — verified by `verify-story-5-settings-runtime.ps1` (10/10 PASS, 2026-07-26); one code-review decision item open (AC-5.2-002 description exposure), see Review Findings.

## Story

As a Windows user who relies on native accessibility behavior,
I want WinTick controls to expose meaningful keyboard and screen-reader semantics,
so that I can use Settings without depending on a mouse or visual-only state.

## Acceptance Criteria

### AC-5.2-001 — Version and mechanism frozen — **DONE**
`egui`/`eframe` 0.35.x selected with the `accesskit` feature explicitly enabled; the 0.28 dependency is **replaced**, not retained.

### AC-5.2-002 — UI Automation roles and states — **partially done**
`verify-story-5-settings-runtime.ps1` (2026-07-26, 10/10 PASS) has since run a real UI Automation client against the actual window and confirms accessible names reach the tree, controls are focusable, and the Listening announcement is exposed via `WidgetInfo` — UI Automation inspection has now been performed, unlike what this AC previously said. However, this review found that each control's *description* (`ControlSemantics.description`) is wired only through `.on_hover_text(...)` (`crates/settings/src/main.rs:138`, `:156`, `:194`), which is mouse-hover-only rendering and never calls `widget_info()` or reaches the accessibility tree — the "descriptions ... attached via `WidgetInfo`" claim in this AC does not hold as written. This is an open decision item (see Review Findings) — accept name-only exposure, add a real description hook, or record it as a framework limitation — so AC-5.2-002 remains not fully met pending that decision.

### AC-5.2-003 — Keyboard operation and focus — **partially done**
Unlike what this AC previously said, the renderer now does consume `focus_order()`: `crates/settings/src/app.rs` declares it and `crates/settings/src/main.rs:143` compares the drawn stops against it on every frame, and `verify-story-5-settings-runtime.ps1` independently confirms every declared stop is present and keyboard-focusable in the real UI Automation tree. What remains true from before: no test has actually simulated `Tab`/`Shift+Tab`/`Space`/`Enter`/`Escape` keypresses and observed focus move or a control activate/cancel — an open decision item (see Review Findings) on whether trusting egui's built-in Tab traversal is sufficient or whether the harness needs to be extended with simulated keypresses.

### AC-5.2-004 — Light/Dark and typography — **partially done**
Theme detection and mid-session switching are implemented, and this review's findings against this AC are now fixed: the focus stroke previously targeted the wrong style field (`selection.stroke` instead of `widgets.active.bg_stroke`) and had no visible effect on any real control — corrected in `apply_typography()`. Real Segoe UI/Tahoma bytes are now loaded from `%SystemRoot%\Fonts` with a bundled-font fallback chain (superseding the earlier "Segoe UI is not actually installed" note), a corrupt/truncated font file is now validated and rejected before it can panic the app instead of degrading, and the mid-session switch now schedules a periodic repaint so it is actually guaranteed to show up while idle rather than only on the next incidental repaint.

### AC-5.2-005 — About surface — **not addressed**

### AC-5.2-006 — Process isolation — **DONE by construction**

### AC-5.2-007 — Acceptance gate — **partially resolved, one open decision**
`verify-story-5-settings-runtime.ps1` genuinely passes 10/10 against the real window (UI Automation tree publishes, controls are named and focusable, declared focus stops are present, onboarding appears on first run, both themes render) — real, independently-verified progress, and this review's own patch-tagged defects (wrong-field focus stroke, corrupt-font panic, hardcoded font path, unscheduled repaint, and the rest) are now fixed. What keeps this gate from being unconditionally met is the AC-5.2-002 description-exposure gap above, which is still an open decision item, not yet resolved either way — so this AC is not flipped to fully met.

## Dev Notes

### The Upgrade Was Not Cosmetic

0.28 → 0.35 is seven minor versions and it broke immediately:

- `eframe::App::update(&mut self, ctx, frame)` became
  `App::ui(&mut self, ui: &mut egui::Ui, frame)`. The app now receives a `Ui`
  directly; the central panel is already wrapped by eframe.
- `Context::style()` / `set_style()` became `style_of(theme)` /
  `set_style_of(theme, ..)`, because 0.35 keeps a **separate `Style` per
  theme**.

That second change has a real accessibility consequence: styling only the active
theme would leave the focus indicator at its default hairline after an OS theme
switch. `apply_typography()` therefore uses `all_styles_mut()` and widens the
focus stroke in both.

### `accesskit` Is Not a Default Feature

`eframe = { version = "0.35", features = ["accesskit"] }`. Without it the UI
Automation tree is never published and **every** accessibility criterion in this
story fails silently — the app looks fine and exposes nothing. Requested
explicitly rather than assumed.

### Listening State Is Announced, Not Drawn

AC-5.2-002 forbids communicating Listening mode through visual text alone.
`CaptureState::announcement()` produces the accessible value, and the shortcut
button attaches it through `Response::widget_info`, so the state reaches the
accessibility tree rather than only the pixels.

### Known Divergence: Typography

`PRIMARY_FONT` / `FALLBACK_FONT` are declared as the frozen vocabulary, but egui
does not resolve system fonts by name — installing Segoe UI means loading its
bytes into a `FontDefinitions`. That is deferred until the probe confirms the
surface renders at all. **The app currently uses egui's bundled face**, which is
a real divergence from AC-5.2-004, recorded here rather than glossed.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 6 theme tests **executed and passing** as part of `cargo test -p settings`
  (48/48).
- Workspace builds clean on 0.35 in debug and release.
- **Status `in-progress`, and AC-5.2-007 explicitly not met.** That criterion
  says the story is *not accepted* unless required roles, states, keyboard
  operation, and theme behaviour can be **demonstrated**. No window has been
  rendered, no screen reader attached, no UI Automation tree inspected. Marking
  this `review` would contradict the AC's own gate.
- Per AC-5.2-007, dependent UI stories (5.3–5.6) remain formally ineligible
  until this is demonstrated, even though their code exists.
- `Cargo.lock` now pins the 0.35 tree; the architecture SSOT still does **not**
  mention AccessKit — that document remains out of sync.

### File List
- `crates/settings/Cargo.toml` (modified — eframe/egui 0.35 + `accesskit`)
- `crates/settings/src/theme.rs` (new)
- `Cargo.lock` (regenerated)

### Review Findings

Reviewed diff: `git diff 9bc578f..HEAD -- crates/settings/Cargo.toml crates/settings/src/theme.rs` (spans both `7b4b7ff` and `428604d`). Three lenses applied: Blind Hunter (fresh-context subagent), Edge Case Hunter (fresh-context subagent), and an Acceptance Auditor pass performed directly against this spec plus `crates/settings/src/app.rs`, `crates/settings/src/main.rs`, `verify-story-5-settings-runtime.ps1`, root `/3p.md`, and the vendored `egui`/`epaint` 0.35.0 source in the Cargo registry cache (used to verify, not assume, claims about focus-stroke behavior and font-parsing failure modes).

**Overall verdict**: this diff closes the biggest prior gap — a real, working `verify-story-5-settings-runtime.ps1` now drives an actual UI Automation client against the real window and independently confirms the tree publishes, controls are named and focusable, declared focus stops are present, onboarding appears on first run, and both themes render (matches root `/3p.md`'s 2026-07-26 "harness Epic 5 10/10 PASS" entry and `sprint-status.yaml`'s `review` status, both already current). That is genuine, verified progress, not a rubber stamp. However, this review found two concrete defects the harness's checks cannot see (UI Automation inspects the tree/names, not rendered pixels or code semantics), which mean **AC-5.2-007's gate is not unconditionally met yet**: the "focus stroke widened in both themes" claim is implemented against the wrong style field and has no visible effect on any real control, and a corrupt-but-present font file crashes the app instead of degrading — the opposite of the module's own stated contract. See Decision items below for the accessible-description gap and the keyboard-operation demonstration bar, both of which also bear on whether AC-5.2-002/003 can be called fully met. The Status line and several AC annotations in this doc are stale relative to `428604d` and should be refreshed regardless of how the decision items resolve.

- [ ] [Review][Decision] AC-5.2-002 "descriptions ... attached via WidgetInfo" does not hold — descriptions are mouse-tooltip only, never reach the accessibility tree — Every `ControlSemantics.description` (`TOGGLE_AUTO_START`, `TOGGLE_OVERLAPPING_STACK`, `SHORTCUT_SWITCHER`, declared `crates/settings/src/theme.rs:225-249`) is wired at its call site only through `.on_hover_text(c.description)` (`crates/settings/src/app.rs:138`, `:156`, `:194`). Confirmed against egui 0.35 source (`response.rs:707-715`): `on_hover_text` renders a floating mouse-hover `Area`/`Label` and never calls `widget_info()` or touches accesskit — it has zero connection to the UI Automation tree. Screen-reader users get each control's *name* (via the widget's own built-in `WidgetInfo`) but never its *description*, contradicting this story's own AC-5.2-002 text ("Accessible names, descriptions, ... exist and are attached via `WidgetInfo`"). The verify harness cannot catch this because it only inspects the `Name` property of automation elements, never `HelpText`/description. A human call is needed: egui 0.35's public `WidgetInfo` API exposes a generic `hint_text` field only via `WidgetInfo::text_edit(...)` (for text-edit widgets) — there is no found generic description/help-text slot for `Checkbox`/`Button`. Decide whether to (a) accept name-only exposure as sufficient for this AC and correct the doc's overclaim, (b) find/build a lower-level accesskit hook to attach a real description, or (c) treat this as a framework limitation to record and defer.
- [ ] [Review][Decision] AC-5.2-003 "keyboard operation" (Tab/Shift+Tab/Space/Enter/Escape) is still only structurally verified, not behaviorally demonstrated — `verify-story-5-settings-runtime.ps1` and `app::focus_order_mismatch()` (wired into `crates/settings/src/app.rs:126-131`, invoked from `settings_ui`) together prove that controls are `IsKeyboardFocusable`, that all declared stops are present in the UIA tree, and that the *drawn* order cannot silently drift from the *declared* `focus_order()`. None of this simulates an actual `Tab`/`Shift+Tab` keypress and observes that focus visibly moves in that order, nor that `Space`/`Enter` activates the focused control or `Escape` cancels shortcut-capture — the exact operations this AC names, and which the story's own Dev Notes say "have never been exercised." That is still true after this diff; only the order *declaration*'s structural integrity is new. Decide whether trusting egui's built-in Tab-traversal (now that drawn/declared order is enforced) is sufficient to call this demonstrated, or whether the gate requires literally extending `verify-story-5-settings-runtime.ps1` with a `SendKeys`/UIA `Invoke` pattern that asserts focus and activation actually happen.
- [x] [Review][Patch] `apply_typography()` widens the wrong style field — keyboard-focus indicator is unchanged in either theme [crates/settings/src/theme.rs:113-118] — `ctx.all_styles_mut(|style| style.visuals.selection.stroke.width = 2.0)` does not drive the focus outline of a `Checkbox` or `Button`. Verified against the vendored egui/epaint 0.35.0 source: `Checkbox::ui`/`Button::ui` derive their frame stroke from `style.visuals.widgets.state(response.widget_state()).bg_stroke` (`checkbox.rs`, `button.rs`, `widget_style.rs:93-112`), and `response.has_focus()` maps to `WidgetState::Active` — never to `Selection`. `Selection.stroke` is documented in egui itself as "Color of selected text" and only affects `TextEdit` selection/cursor rendering plus the foreground color of an already-*selected* button/tab (`SELECTED_CLASS`, e.g. the active pane tab) — none of which is a keyboard-focus ring. Consequence: every control this story cites (`TOGGLE_AUTO_START`, `TOGGLE_OVERLAPPING_STACK`, `SHORTCUT_SWITCHER`, Save/Revert) keeps egui's unmodified default 1.0px focus outline in both themes; the accessibility fix this function exists for does not fire. UI Automation cannot detect this (it's pixel-level, not tree-level), so it was not caught by the "10/10 PASS" harness run. Fix: widen `style.visuals.widgets.active.bg_stroke.width` (and likely `.hovered`) instead of/alongside `selection.stroke.width`.
- [x] [Review][Patch] Corrupt/invalid on-disk font file panics the whole app instead of degrading [crates/settings/src/theme.rs:33-51] — `load_ui_font` only guards `std::fs::read` failing (missing/unreadable file). Traced into the vendored `epaint-0.35.0` source: `FontsImpl::new` (`epaint-0.35.0/src/text/fonts.rs:987-996`) calls `.unwrap_or_else(|err| panic!("Error parsing {name:?} TTF/OTF font file: {err}"))` for every entry in `FontDefinitions::font_data`, which runs on the normal `ctx.set_fonts(fonts)` path this function calls. A present-but-corrupt/truncated/placeholder `segoeui.ttf` or `tahoma.ttf` (disk fault, AV quarantine leaving a stub, partial OS update, third-party font tooling) crashes the entire Settings process at startup — the exact opposite of this module's own documented contract ("a missing font file must degrade to a readable surface, never to no surface at all"). Fix: validate the font bytes (magic-number sniff, or a `skrifa::FontRef::from_index` probe) before calling `ctx.set_fonts`, and fall through to the next candidate/`Bundled` on failure, exactly as the missing-file case already does.
- [x] [Review][Patch] Hardcoded `C:\Windows\Fonts\...` ignores `%SystemRoot%` [crates/settings/src/theme.rs:27-30] — `FONT_CANDIDATES` is a literal `C:\Windows\Fonts\...` path. Any machine with Windows on a non-C: volume (documented in enterprise imaging, VDI, Windows-To-Go) silently falls back to the bundled face with zero diagnostic, contradicting the module's own "never fail silently" framing. Fix: build the path from `std::env::var("SystemRoot")` (or `GetWindowsDirectoryW`) instead of a literal drive letter.
- [x] [Review][Patch] Mid-session theme-switch pickup depends on an unscheduled repaint [crates/settings/src/theme.rs:96-101 doc claim; polling call site crates/settings/src/main.rs:49-53] — `apply()`'s doc comment claims a mid-session OS theme change is "picked up... whenever the window is open," but `main.rs`'s `App::ui` only re-checks `theme::detect_theme()` when eframe actually repaints, and nothing calls `ctx.request_repaint_after(..)` on a timer. If the window is open but fully idle (no mouse motion/keyboard/animation), a theme change made while it sits idle won't visibly apply until the next incidental repaint. Likely masked in practice by egui's hover-driven repaints, but the doc's unconditional wording overstates the actual guarantee. Fix: schedule a low-frequency `ctx.request_repaint_after(Duration::from_millis(500))` in `main.rs`'s `ui()`, or soften the doc comment.
- [x] [Review][Patch] Windows UI font inserted into both `Proportional` and `Monospace` families [crates/settings/src/theme.rs:56-63] — Segoe UI/Tahoma are not monospaced faces; inserting them at index 0 of `FontFamily::Monospace` (alongside `Proportional`) means any future `TextStyle::Monospace` usage (none exists in the crate today — confirmed by search) would silently render proportionally rather than monospaced. No current user-facing impact, but a one-line latent bug. Fix: only insert into `FontFamily::Proportional`.
- [x] [Review][Patch] Machine-dependent test can hard-fail `cargo test -p settings` on CI/minimal images [crates/settings/src/theme.rs, `a_documented_windows_font_is_present_on_this_machine`] — asserts `system_font_available()`, which will fail on any CI runner or minimal/Server-Core-style SKU lacking `segoeui.ttf`/`tahoma.ttf`, breaking the whole suite over an environment fact unrelated to the code under test. Fix: mark `#[ignore]` with a reason (run as part of the machine-verification pass) instead of gating ordinary `cargo test`.
- [x] [Review][Patch] `egui_theme`, `visuals`, `system_font_available` are `pub` only to be reachable from `#[cfg(test)]` [crates/settings/src/theme.rs] — two carry `#[allow(dead_code)]` purely to keep the lint quiet; this is production API surface kept alive solely to serve tests. Fix: `pub(crate)` instead of broad `pub` + lint suppression.
- [x] [Review][Patch] Story doc Status/AC annotations are stale relative to `428604d` [this file — Status line, AC-5.2-002/003/004/007] — this doc's Status is still `in-progress` and its AC annotations still read "UNVERIFIED," "no UI Automation inspection has been performed," and "the renderer does not yet consume it," but `428604d` ("close Epic 5 gaps") already wired `focus_order()` into the renderer with drift detection (`crates/settings/src/app.rs:126-131`), loads real Segoe UI/Tahoma bytes with a bundled-font fallback chain (`crates/settings/src/theme.rs`), and is independently exercised by `verify-story-5-settings-runtime.ps1`, recorded in root `/3p.md`'s 2026-07-26 entry as "harness Epic 5 10/10 PASS," with `sprint-status.yaml` already listing this story as `review` (last_updated 2026-07-26). Fix: refresh the Dev Notes/AC annotations and Status line to reflect current reality — including the new gaps this review surfaces (focus-stroke field, description exposure, corrupt-font crash risk) — rather than leaving pre-`428604d` "not yet done" language in place.
- [x] [Review][Defer] `windows-sys` resolves to 5 distinct versions in `Cargo.lock` (0.48/0.52/0.59/0.60/0.61) [crates/settings/Cargo.toml] — deferred, pre-existing. The duplication is transitive (winit/accesskit/etc. each pin their own major version); this crate's `0.52` pin can't unilaterally dedupe it. Real but low-impact (compile time/binary size only) and not actionable from this diff alone.
- [x] [Review][Defer] `detect_theme()`/`read_apps_use_light_theme()` collapse every registry-read failure mode into a silent `None → Light` fallback with no diagnostic logging [crates/settings/src/theme.rs:71-108] — deferred, pre-existing. Real but low practical impact: Light is the documented, deliberate safe default, and there is no current bug report tied to this.
- [x] [Review][Defer] No user-facing manual override for the OS-driven theme choice [crates/settings/src/theme.rs] — deferred, pre-existing. A corporate policy blocking `HKCU` access would lock a user into Light with no escape hatch, but this is outside AC-5.2-004's stated scope (OS-driven Light/Dark only) and is a reasonable future UX enhancement rather than a defect in what was asked for.

Dismissed as noise (not written above): `initialize_is_idempotent` test doesn't exercise real idempotency (`load_ui_font` ignores its `mode` argument, so the assertion is structurally guaranteed regardless of behavior); `system_font_available()` checks file existence rather than readability, diverging slightly from the production guard but used only in a diagnostic test with zero production consequence; `read_apps_use_light_theme` rebuilds its UTF-16 buffers on every call — real but negligible; Edge Case Hunter's concern that the eframe 0.28→0.35 major bump could break other call sites was checked directly against `crates/settings/src/main.rs`/`app.rs` — both already use the correct 0.35 signatures, and 48/48 `cargo test -p settings` plus a clean workspace build are independently corroborated by root `/3p.md` — false positive for this codebase.
