//! Frozen arrangement contract — pure geometry, no User32.
//! Rectangles use **signed physical-pixel** coordinates with **half-open**
//! edges: `left <= x < right`, `top <= y < bottom`. Signed because a secondary
//! monitor legitimately sits at negative coordinates; half-open because it makes
//! "left half plus right half exactly tiles the work area" true by construction
//! rather than by an off-by-one convention everyone has to remember.
//! Every arithmetic step is checked. A geometry that cannot be represented
//! safely yields a [`PlanError`] rather than a partial or wrapped placement.
//! Ownership after this story is frozen :
//! - `arrangement/snap.rs` →
//! - `arrangement/stack.rs` →
//! - `arrangement/win32.rs` →
//! - `arrangement/mod.rs`, `hook.rs`, `worker.rs`, final composition →

// Published ahead of its consumers, same as the and contracts.
#![allow(dead_code)]

pub mod snap;
pub mod stack;
pub mod win32;

use crate::cycling::WindowId;

/// Half-open rectangle in signed physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// Construct a rectangle, rejecting empty or inverted geometry.
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Result<Rect, PlanError> {
        let rect = Rect {
            left,
            top,
            right,
            bottom,
        };
        rect.validate()?;
        Ok(rect)
    }

    pub fn checked_width(&self) -> Option<i32> {
        self.right.checked_sub(self.left)
    }

    pub fn checked_height(&self) -> Option<i32> {
        self.bottom.checked_sub(self.top)
    }

    /// Width. Only meaningful once the rectangle has been validated.
    pub fn width(&self) -> i32 {
        self.checked_width().unwrap_or(0)
    }

    pub fn height(&self) -> i32 {
        self.checked_height().unwrap_or(0)
    }

    /// Convenience predicate. Prefer [`Rect::validate`] on any path that
    /// reports an error: this collapses "overflowed" and "empty" into one
    /// `false` and cannot tell the caller which happened.
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Validate extents, distinguishing an unrepresentable rectangle from a
    /// merely empty or inverted one.
    pub fn validate(&self) -> Result<(), PlanError> {
        let width = self
            .checked_width()
            .ok_or(PlanError::UnrepresentableGeometry)?;
        let height = self
            .checked_height()
            .ok_or(PlanError::UnrepresentableGeometry)?;
        if width <= 0 || height <= 0 {
            return Err(PlanError::EmptyOrInvertedWorkArea);
        }
        Ok(())
    }

    /// True when `self` fully encloses `other`.
    pub fn contains(&self, other: &Rect) -> bool {
        other.left >= self.left
            && other.top >= self.top
            && other.right <= self.right
            && other.bottom <= self.bottom
    }
}

/// Monitor work area plus its DPI context.
/// `rect` is the *usable* area — taskbar and reserved appbars already excluded
/// by the caller. `dpi` is carried for traceability only: coordinates arrive
/// already in physical pixels from a Per-Monitor-V2-aware process, so planners
/// must never scale by it a second time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkArea {
    pub rect: Rect,
    pub dpi: u32,
}

impl WorkArea {
    pub fn new(rect: Rect, dpi: u32) -> Result<WorkArea, PlanError> {
        rect.validate()?;
        Ok(WorkArea { rect, dpi })
    }
}

/// Deterministic planning failure. No partial placement ever accompanies one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    EmptyOrInvertedWorkArea,
    UnrepresentableGeometry,
    InvalidWidthPercent(u32),
}

/// One window and where it should go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub window: WindowId,
    pub rect: Rect,
}

/// A complete plan. An **empty** plan is a successful no-op, not a failure —
/// stack-disabled and zero-candidate cases both land here.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlacementPlan {
    pub placements: Vec<Placement>,
}

impl PlacementPlan {
    pub fn empty() -> Self {
        PlacementPlan::default()
    }

    pub fn single(window: WindowId, rect: Rect) -> Self {
        PlacementPlan {
            placements: vec![Placement { window, rect }],
        }
    }

    pub fn is_noop(&self) -> bool {
        self.placements.is_empty()
    }

    /// Every placement lies inside `work`.
    pub fn fits_within(&self, work: &Rect) -> bool {
        self.placements.iter().all(|p| work.contains(&p.rect))
    }
}

pub type PlanResult = Result<PlacementPlan, PlanError>;

/// Applies a plan to the real desktop. Implemented by.
pub trait WindowMover {
    fn apply(&mut self, placement: &Placement) -> bool;
}

/// Maximum windows an overlapping stack may arrange.
pub const STACK_MAX_WINDOWS: usize = 3;

#[cfg(test)]
pub mod fixtures {
    use super::*;

    /// Primary 1920x1080 with a 40 px taskbar already excluded.
    pub fn primary_work_area() -> WorkArea {
        WorkArea::new(Rect::new(0, 0, 1920, 1040).unwrap(), 96).unwrap()
    }

    /// Secondary monitor at negative coordinates — a real multi-monitor layout.
    pub fn negative_origin_work_area() -> WorkArea {
        WorkArea::new(Rect::new(-1920, -200, 0, 880).unwrap(), 120).unwrap()
    }

    /// Odd width, to exercise the deterministic split.
    pub fn odd_width_work_area() -> WorkArea {
        WorkArea::new(Rect::new(0, 0, 1367, 769).unwrap(), 144).unwrap()
    }

