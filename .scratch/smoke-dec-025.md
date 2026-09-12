# Smoke Test Execution — Mandate DEC-025

**Date:** 2026-09-13
**Mandate:** DEC-025
**Scope:** SPEC-12 (About Pane & Settings Polish), SPEC-13 (Same-App Visual Switcher)
**Executor:** Agent (headless, automated test suite & binary verification)

---

### Specifications Delivered

#### 1. SPEC-12: About Pane & Settings Polish
- **SPEC-12-01**: About pane 3-pillar product description, in-process updater disclosure, external vector link icons (`↗`), and dedicated Publisher website row.
  - **Verdict:** PASS
  - **Evidence:** `app::tests::about_pane_renders_three_pillars_description`, `app::tests::about_pane_renders_in_process_disclosure`, `app::tests::about_pane_renders_open_link_icons_and_reordered_hierarchy`.
- **SPEC-12-02**: Mouse preset dropdown category header click absorption & inertness.
  - **Verdict:** PASS
  - **Evidence:** `app::tests::clicking_preset_dropdown_category_header_does_not_dismiss_overlay`.
- **SPEC-12-03**: Save Changes button dirty state visual affordance and accessible default action gating.
  - **Verdict:** PASS
  - **Evidence:** `app::tests::save_changes_button_disabled_when_clean_and_enabled_when_dirty`.

#### 2. SPEC-13: Same-App Visual Switcher
- **SPEC-13-01**: Tracer bullet — hold cycle chord past 150ms opens overlay, release commits, Escape restores origin window, candidate set parity with blind cycle.
  - **Verdict:** PASS
  - **Evidence:** `switcher::tests::switcher_candidate_set_equals_blind_cycle_eligible_set`, `switcher::tests::card_order_equals_cycle_order`, `hook::tests::main_key_release_before_deadline_disarms_without_delaying_cycle`, `hook::tests::deadline_with_modifiers_down_opens_the_switcher`, `hook::tests::a_chord_without_a_modifier_never_arms_the_switcher`, `switcher::thumbnail::tests::every_registration_is_unregistered_on_dismissal`, `worker::tests::switcher_commit_falls_through_on_invalid_target`.
- **SPEC-13-02**: Adaptive grid layout derived from monitor work area, pagination, in-page navigation clamping, and GDI page dots (`● ○`).
  - **Verdict:** PASS
  - **Evidence:** `switcher::layout::tests::columns_derive_from_the_work_area_and_never_overflow_it`, `switcher::layout::tests::candidates_beyond_one_page_paginate_rather_than_shrink`, `switcher::selection::tests::advancing_past_a_page_edge_turns_the_page`, `switcher::selection::tests::up_and_down_clamp_within_page`.
- **SPEC-13-03**: Card chrome with app icon, cached window title, DWM rounded corners, theme-aware canvas/card palette, per-monitor DPI font/icon scaling, and mouse hover/click commit.
  - **Verdict:** PASS
  - **Evidence:** `switcher::overlay::tests::chrome_never_overlaps_the_preview_rectangle`, `switcher::overlay::tests::a_failed_registration_still_renders_icon_and_title`.
- **SPEC-13-04**: Configuration settings (`visual_enabled`, `visual_hold_delay_ms`), persistence validation (100..=500ms), GeneralPane UI toggle and spinner, and daemon reload.
  - **Verdict:** PASS
  - **Evidence:** `config::tests::a_pre_spec_13_config_gains_both_fields_without_migration`, `persistence::tests::visual_hold_delay_outside_its_bounds_is_rejected`, `app::tests::general_pane_exposes_the_visual_switcher_toggle`, `hook::tests::reload_with_visual_switcher_disabled_leaves_blind_cycling_unchanged`.

---

### Verification Summary
- **Workspace Test Suite:** 382 daemon tests + 190 settings tests + 70 shared tests (642 tests total) all GREEN.
- **Compiler Hygiene:** `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Binary Imports:** `scripts/verify-release-binary.ps1` confirms valid release binaries in `target/release/`.
- **Public Export Hygiene:** `scripts/verify-public-export.ps1` passes 10/10 checks.
- **Method Validation:** `validate.py --generate` passes GREEN with zero findings.
