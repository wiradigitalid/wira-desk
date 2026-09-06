//! DPI-aware thirds snap planning. Pure geometry — no User32.
//! Coordinates arrive already in physical pixels from a Per-Monitor-V2-aware
//! process, so nothing here scales by DPI. The `dpi` field on [`WorkArea`] is
//! carried for traceability only; applying it would scale coordinates twice.

use crate::cycling::WindowId;

use super::{PlacementPlan, PlanError, PlanResult, Rect, WorkArea};

/// Which third column to snap into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThirdColumn {
    Left,
    Middle,
    Right,
}

fn ensure_usable(work: &WorkArea) -> Result<(), PlanError> {
    work.rect.validate()
}

/// Snap active window to the left, middle, or right third of the work area.
pub fn plan_snap_third(work: &WorkArea, window: WindowId, column: ThirdColumn) -> PlanResult {
    ensure_usable(work)?;

    let width = work
        .rect
        .checked_width()
        .ok_or(PlanError::UnrepresentableGeometry)?;

    // Refuse when the work area is too narrow to produce three non-zero columns
    if width < 3 {
        return Err(PlanError::EmptyOrInvertedWorkArea);
    }

    let base = width / 3;
    let rem = width % 3;

    let w_left = base;
    let w_middle = base + rem;

    let x0 = work.rect.left;
    let x1 = x0
        .checked_add(w_left)
        .ok_or(PlanError::UnrepresentableGeometry)?;
    let x2 = x1
        .checked_add(w_middle)
        .ok_or(PlanError::UnrepresentableGeometry)?;
    let x3 = work.rect.right;

    let rect = match column {
        ThirdColumn::Left => Rect::new(x0, work.rect.top, x1, work.rect.bottom)?,
        ThirdColumn::Middle => Rect::new(x1, work.rect.top, x2, work.rect.bottom)?,
        ThirdColumn::Right => Rect::new(x2, work.rect.top, x3, work.rect.bottom)?,
    };

    Ok(PlacementPlan::single(window, rect))
}

pub fn plan_snap_third_left(work: &WorkArea, window: WindowId) -> PlanResult {
    plan_snap_third(work, window, ThirdColumn::Left)
}

pub fn plan_snap_third_middle(work: &WorkArea, window: WindowId) -> PlanResult {
    plan_snap_third(work, window, ThirdColumn::Middle)
}

pub fn plan_snap_third_right(work: &WorkArea, window: WindowId) -> PlanResult {
    plan_snap_third(work, window, ThirdColumn::Right)
}

#[cfg(test)]
pub mod tests {
    use super::super::fixtures::*;
    use super::*;

    fn only(plan: &PlacementPlan) -> Rect {
        assert_eq!(plan.placements.len(), 1);
        plan.placements[0].rect
    }

    const W: WindowId = WindowId(1);

    #[test]
    fn thirds_tile_the_work_area_without_gap_or_overlap() {
        let work = primary_work_area();
        let left = only(&plan_snap_third(&work, W, ThirdColumn::Left).unwrap());
        let middle = only(&plan_snap_third(&work, W, ThirdColumn::Middle).unwrap());
        let right = only(&plan_snap_third(&work, W, ThirdColumn::Right).unwrap());

        // Left aligns with work area left, right aligns with work area right
        assert_eq!(left.left, work.rect.left);
        assert_eq!(right.right, work.rect.right);

        // Columns share internal edges with no gap and no overlap
        assert_eq!(left.right, middle.left);
        assert_eq!(middle.right, right.left);

        // Full vertical span
        assert_eq!(left.top, work.rect.top);
        assert_eq!(left.bottom, work.rect.bottom);
        assert_eq!(middle.top, work.rect.top);
        assert_eq!(middle.bottom, work.rect.bottom);
        assert_eq!(right.top, work.rect.top);
        assert_eq!(right.bottom, work.rect.bottom);

        // Sum of widths equals work area width
        assert_eq!(
            left.width() + middle.width() + right.width(),
            work.rect.width()
        );
    }

