//! Same-App Visual Switcher domain module.

#![allow(dead_code)]

pub mod layout;
pub mod overlay;
pub mod selection;
pub mod thumbnail;

use crate::cycling::{ActiveContext, Candidate, WindowId};
use crate::switcher::layout::Rect;
use crate::switcher::overlay::SwitcherOverlay;
use crate::switcher::thumbnail::DwmThumbnailSink;
use shared::Shortcut;
use std::sync::{Arc, Mutex};

/// State of the visual switcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitcherState {
    Inactive,
    Armed {
        chord: Shortcut,
        deadline_ms: u64,
    },
    Active {
        candidates: Vec<WindowId>,
        selected: usize,
        page: usize,
        origin_foreground: WindowId,
    },
}

/// Worker-side controller for visual switcher state and overlay lifecycle.
pub struct SwitcherController {
    overlay: SwitcherOverlay,
    origin_foreground: WindowId,
    candidates: Vec<WindowId>,
    selected_index: usize,
    open_time_ms: u64,
}

impl Default for SwitcherController {
    fn default() -> Self {
        Self::new()
    }
}

impl SwitcherController {
    pub fn new() -> Self {
        Self {
            overlay: SwitcherOverlay::new(Arc::new(Mutex::new(DwmThumbnailSink))),
            origin_foreground: WindowId(0),
            candidates: Vec::new(),
            selected_index: 0,
            open_time_ms: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.overlay.is_visible()
    }

    pub fn open_time_ms(&self) -> u64 {
        self.open_time_ms
    }

    pub fn open(
        &mut self,
        origin: WindowId,
        work_area: Rect,
        candidates: Vec<WindowId>,
        selected_index: usize,
    ) {
        self.origin_foreground = origin;
        self.candidates = candidates;
        self.selected_index = selected_index;
        self.open_time_ms = crate::hook::tick_ms();
        self.overlay
            .show(work_area, self.candidates.clone(), selected_index);
    }

    pub fn next(&mut self) {
        if self.candidates.is_empty() {
            return;
        }
        let (idx, _) = selection::select_next(
            self.candidates.len(),
            self.selected_index,
            self.overlay.per_page(),
        );
        self.selected_index = idx;
        self.overlay.set_selected_index(idx);
    }

    pub fn prev(&mut self) {
        if self.candidates.is_empty() {
            return;
        }
        let (idx, _) = selection::select_prev(
            self.candidates.len(),
            self.selected_index,
            self.overlay.per_page(),
        );
        self.selected_index = idx;
        self.overlay.set_selected_index(idx);
    }

    pub fn up(&mut self) {
        if self.candidates.is_empty() {
            return;
        }
        let (idx, _) = selection::select_up(
            self.candidates.len(),
            self.selected_index,
            self.overlay.cols(),
            self.overlay.per_page(),
        );
        self.selected_index = idx;
        self.overlay.set_selected_index(idx);
    }

    pub fn down(&mut self) {
        if self.candidates.is_empty() {
            return;
        }
        let (idx, _) = selection::select_down(
            self.candidates.len(),
            self.selected_index,
            self.overlay.cols(),
            self.overlay.per_page(),
        );
        self.selected_index = idx;
        self.overlay.set_selected_index(idx);
    }

    pub fn selected_window(&self) -> Option<WindowId> {
        self.candidates.get(self.selected_index).copied()
    }

    /// Returns candidate targets starting from `selected_index` for fallback activation.
    pub fn candidates_from_selection(&self) -> Vec<WindowId> {
        if self.candidates.is_empty() {
            return Vec::new();
        }
        let mut res = Vec::with_capacity(self.candidates.len());
        for i in self.selected_index..self.candidates.len() {
            res.push(self.candidates[i]);
        }
        for i in 0..self.selected_index {
            res.push(self.candidates[i]);
        }
        res
    }

    pub fn origin_window(&self) -> WindowId {
        self.origin_foreground
    }

    pub fn dismiss(&mut self) {
        self.overlay.dismiss();
        self.candidates.clear();
        self.selected_index = 0;
        self.open_time_ms = 0;
    }
}

/// Returns the card order for candidates, matching `cycle_order`.
pub fn card_order_for_candidates(
    candidates: &[Candidate],
    active: &ActiveContext,
) -> Vec<WindowId> {
    crate::cycling::cycle_order(candidates, active)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::context::fixtures::{FakeDesktops, FakeMonitors, MONITOR_A, MONITOR_B};
    use crate::cycling::eligibility::WindowEligibility;
    use crate::cycling::fixtures::*;
    use crate::cycling::{cycle_order, WindowId};
    use crate::worker::collect_eligible_candidates;

    #[test]
    fn switcher_candidate_set_equals_blind_cycle_eligible_set() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3), normal(4)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_A)),
            (WindowId(3), Some(MONITOR_B)),
            (WindowId(4), Some(MONITOR_A)),
        ]);
        let desktops = FakeDesktops(vec![
            (WindowId(1), Some(true)),
            (WindowId(2), Some(false)),
            (WindowId(3), Some(true)),
            (WindowId(4), Some(true)),
        ]);
        let spatial = crate::context::SpatialContext {
            origin_monitor: Some(MONITOR_A),
        };
        let active = active(1);

        let (blind_candidates, blind_eligible) = collect_eligible_candidates(
            &StaticSource(candidates.clone()),
            &WindowEligibility,
            &active,
            &monitors,
            Some(&desktops),
            &spatial,
        );

        let (switcher_candidates, switcher_eligible) = collect_eligible_candidates(
            &StaticSource(candidates),
            &WindowEligibility,
            &active,
            &monitors,
            Some(&desktops),
            &spatial,
        );

        assert_eq!(blind_candidates, switcher_candidates);
        assert_eq!(blind_eligible, switcher_eligible);
    }

    #[test]
    fn card_order_equals_cycle_order() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3), normal(4)]);
        let active = active(2);

        let order = cycle_order(&candidates, &active);
        let card_order = card_order_for_candidates(&candidates, &active);
        assert_eq!(card_order, order);
    }
}
