//! DPI-aware snap planning. Pure geometry — no User32.
//! Coordinates arrive already in physical pixels from a Per-Monitor-V2-aware
//! process, so nothing here scales by DPI. The `dpi` field on [`WorkArea`] is
//! carried for traceability only; applying it would scale coordinates twice.

use crate::cycling::WindowId;

use super::{PlacementPlan, PlanError, PlanResult, Rect, WorkArea};

/// The x coordinate that divides the work area into halves.
/// An odd width is split deterministically: the left half takes the floor, the
/// right half takes the remainder. Because both halves are computed from this
/// one boundary, `left.right == right.left` always holds — the halves tile the
/// work area with neither a one-pixel gap nor a one-pixel overlap.
fn split_x(work: &Rect) -> Result<i32, PlanError> {
    let width = work
        .checked_width()
        .ok_or(PlanError::UnrepresentableGeometry)?;
    if width <= 0 {
        return Err(PlanError::EmptyOrInvertedWorkArea);
    }
    work.left
        .checked_add(width / 2)
        .ok_or(PlanError::UnrepresentableGeometry)
}

/// Validate before planning. Uses [`Rect::validate`] rather than `is_valid`
/// so an unrepresentable work area is reported as such instead of being
/// misfiled as merely empty.
fn ensure_usable(work: &WorkArea) -> Result<(), PlanError> {
    work.rect.validate()
}

/// Left half of the usable work area.
pub fn plan_snap_left(work: &WorkArea, window: WindowId) -> PlanResult {
    ensure_usable(work)?;
    let mid = split_x(&work.rect)?;
    let rect = Rect::new(work.rect.left, work.rect.top, mid, work.rect.bottom)?;
    Ok(PlacementPlan::single(window, rect))
}

/// Right half — the exact complement of [`plan_snap_left`].
pub fn plan_snap_right(work: &WorkArea, window: WindowId) -> PlanResult {
    ensure_usable(work)?;
    let mid = split_x(&work.rect)?;
    let rect = Rect::new(mid, work.rect.top, work.rect.right, work.rect.bottom)?;
    Ok(PlacementPlan::single(window, rect))
}