    #[test]
    fn remainder_width_goes_to_the_middle_column() {
        // 1367 px width: 1367 / 3 = 455, remainder = 2
        // left: 455, middle: 457, right: 455
        let work = odd_width_work_area();
        assert_eq!(work.rect.width(), 1367);
        let left = only(&plan_snap_third(&work, W, ThirdColumn::Left).unwrap());
        let middle = only(&plan_snap_third(&work, W, ThirdColumn::Middle).unwrap());
        let right = only(&plan_snap_third(&work, W, ThirdColumn::Right).unwrap());

        assert_eq!(left.width(), 455);
        assert_eq!(middle.width(), 457);
        assert_eq!(right.width(), 455);
        assert_eq!(left.width() + middle.width() + right.width(), 1367);

        // Case with remainder = 1: width 1000: 1000 / 3 = 333, remainder = 1
        // left: 333, middle: 334, right: 333
        let work1000 = WorkArea::new(Rect::new(0, 0, 1000, 600).unwrap(), 96).unwrap();
        let l = only(&plan_snap_third(&work1000, W, ThirdColumn::Left).unwrap());
        let m = only(&plan_snap_third(&work1000, W, ThirdColumn::Middle).unwrap());
        let r = only(&plan_snap_third(&work1000, W, ThirdColumn::Right).unwrap());

        assert_eq!(l.width(), 333);
        assert_eq!(m.width(), 334);
        assert_eq!(r.width(), 333);
        assert_eq!(l.width() + m.width() + r.width(), 1000);
    }

    #[test]
    fn narrow_work_area_is_refused() {
        // Width of 2 cannot produce three non-zero columns
        let narrow = WorkArea::new(Rect::new(0, 0, 2, 100).unwrap(), 96).unwrap();
        assert!(plan_snap_third(&narrow, W, ThirdColumn::Left).is_err());
        assert!(plan_snap_third(&narrow, W, ThirdColumn::Middle).is_err());
        assert!(plan_snap_third(&narrow, W, ThirdColumn::Right).is_err());

        // Width of 1 cannot produce three non-zero columns
        let narrow1 = WorkArea::new(Rect::new(0, 0, 1, 100).unwrap(), 96).unwrap();
        assert!(plan_snap_third(&narrow1, W, ThirdColumn::Left).is_err());
    }

    #[test]
    fn negative_origin_work_area_is_supported() {
        let work = negative_origin_work_area();
        let left = only(&plan_snap_third(&work, W, ThirdColumn::Left).unwrap());
        let middle = only(&plan_snap_third(&work, W, ThirdColumn::Middle).unwrap());
        let right = only(&plan_snap_third(&work, W, ThirdColumn::Right).unwrap());

        assert_eq!(left.left, -1920);
        assert_eq!(left.right, -1280);
        assert_eq!(middle.left, -1280);
        assert_eq!(middle.right, -640);
        assert_eq!(right.left, -640);
        assert_eq!(right.right, 0);

        assert_eq!(left.width() + middle.width() + right.width(), 1920);
    }

    #[test]
    fn thirds_stay_inside_the_work_area() {
        for work in [
            primary_work_area(),
            negative_origin_work_area(),
            odd_width_work_area(),
        ] {
            assert!(plan_snap_third(&work, W, ThirdColumn::Left)
                .unwrap()
                .fits_within(&work.rect));
            assert!(plan_snap_third(&work, W, ThirdColumn::Middle)
                .unwrap()
                .fits_within(&work.rect));
            assert!(plan_snap_third(&work, W, ThirdColumn::Right)
                .unwrap()
                .fits_within(&work.rect));
        }
    }

    #[test]
    fn identical_geometry_yields_identical_plans_at_every_dpi() {
        let mut seen: Option<(Rect, Rect, Rect)> = None;
        for work in dpi_variants() {
            let triple = (
                only(&plan_snap_third(&work, W, ThirdColumn::Left).unwrap()),
                only(&plan_snap_third(&work, W, ThirdColumn::Middle).unwrap()),
                only(&plan_snap_third(&work, W, ThirdColumn::Right).unwrap()),
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
            plan_snap_third(&empty, W, ThirdColumn::Left),
            plan_snap_third(&empty, W, ThirdColumn::Middle),
            plan_snap_third(&empty, W, ThirdColumn::Right),
        ] {
            assert_eq!(result, Err(PlanError::EmptyOrInvertedWorkArea));
        }
    }

    #[test]
    fn inverted_work_area_fails_without_placement() {
        let inverted = WorkArea {
            rect: Rect {
                left: 100,
                top: 0,
                right: 10,
                bottom: 100,
            },
            dpi: 96,
        };
        for result in [
            plan_snap_third(&inverted, W, ThirdColumn::Left),
            plan_snap_third(&inverted, W, ThirdColumn::Middle),
            plan_snap_third(&inverted, W, ThirdColumn::Right),
        ] {
            assert_eq!(result, Err(PlanError::EmptyOrInvertedWorkArea));
        }
    }
}
