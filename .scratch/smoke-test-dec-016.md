# Smoke test — autopilot mandate DEC-016

You are running the **smoke test** for this mandate's Finish step. This is testing only — you are not
building or fixing anything. Report honestly what you can and cannot verify from this session; do not
round "the code is tested" up to "the feature was smoke-tested live."

## What this mandate delivered

- `SPEC-2-01` (`UC-4`/`UC-9`): commit-on-departure for typed percentage fields in the Shortcuts and Layout
  panes (Save/steppers/Tab/row-navigation all commit, including the cross-row fix), stepper refusal parity
  (`layout_pane.slint` no longer silently clamps out-of-range values), in-pane heading corrected to
  "Layout".
- `SPEC-2-02` (`FR-25`/`UC-8`, backfill of already-shipped code): "Check for Updates" in the About pane.
- `SPEC-3-01`/`SPEC-3-02` (`FR-28`/`FR-29`): every one of the 16 editable shortcut actions gets its own
  persisted enable/disable flag; a disabled action's chord is excluded from keyboard-hook registration
  entirely (falls through to the OS/other applications) rather than being claimed and discarded.

## Steps

1. Build the release binaries: `./build.ps1` (from the repo root, this worktree, `autopilot/DEC-016`
   checked out).
2. Run the full workspace test suite once more as a final confirmation:
   `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace` — record pass/fail count.
3. Launch the Settings GUI (`wiradesk-settings.exe`) and exercise, live, whatever this session can reach
   without elevation or keyboard-input-injection tooling:
   - **FR-28 proof of done** (per the PRD): turn off "Snap to left third" in Settings, save, confirm every
     other action's chord is untouched and the disabled row reads visibly differently from a collision-unbound
     row (`BR-9`).
   - **`SPEC-2-01` proof**: type a percentage value into a Shortcuts-pane field WITHOUT pressing Enter, click
     Save, confirm the typed value was committed (not the old one) — and the same for a stepper click and a
     Tab-away.
   - **`SPEC-2-02` proof (FR-25)**: confirm the "Check for Updates" control in the About pane is present and
     clickable; you do NOT need a real newer release to test against — record what's reachable.
4. For whatever is NOT reachable from this headless session — most likely: the daemon itself (it links an
   elevation manifest and needs interactive UAC to start elevated), and therefore **FR-29's actual proof**
   (a real global keypress reaching another application when the action is disabled) and FR-25's final
   elevated-launch step — say so explicitly rather than guessing or skipping silently. Name exactly what
   blocked it (e.g. "no interactive UAC in this session", "no keyboard-input-injection tooling here").
5. Do not modify any code. Do not commit anything. This is a read/run-only pass.

## Report back

One structured report: for each of `FR-25`, `FR-28`, `FR-29`, and the `SPEC-2-01` UI fix — PASS (with what
you actually did to verify it), or NOT VERIFIABLE FROM THIS SESSION (with the specific reason). Include the
release build result and the final full-suite test count.
