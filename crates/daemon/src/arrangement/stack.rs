//! Overlapping-stack planning. Pure geometry — no User32.
//! Up to three same-application windows are laid out at a configured width,
//! spread evenly across the leftover horizontal travel so each keeps a visible
//! clickable edge. Disabled or empty input is a **successful no-op**, never an
//! deterministic policy errors for invalid geometry.

use shared::config::LayoutConfig;

use crate::cycling::WindowId;

use super::{Placement, PlacementPlan, PlanError, PlanResult, Rect, WorkArea, STACK_MAX_WINDOWS};

/// Plan an overlapping stack for `candidates` in their accepted live order.
pub fn plan_stack(layout: &LayoutConfig, work: &WorkArea, candidates: &[WindowId]) -> PlanResult {
    if !layout.enable_overlapping_stack {
        return Ok(PlacementPlan::empty());
    }
    work.rect.validate()?;

    let percent = layout.stack_width_percent;
    if percent == 0 || percent > 100 {
        return Err(PlanError::InvalidWidthPercent(percent));
    }

    // A disabled stack short-circuits above, so an empty candidate list here is
    // still a success — there is simply nothing to arrange.
    if candidates.is_empty() {
        return Ok(PlacementPlan::empty());
    }

    let work_width = work.rect.width();
    let window_width = (work_width as i64 * percent as i64 / 100) as i32;
    if window_width <= 0 {
        return Err(PlanError::InvalidWidthPercent(percent));
    }

    // Leftover horizontal room the anchors are distributed across.
    let travel = work_width - window_width;

    // Live order is preserved; only the first three are arranged.
    let chosen: Vec<WindowId> = candidates.iter().copied().take(STACK_MAX_WINDOWS).collect();

    let placements = chosen
        .iter()
        .enumerate()
        .map(|(i, window)| {
            let offset = anchor_offset(i, chosen.len(), travel);
            let left = work
                .rect
                .left
                .checked_add(offset)
                .ok_or(PlanError::UnrepresentableGeometry)?;
            let right = left
                .checked_add(window_width)
                .ok_or(PlanError::UnrepresentableGeometry)?;
            let rect = Rect::new(left, work.rect.top, right, work.rect.bottom)?;
            Ok(Placement {
                window: *window,
                rect,
            })
        })
        .collect::<Result<Vec<Placement>, PlanError>>()?;

    Ok(PlacementPlan { placements })
}