/// The complete usable work area.
/// Note this is the *work area*, never full monitor bounds: the taskbar and any
/// reserved appbar are already excluded by whoever supplied the [`WorkArea`],
/// and re-expanding to monitor bounds would cover them.
pub fn plan_snap_maximize(work: &WorkArea, window: WindowId) -> PlanResult {
    ensure_usable(work)?;
    let rect = Rect::new(
        work.rect.left,
        work.rect.top,
        work.rect.right,
        work.rect.bottom,
    )?;
    Ok(PlacementPlan::single(window, rect))
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    fn only(plan: &PlacementPlan) -> Rect {
        assert_eq!(plan.placements.len(), 1);
        plan.placements[0].rect
    }

    const W: WindowId = WindowId(1);

    // --- 002: halves ------------------------------------------

    #[test]
    fn snap_left_returns_left_half() {
        let work = primary_work_area();
        let r = only(&plan_snap_left(&work, W).unwrap());
        assert_eq!(r, Rect::new(0, 0, 960, 1040).unwrap());
    }

    #[test]
    fn snap_right_returns_complementary_half() {
        let work = primary_work_area();
        let r = only(&plan_snap_right(&work, W).unwrap());
        assert_eq!(r, Rect::new(960, 0, 1920, 1040).unwrap());
    }

    #[test]
    fn halves_tile_the_work_area_without_gap_or_overlap() {
        for work in [
            primary_work_area(),
            negative_origin_work_area(),
            odd_width_work_area(),
        ] {
            let left = only(&plan_snap_left(&work, W).unwrap());
            let right = only(&plan_snap_right(&work, W).unwrap());
            assert_eq!(left.right, right.left, "gap or overlap at the seam");
            assert_eq!(left.left, work.rect.left);
            assert_eq!(right.right, work.rect.right);
            assert_eq!(
                left.width() + right.width(),
                work.rect.width(),
                "halves must exactly cover the work area"
            );
        }
    }

    #[test]
    fn odd_width_is_split_deterministically() {
        let work = odd_width_work_area(); // width 1367
        let left = only(&plan_snap_left(&work, W).unwrap());
        let right = only(&plan_snap_right(&work, W).unwrap());
        assert_eq!(left.width(), 683);
        assert_eq!(right.width(), 684);
        // Repeat: the split must not drift.
        for _ in 0..4 {
            assert_eq!(only(&plan_snap_left(&work, W).unwrap()), left);
            assert_eq!(only(&plan_snap_right(&work, W).unwrap()), right);
        }
    }

    #[test]
    fn halves_stay_inside_the_work_area() {
        for work in [
            primary_work_area(),
            negative_origin_work_area(),
            odd_width_work_area(),
        ] {
            assert!(plan_snap_left(&work, W).unwrap().fits_within(&work.rect));
            assert!(plan_snap_right(&work, W).unwrap().fits_within(&work.rect));
        }
    }

    #[test]
    fn halves_work_at_negative_origin() {
        let work = negative_origin_work_area(); // left -1920, width 1920
        let left = only(&plan_snap_left(&work, W).unwrap());
        let right = only(&plan_snap_right(&work, W).unwrap());
        assert_eq!(left, Rect::new(-1920, -200, -960, 880).unwrap());
        assert_eq!(right, Rect::new(-960, -200, 0, 880).unwrap());
    }

    // --- maximize uses the work area ---------------------------

    #[test]
    fn maximize_returns_the_whole_work_area() {
        let work = primary_work_area();
        let r = only(&plan_snap_maximize(&work, W).unwrap());
        assert_eq!(r, work.rect);
    }

    #[test]
    fn maximize_never_exceeds_the_work_area() {
        // The taskbar strip below 1040 must remain uncovered.
        let work = primary_work_area();
        let r = only(&plan_snap_maximize(&work, W).unwrap());
        assert_eq!(r.bottom, 1040);
        assert!(r.bottom < 1080, "maximize reached full monitor bounds");
    }

    // --- DPI is never applied twice ----------------------------

    #[test]
    fn identical_geometry_yields_identical_plans_at_every_dpi() {
        let mut seen: Option<(Rect, Rect, Rect)> = None;
        for work in dpi_variants() {
            let triple = (
                only(&plan_snap_left(&work, W).unwrap()),
                only(&plan_snap_right(&work, W).unwrap()),
                only(&plan_snap_maximize(&work, W).unwrap()),
            );
            match &seen {
                None => seen = Some(triple),
                Some(prev) => assert_eq!(
                    *prev, triple,
                    "DPI {} changed the plan; coordinates were scaled twice",
                    work.dpi
                ),
            }
        }
    }

    // --- deterministic failure, never a partial placement ------

    #[test]
    fn empty_work_area_fails_without_placement() {
        let empty = WorkArea {
            rect: Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 1040,
            },
            dpi: 96,
        };
        for result in [
            plan_snap_left(&empty, W),
            plan_snap_right(&empty, W),
            plan_snap_maximize(&empty, W),
        ] {
            assert_eq!(result, Err(PlanError::EmptyOrInvertedWorkArea));
        }
    }

    #[test]
    fn inverted_work_area_fails_without_placement() {
        let inverted = WorkArea {
            rect: Rect {
                left: 1920,
                top: 0,
                right: 0,
                bottom: 1040,
            },
            dpi: 96,
        };
        assert_eq!(
            plan_snap_left(&inverted, W),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }

    #[test]
    fn unrepresentable_work_area_fails_without_placement() {
        let huge = WorkArea {
            rect: Rect {
                left: i32::MIN,
                top: 0,
                right: i32::MAX,
                bottom: 1040,
            },
            dpi: 96,
        };
        assert_eq!(
            plan_snap_left(&huge, W),
            Err(PlanError::UnrepresentableGeometry)
        );
    }

    #[test]
    fn one_pixel_wide_work_area_still_splits_safely() {
        // Degenerate but representable: the left half becomes empty, which the
        // Rect constructor rejects rather than emitting a zero-width placement.
        let sliver = WorkArea::new(Rect::new(0, 0, 1, 100).unwrap(), 96).unwrap();
        assert_eq!(
            plan_snap_left(&sliver, W),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
        // The right half is the whole sliver and remains valid.
        assert_eq!(
            only(&plan_snap_right(&sliver, W).unwrap()),
            Rect::new(0, 0, 1, 100).unwrap()
        );
    }
}
