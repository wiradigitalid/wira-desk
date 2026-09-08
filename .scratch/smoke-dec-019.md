# Smoke test — mandate `DEC-019` (SPEC-5 Delivery)

Date: 2026-09-08
Head SHA: 16a3dc9
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | DEF-14 (Tooltip size) | Tooltip height fits its single line of text (~22px total height), not stretched to 50px | Height constrained to `tooltip_layout.preferred-height` (rendered height 14px line, ~22px with padding) | PASS |
| 2 | DEF-14 (Tooltip position) | Tooltip does not overlap title line of hovered row or neighbouring text | Tooltip positioned below row with clean separation; 2D bounds non-overlapping with title Text | PASS |
| 3 | DEF-14 (Tooltip styling) | Solid elevated background with clear border, not faint or near-transparent | Background `#282C34` (dark) / `#FFFFFF` (light) with 16% alpha stroke, high contrast over card | PASS |
| 4 | DEF-14 (Longest description) | Longest description string (SnapPercentBottom, 65 chars) fits in single line | Width constrained to `min(480px, preferred-width)` fitting 65-char description on one line | PASS |
| 5 | DEF-15 (Toggle height) | Row height stays identical toggle-on vs toggle-off when "Disabled" appears | Keycap pitch is exactly 54.0px toggle-on and 54.0px toggle-off (0.0px delta); "Disabled" rendered | PASS |
| 6 | DEF-15 (No horizontal scroll) | 53px row height across 16 rows fits without horizontal scroll | All 5 groups fit standard width (760px) with 0 horizontal scroll; vertical scroll in Flickable | PASS |
| 7 | DEF-13 (Multi-digit typing) | Typing "5" then "5" to reach "55" keeps intermediate "5" without mid-keystroke revert | "5" remains in field after first keystroke; "55" completes on second keystroke | PASS |
| 8 | DEF-13 (Stack min bound) | Typing "7" then "5" on Stack row (bounds 10-100) keeps intermediate "7" | "7" is not reverted despite being momentarily below minimum 10 | PASS |
| 9 | DEF-13 (Backspace deletion) | Backspacing from 2 digits to 1 digit retains the single digit before departure | Field retains single digit after backspace without reverting to saved value | PASS |
| 10 | DEF-13 (Departure refusal) | Departing field with out-of-range value refuses with actionable message | Refused with actionable error on save/departure, saved config untouched | PASS |
| 11 | General | 5 groups in Shortcuts pane in order per DEC-014 | Switching, Snap to half, Snap to third, Snap to custom, Resize/move/arrange in order | PASS |

11 PASS, 0 FAIL, 0 NOT VERIFIABLE.
All three defects (DEF-13, DEF-14, DEF-15) confirmed fixed.