    pub fn dpi_variants() -> Vec<WorkArea> {
        [96u32, 120, 144, 192]
            .into_iter()
            .map(|dpi| WorkArea::new(Rect::new(0, 0, 1920, 1040).unwrap(), dpi).unwrap())
            .collect()
    }

    pub fn windows(n: usize) -> Vec<WindowId> {
        (1..=n as isize).map(WindowId).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;
    use shared::config::{LayoutConfig, SnappingConfig};
    use shared::Command;

    // --- frozen command contract, verified from the consumer ----

    #[test]
    fn arrangement_commands_retain_frozen_wire_values() {
        assert_eq!(Command::SnapLeft.as_u8(), 2);
        assert_eq!(Command::SnapRight.as_u8(), 3);
        assert_eq!(Command::SnapMaximize.as_u8(), 4);
        assert_eq!(Command::OverlappingStack.as_u8(), 5);
    }

    #[test]
    fn unknown_command_values_still_decode_as_nop() {
        assert_eq!(Command::from_u8(6), Command::Nop);
        assert_eq!(Command::from_u8(200), Command::Nop);
    }

    // --- frozen shortcut defaults ------------------------------

    #[test]
    fn arrangement_shortcut_defaults_are_frozen() {
        let snapping = SnappingConfig::default();
        assert_eq!(snapping.snap_half_left, "ctrl+win+left");
        assert_eq!(snapping.snap_half_right, "ctrl+win+right");
        assert_eq!(snapping.snap_maximize, "ctrl+win+enter");
        assert_eq!(LayoutConfig::default().stack_shortcut, "ctrl+win+down");
    }

    #[test]
    fn stack_is_disabled_by_default_at_fifty_percent() {
        let layout = LayoutConfig::default();
        assert!(!layout.enable_overlapping_stack);
        assert_eq!(layout.stack_width_percent, 50);
    }

    // --- geometry invariants -----------------------------------

    #[test]
    fn valid_rect_reports_width_and_height() {
        let r = Rect::new(10, 20, 110, 220).unwrap();
        assert_eq!(r.width(), 100);
        assert_eq!(r.height(), 200);
    }

    #[test]
    fn negative_coordinates_are_supported() {
        let r = Rect::new(-1920, -200, 0, 880).unwrap();
        assert_eq!(r.width(), 1920);
        assert_eq!(r.height(), 1080);
    }

    #[test]
    fn empty_rect_is_rejected() {
        assert_eq!(
            Rect::new(0, 0, 0, 100),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
        assert_eq!(
            Rect::new(0, 0, 100, 0),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }

    #[test]
    fn inverted_rect_is_rejected() {
        assert_eq!(
            Rect::new(100, 0, 10, 100),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
        assert_eq!(
            Rect::new(0, 100, 100, 10),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }

    #[test]
    fn overflowing_extent_is_rejected_not_wrapped() {
        // right - left would overflow i32; checked arithmetic must catch it
        // rather than silently producing a negative width.
        assert_eq!(
            Rect::new(i32::MIN, 0, i32::MAX, 100),
            Err(PlanError::UnrepresentableGeometry)
        );
    }

    #[test]
    fn containment_is_inclusive_of_shared_edges() {
        let work = Rect::new(0, 0, 100, 100).unwrap();
        assert!(work.contains(&Rect::new(0, 0, 100, 100).unwrap()));
        assert!(work.contains(&Rect::new(0, 0, 50, 100).unwrap()));
        assert!(!work.contains(&Rect::new(0, 0, 101, 100).unwrap()));
        assert!(!work.contains(&Rect::new(-1, 0, 50, 100).unwrap()));
    }

    #[test]
    fn work_area_rejects_invalid_geometry() {
        let inverted = Rect {
            left: 100,
            top: 0,
            right: 0,
            bottom: 100,
        };
        assert_eq!(
            WorkArea::new(inverted, 96),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }

    // --- no inter-monitor command ------------------------------

    #[test]
    fn command_set_contains_no_inter_monitor_arrangement() {
        // Movement between monitors stays delegated to native Win+Shift+Arrow.
        // The frozen u8 range is 0..=5 and every value is accounted for, so an
        // inter-monitor command cannot exist without breaking the freeze.
        for raw in 0u8..=5 {
            let cmd = Command::from_u8(raw);
            assert!(
                matches!(
                    cmd,
                    Command::Nop
                        | Command::Cycle
                        | Command::SnapLeft
                        | Command::SnapRight
                        | Command::SnapMaximize
                        | Command::OverlappingStack
                ),
                "unexpected command at wire value {raw}"
            );
        }
    }

    // --- Plan semantics -----------------------------------------------------

    #[test]
    fn empty_plan_is_a_successful_noop() {
        let plan = PlacementPlan::empty();
        assert!(plan.is_noop());
        assert!(plan.fits_within(&primary_work_area().rect));
    }

    #[test]
    fn fits_within_detects_an_escaping_placement() {
        let work = primary_work_area();
        let plan = PlacementPlan::single(WindowId(1), Rect::new(0, 0, 5000, 100).unwrap());
        assert!(!plan.fits_within(&work.rect));
    }

    #[test]
    fn dpi_is_carried_but_never_applied() {
        // Same pixel geometry at every DPI: the planner must not rescale.
        for work in dpi_variants() {
            assert_eq!(work.rect.width(), 1920);
            assert_eq!(work.rect.height(), 1040);
        }
    }
}
