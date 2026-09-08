# Smoke test — mandate `DEC-020` (SPEC-6 Delivery)

Date: 2026-09-08
Head SHA: 0328d4f
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | DEF-16 (Tooltip over next row) | Tooltip on middle row (Switcher) renders above next row (Fallback) controls | Tooltip rendered via window-level floating overlay, painted after all row cards; verified by `tooltip_paints_above_the_next_row_when_it_overflows_into_it` | PASS |
| 2 | DEF-16 (Tooltip over KeyCheck) | Tooltip on last row (Overlapping Stack) renders above pinned KeyCheck panel | Tooltip rendered via window-level floating overlay, painted after KeyCheck band; verified by `the_last_rows_tooltip_paints_above_the_key_check_panel` | PASS |
| 3 | DEF-16 (Tooltip size & height) | Tooltip height remains constrained to single line text (~22px with padding) without overlap | Height matches `preferred-height`, no title overlap; verified by `tooltip_height_fits_its_own_text` | PASS |
| 4 | DEF-16 (Keyboard focus tooltip) | Tooltip surfaces on keyboard focus when tabbing to title block, and hides on tab departure | FocusScope triggers `focus_gained`/`focus_lost` without stealing focus; verified by `description_renders_as_a_tooltip_not_a_visible_line` | PASS |
| 5 | DEF-17 (Above-max recovery) | Typing 101 into snap percentage (1-99) and clicking '-' steps down to 99 | Stepper guard clamps raw value to `percent_max` (99), recovering from frozen state; verified by `stepper_recovers_from_an_above_max_typed_value` | PASS |
| 6 | DEF-17 (Above-max '+' clamp) | Clicking '+' when value is above max remains clamped at 99 | Value clamped at max bound, saving valid 99 without error | PASS |
| 7 | DEF-17 (Below-min recovery) | Typing 0 into snap percentage (1-99) and clicking '+' steps up to 1 | Stepper guard clamps raw value to `percent_min` (1), recovering from frozen state; verified by `stepper_recovers_from_a_below_min_typed_value` | PASS |
| 8 | DEF-17 (Below-min '-' clamp) | Clicking '-' when value is below min remains clamped at 1 | Value clamped at min bound, saving valid 1 without error | PASS |
| 9 | DEF-17 (Stack above-max recovery) | Typing 150 into Stack width (10-100) and clicking '-' recovers to 100 | Stepper recovers to `MAX_STACK_WIDTH_PERCENT` (100); verified by `stack_row_stepper_recovers_from_an_out_of_range_typed_value` | PASS |
| 10 | DEF-17 (Stack below-min recovery) | Typing 5 into Stack width (10-100) and clicking '+' recovers to 10 | Stepper recovers to `MIN_STACK_WIDTH_PERCENT` (10); verified by `stack_row_stepper_recovers_from_an_out_of_range_typed_value` | PASS |
| 11 | General | 5 groups in Shortcuts pane render with no horizontal scroll and workspace suite passes | All 568 tests pass, clippy clean, fmt clean, export hygiene passes | PASS |

11 PASS, 0 FAIL, 0 NOT VERIFIABLE.
Both defects (DEF-16, DEF-17) confirmed fixed.
