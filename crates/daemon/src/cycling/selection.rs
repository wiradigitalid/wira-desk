//! Stateless next-target selection. Owns no Win32 calls.
//! Selection is a pure function of the snapshot it is handed. It keeps no
//! cursor between commands, so a window closing or opening between shortcuts
//! can never desynchronize it.

use super::{cycle_order, ActiveContext, Candidate, SelectionResult, WindowId};

/// Build the full attempt order: every eligible window except the active one,
/// starting immediately after the active window and wrapping **at most once**.
/// The active window is filtered out rather than skipped mid-loop, which is
/// what bounds the pass — the sequence itself can never revisit it.
pub fn attempt_order(
    candidates: &[Candidate],
    eligible: &[WindowId],
    active: &ActiveContext,
) -> Vec<WindowId> {
    cycle_order(candidates, active)
        .into_iter()
        .filter(|w| *w != active.foreground && eligible.contains(w))
        .collect()
}

/// Choose the next target, or [`SelectionResult::NoCandidate`] for a no-op.
pub fn select_next(
    candidates: &[Candidate],
    eligible: &[WindowId],
    active: &ActiveContext,
) -> SelectionResult {
    match attempt_order(candidates, eligible, active).first() {
        Some(target) => SelectionResult::Target(*target),
        None => SelectionResult::NoCandidate,
    }
}

/// Advance past targets already tried in this pass.
/// Used by the driver after an activation failure so each candidate is
/// attempted no more than once.
pub fn select_after(
    candidates: &[Candidate],
    eligible: &[WindowId],
    active: &ActiveContext,
    tried: &[WindowId],
) -> SelectionResult {
    match attempt_order(candidates, eligible, active)
        .into_iter()
        .find(|w| !tried.contains(w))
    {
        Some(target) => SelectionResult::Target(target),
        None => SelectionResult::NoCandidate,
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::*;
    use super::*;

    fn all_windows(candidates: &[Candidate]) -> Vec<WindowId> {
        candidates.iter().map(|c| c.facts.window).collect()
    }

    // --- next, wrap, no-op ------------------------------------

    #[test]
    fn selects_the_least_recently_used_window() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        assert_eq!(
            select_next(&candidates, &eligible, &active(1)),
            SelectionResult::Target(WindowId(3))
        );
    }

    #[test]
    fn wraps_to_beginning_from_last_window() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        assert_eq!(
            select_next(&candidates, &eligible, &active(3)),
            SelectionResult::Target(WindowId(2))
        );
    }

    #[test]
    fn wraps_at_most_once() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        let order = attempt_order(&candidates, &eligible, &active(2));
        assert_eq!(order, vec![WindowId(1), WindowId(3)]);
        // Two other windows, two entries — no repetition from a second lap.
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn single_window_sequence_is_a_no_op() {
        let candidates = ordered(vec![normal(1)]);
        let eligible = all_windows(&candidates);
        assert_eq!(
            select_next(&candidates, &eligible, &active(1)),
            SelectionResult::NoCandidate
        );
    }

    #[test]
    fn empty_eligible_set_is_a_no_op() {
        let candidates = ordered(vec![normal(1), normal(2)]);
        assert_eq!(
            select_next(&candidates, &[], &active(1)),
            SelectionResult::NoCandidate
        );
    }

    #[test]
    fn active_window_is_never_a_target() {
        let candidates = ordered(vec![normal(1), normal(2)]);
        let eligible = all_windows(&candidates);
        let order = attempt_order(&candidates, &eligible, &active(1));
        assert!(!order.contains(&WindowId(1)));
    }

    #[test]
    fn ineligible_windows_are_not_targets() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        // Only window 3 is eligible.
        let eligible = vec![WindowId(3)];
        assert_eq!(
            select_next(&candidates, &eligible, &active(1)),
            SelectionResult::Target(WindowId(3))
        );
    }

    // --- active window absent ---------------------------------

    #[test]
    fn active_absent_selects_first_eligible_deterministically() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        for _ in 0..8 {
            assert_eq!(
                select_next(&candidates, &eligible, &active(99)),
                SelectionResult::Target(WindowId(3))
            );
        }
    }

    #[test]
    fn active_absent_order_covers_every_eligible_window_once() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        let order = attempt_order(&candidates, &eligible, &active(99));
        assert_eq!(order, vec![WindowId(3), WindowId(2), WindowId(1)]);
    }

    #[test]
    fn selection_keeps_no_cursor_between_calls() {
        // Same inputs must yield the same answer; there is no cached position.
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        let first = select_next(&candidates, &eligible, &active(1));
        let second = select_next(&candidates, &eligible, &active(1));
        assert_eq!(first, second);
    }

    // --- failure continuation ---------------------------------

    #[test]
    fn advances_past_a_tried_target() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        assert_eq!(
            select_after(&candidates, &eligible, &active(1), &[WindowId(3)]),
            SelectionResult::Target(WindowId(2))
        );
    }

    #[test]
    fn exhaustion_is_a_silent_no_candidate() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let eligible = all_windows(&candidates);
        assert_eq!(
            select_after(
                &candidates,
                &eligible,
                &active(1),
                &[WindowId(2), WindowId(3)]
            ),
            SelectionResult::NoCandidate
        );
    }

    #[test]
    fn every_candidate_attempted_at_most_once() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3), normal(4)]);
        let eligible = all_windows(&candidates);
        let mut tried: Vec<WindowId> = Vec::new();
        while let SelectionResult::Target(t) =
            select_after(&candidates, &eligible, &active(1), &tried)
        {
            assert!(!tried.contains(&t), "candidate re-attempted: {t:?}");
            tried.push(t);
            assert!(tried.len() <= 3, "pass exceeded the eligible set");
        }
        assert_eq!(tried, vec![WindowId(4), WindowId(3), WindowId(2)]);
    }

    // --- fake-activator and closing-window harness -------------

    #[test]
    fn closing_window_mid_cycle_falls_through_to_next() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let source = StaticSource(candidates);
        let mut activator = ScriptedActivator::scripted(vec![
            (WindowId(3), ActivationOutcome::InvalidTarget),
            (WindowId(2), ActivationOutcome::Activated),
        ]);
        let outcome = run_cycle(&source, &ReferencePolicy, &mut activator, &active(1));
        assert_eq!(outcome, CycleOutcome::Activated(WindowId(2)));
        assert_eq!(activator.attempts, vec![WindowId(3), WindowId(2)]);
    }

    #[test]
    fn all_targets_closing_ends_silently() {
        let source = StaticSource(ordered(vec![normal(1), normal(2), normal(3)]));
        let mut activator = ScriptedActivator::always(ActivationOutcome::InvalidTarget);
        let outcome = run_cycle(&source, &ReferencePolicy, &mut activator, &active(1));
        assert_eq!(outcome, CycleOutcome::Exhausted);
        assert_eq!(activator.attempts.len(), 2);
    }

    #[test]
    fn hung_target_receives_the_same_single_attempt() {
        // A hung window is indistinguishable here, so it gets exactly the one
        // bounded attempt a responsive window would get — no probe, no wait.
        let source = StaticSource(ordered(vec![normal(1), normal(2)]));
        let mut activator = ScriptedActivator::always(ActivationOutcome::Activated);
        let outcome = run_cycle(&source, &ReferencePolicy, &mut activator, &active(1));
        assert_eq!(outcome, CycleOutcome::Activated(WindowId(2)));
        assert_eq!(activator.attempts, vec![WindowId(2)]);
    }
}
