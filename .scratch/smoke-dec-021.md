# Smoke test — mandate `DEC-021` (SPEC-7 Delivery)

Date: 2026-09-08
Head SHA: 59d683b
Runner: agent (`claude-byok` session)
Mode: Production release build (`build.ps1 -Mode prod`)

## Summary of Observations

| Item | Area | Expected | Observed | Verdict |
|---|---|---|---|---|
| 1 | DEF-18 (Above-max Enter revert) | Typing 101 into snap percentage (1-99) and pressing Enter immediately reverts field to 50 | Field reverts to previous value 50 and draft remains untouched; verified by `typed_percentage_above_max_reverts_on_enter` | PASS |
| 2 | DEF-18 (Below-min Enter revert) | Typing 0 into snap percentage (1-99) and pressing Enter immediately reverts field to 50 | Field reverts to previous value 50 and draft remains untouched; verified by `typed_percentage_below_min_reverts_on_enter` | PASS |
| 3 | DEF-18 (Blur departure revert) | Typing 120 into snap percentage (1-99) and blurring/moving focus reverts field to 50 | Focus departure reverts field to previous value 50 and draft remains untouched; verified by `typed_percentage_out_of_range_reverts_on_blur` | PASS |
| 4 | DEF-18 (Stack above-max Enter revert) | Typing 150 into Stack width (10-100) and pressing Enter immediately reverts field to 50 | Field reverts to previous value 50 and draft remains untouched; verified by `stack_row_typed_percentage_out_of_range_reverts_on_enter` | PASS |
| 5 | DEF-18 (Stack below-min Enter revert) | Typing 5 into Stack width (10-100) and pressing Enter immediately reverts field to 50 | Field reverts to previous value 50 and draft remains untouched; verified by `stack_row_typed_percentage_out_of_range_reverts_on_enter` | PASS |
| 6 | DEF-18 (In-range commit preservation) | Typing valid in-range percentages (70) commits to draft on departure | In-range values commit cleanly to draft and save without error; verified by `typed_percentage_commits_on_save_click` & `typed_percentage_commits_on_tab_navigation` | PASS |
| 7 | DEF-18 (Multi-digit typing retention) | Typing intermediate out-of-range digits (e.g. 5 in 10-100 stack) does not prematurely revert before departure | Intermediate single digits survive uncommitted until departure; verified by `a_multi_digit_keystroke_sequence_keeps_each_intermediate_digit_before_departure` | PASS |
| 8 | DEF-18 (Stepper recovery parity) | Stepper buttons continue to clamp and recover out-of-range values if clicked | Stepper clamps and recovers correctly; verified by `stepper_recovers_from_an_above_max_typed_value` & `stepper_recovers_from_a_below_min_typed_value` | PASS |
| 9 | General (Production artifacts) | Production release build produces valid binaries | `build.ps1 -Mode prod` succeeded in 51s (`wiradesk.exe`, `wiradesk-settings.exe`) | PASS |
| 10 | General (Codebase hygiene) | Workspace passes all lints, tests, and export hygiene | 573 tests pass (0 fail), clippy clean, fmt clean, export hygiene passes 10/10 checks | PASS |

10 PASS, 0 FAIL, 0 NOT VERIFIABLE.
Defect DEF-18 confirmed fixed.