/// Horizontal anchor for window `index` of `count`, across `travel` pixels.
/// One window is centred; two are pinned to the ends; three take start, middle,
/// and end. Distributing across the *travel* range rather than the full width
/// is what keeps every rectangle inside the work area by construction.
fn anchor_offset(index: usize, count: usize, travel: i32) -> i32 {
    match count {
        1 => travel / 2,
        _ => {
            // Evenly spaced: index * travel (count - 1), integer-rounded.
            let denom = (count - 1) as i64;
            ((index as i64 * travel as i64) / denom) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    fn enabled(percent: u32) -> LayoutConfig {
        LayoutConfig {
            enable_overlapping_stack: true,
            stack_width_percent: percent,
            ..LayoutConfig::default()
        }
    }

    // --- disabled and empty are successful no-ops --------------

    #[test]
    fn disabled_stack_is_a_successful_noop() {
        let layout = LayoutConfig::default(); // disabled
        let plan = plan_stack(&layout, &primary_work_area(), &windows(3)).unwrap();
        assert!(plan.is_noop());
    }

    #[test]
    fn disabled_stack_ignores_even_invalid_width() {
        let layout = LayoutConfig {
            enable_overlapping_stack: false,
            stack_width_percent: 0,
            ..LayoutConfig::default()
        };
        assert!(plan_stack(&layout, &primary_work_area(), &windows(2))
            .unwrap()
            .is_noop());
    }

    #[test]
    fn zero_candidates_is_a_successful_noop() {
        let plan = plan_stack(&enabled(50), &primary_work_area(), &[]).unwrap();
        assert!(plan.is_noop());
    }

    // --- order and cap -----------------------------------------

    #[test]
    fn live_order_is_preserved() {
        let candidates = vec![WindowId(7), WindowId(3), WindowId(9)];
        let plan = plan_stack(&enabled(50), &primary_work_area(), &candidates).unwrap();
        let got: Vec<WindowId> = plan.placements.iter().map(|p| p.window).collect();
        assert_eq!(got, candidates);
    }

    #[test]
    fn no_more_than_three_windows_are_arranged() {
        let plan = plan_stack(&enabled(50), &primary_work_area(), &windows(7)).unwrap();
        assert_eq!(plan.placements.len(), STACK_MAX_WINDOWS);
        let got: Vec<WindowId> = plan.placements.iter().map(|p| p.window).collect();
        assert_eq!(got, windows(3), "the first three in live order");
    }

    // --- anchoring at the default width ------------------------

    #[test]
    fn one_window_is_centred() {
        let work = primary_work_area(); // width 1920
        let plan = plan_stack(&enabled(50), &work, &windows(1)).unwrap();
        let r = plan.placements[0].rect;
        assert_eq!(r.width(), 960);
        // travel = 960, centred at 480.
        assert_eq!(r.left, 480);
        assert_eq!(r.right, 1440);
        assert_eq!(r.left - work.rect.left, work.rect.right - r.right);
    }

    #[test]
    fn two_windows_anchor_left_and_right() {
        let work = primary_work_area();
        let plan = plan_stack(&enabled(50), &work, &windows(2)).unwrap();
        assert_eq!(plan.placements[0].rect.left, 0);
        assert_eq!(plan.placements[1].rect.right, 1920);
    }

    #[test]
    fn three_windows_anchor_left_centre_and_right() {
        let work = primary_work_area();
        let plan = plan_stack(&enabled(50), &work, &windows(3)).unwrap();
        let lefts: Vec<i32> = plan.placements.iter().map(|p| p.rect.left).collect();
        assert_eq!(lefts, vec![0, 480, 960]);
        assert_eq!(plan.placements[2].rect.right, 1920);
    }

    #[test]
    fn every_window_keeps_a_visible_clickable_edge() {
        let work = primary_work_area();
        for n in 2..=3 {
            let plan = plan_stack(&enabled(50), &work, &windows(n)).unwrap();
            let lefts: Vec<i32> = plan.placements.iter().map(|p| p.rect.left).collect();
            for pair in lefts.windows(2) {
                assert!(
                    pair[1] > pair[0],
                    "windows {pair:?} fully overlap — no clickable edge"
                );
            }
        }
    }

    // --- width policy and containment --------------------------

    #[test]
    fn zero_percent_is_a_deterministic_policy_error() {
        assert_eq!(
            plan_stack(&enabled(0), &primary_work_area(), &windows(2)),
            Err(PlanError::InvalidWidthPercent(0))
        );
    }

    #[test]
    fn above_one_hundred_percent_is_a_deterministic_policy_error() {
        assert_eq!(
            plan_stack(&enabled(101), &primary_work_area(), &windows(2)),
            Err(PlanError::InvalidWidthPercent(101))
        );
        assert_eq!(
            plan_stack(&enabled(1000), &primary_work_area(), &windows(1)),
            Err(PlanError::InvalidWidthPercent(1000))
        );
    }

    #[test]
    fn all_rectangles_stay_inside_the_work_area() {
        for work in [
            primary_work_area(),
            negative_origin_work_area(),
            odd_width_work_area(),
        ] {
            for percent in [10u32, 33, 50, 75, 100] {
                for n in 1..=3 {
                    let plan = plan_stack(&enabled(percent), &work, &windows(n)).unwrap();
                    assert!(
                        plan.fits_within(&work.rect),
                        "escaped work area: percent={percent} n={n} dpi={}",
                        work.dpi
                    );
                }
            }
        }
    }

    #[test]
    fn full_width_degenerates_to_a_single_position() {
        // 100 percent is valid but leaves zero travel, so the windows coincide
        // and no clickable edge remains. Documented, not silently prevented.
        let work = primary_work_area();
        let plan = plan_stack(&enabled(100), &work, &windows(3)).unwrap();
        let lefts: Vec<i32> = plan.placements.iter().map(|p| p.rect.left).collect();
        assert_eq!(lefts, vec![0, 0, 0]);
        assert!(plan.fits_within(&work.rect));
    }

    #[test]
    fn width_too_small_to_represent_is_a_policy_error() {
        // 1 percent of a 50 px work area rounds to zero pixels.
        let tiny = WorkArea::new(Rect::new(0, 0, 50, 100).unwrap(), 96).unwrap();
        assert_eq!(
            plan_stack(&enabled(1), &tiny, &windows(2)),
            Err(PlanError::InvalidWidthPercent(1))
        );
    }

    // --- odd, negative, and DPI fixtures -----------------------

    #[test]
    fn negative_origin_anchors_correctly() {
        let work = negative_origin_work_area(); // left -1920, width 1920
        let plan = plan_stack(&enabled(50), &work, &windows(2)).unwrap();
        assert_eq!(plan.placements[0].rect.left, -1920);
        assert_eq!(plan.placements[1].rect.right, 0);
    }

    #[test]
    fn odd_dimensions_produce_deterministic_results() {
        let work = odd_width_work_area(); // width 1367
        let first = plan_stack(&enabled(50), &work, &windows(3)).unwrap();
        for _ in 0..4 {
            assert_eq!(plan_stack(&enabled(50), &work, &windows(3)).unwrap(), first);
        }
        assert!(first.fits_within(&work.rect));
    }

    #[test]
    fn identical_geometry_yields_identical_plans_at_every_dpi() {
        let mut seen: Option<PlacementPlan> = None;
        for work in dpi_variants() {
            let plan = plan_stack(&enabled(50), &work, &windows(3)).unwrap();
            match &seen {
                None => seen = Some(plan),
                Some(prev) => assert_eq!(
                    *prev, plan,
                    "DPI {} changed the plan; coordinates were scaled twice",
                    work.dpi
                ),
            }
        }
    }

    #[test]
    fn invalid_work_area_fails_without_placement() {
        let inverted = WorkArea {
            rect: Rect {
                left: 100,
                top: 0,
                right: 0,
                bottom: 100,
            },
            dpi: 96,
        };
        assert_eq!(
            plan_stack(&enabled(50), &inverted, &windows(2)),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }
}
