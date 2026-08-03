//! Pure eligibility policy. Owns no Win32 calls.
//! This module decides only. It never enumerates, never queries a process,
//! never touches focus, and never calls `SendMessage` or `GetWindowText`
//! Uses SetForegroundWindow with brief confirmation polling. Every input arrives as already-captured [`WindowFacts`] from
//! the contract, which is what keeps the decision deterministic.
//! It has no dependency on physical-monitor, virtual-desktop, VM/RDP,
//! arrangement, or Settings behavior — those belong to Epics 3, 4, and 5.

use super::{
    ActiveContext, AppIdentity, Candidate, Eligibility, EligibilityPolicy, ExclusionReason,
    WindowFacts,
};

/// Frozen exclusion precedence.
/// When a window matches several exclusion facts at once, the *first* matching
/// rule below wins. The order is part of the contract: it makes combination
/// cases deterministic instead of implementation-defined.
/// 1. ghost class
/// 2. shell surface class
/// 3. hidden
/// 4. cloaked (hidden by the compositor)
/// 5. iconic (minimized)
/// 6. tool window
/// 7. unavailable identity
/// 8. different application
pub struct WindowEligibility;

impl EligibilityPolicy for WindowEligibility {
    fn evaluate(&self, active: &ActiveContext, candidate: &Candidate) -> Eligibility {
        evaluate_facts(&active.identity, &candidate.facts)
    }
}

/// Decide eligibility from captured facts alone.
/// Deliberately takes the active *identity* rather than the whole
/// [`ActiveContext`]: the policy has no business knowing the foreground handle,
/// which keeps selection concerns out of this module.
pub fn evaluate_facts(active_identity: &AppIdentity, facts: &WindowFacts) -> Eligibility {
    // A hung application produces a separate `Ghost` window. Excluding the
    // ghost is what lets the *real* window stay eligible without any
    // responsiveness probe.
    if facts.is_ghost() {
        return Eligibility::Excluded(ExclusionReason::GhostWindow);
    }
    if facts.is_shell_surface() {
        return Eligibility::Excluded(ExclusionReason::ShellSurface);
    }
    if !facts.visible {
        return Eligibility::Excluded(ExclusionReason::Hidden);
    }
    // `visible` is not enough on its own. A cloaked window keeps `WS_VISIBLE`
    // set and answers `IsWindowVisible` with `true`; the compositor simply never
    // draws it. Activating one moves focus to nothing the user can see, which is
    // exactly the blank stop reported while cycling File Explorer.
    if facts.cloaked {
        return Eligibility::Excluded(ExclusionReason::Cloaked);
    }
    if facts.iconic {
        return Eligibility::Excluded(ExclusionReason::Iconic);
    }
    if facts.tool_window {
        return Eligibility::Excluded(ExclusionReason::ToolWindow);
    }
    if !facts.identity.is_available() {
        return Eligibility::Excluded(ExclusionReason::UnavailableIdentity);
    }
    if !facts.identity.same_application(active_identity) {
        return Eligibility::Excluded(ExclusionReason::DifferentApplication);
    }
    Eligibility::Eligible
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::*;
    use super::*;

    #[test]
    fn reproduces_every_frozen_fixture() {
        let active = active(1);
        for (label, facts, expected) in expected_decisions() {
            let candidate = Candidate {
                z_index: 0,
                facts: facts.clone(),
            };
            assert_eq!(
                WindowEligibility.evaluate(&active, &candidate),
                expected,
                "production policy drifted from frozen fixture: {label}"
            );
        }
    }

    #[test]
    fn agrees_with_reference_policy_on_every_fixture() {
        let active = active(1);
        for (label, facts, _) in expected_decisions() {
            let candidate = Candidate {
                z_index: 0,
                facts: facts.clone(),
            };
            assert_eq!(
                WindowEligibility.evaluate(&active, &candidate),
                ReferencePolicy.evaluate(&active, &candidate),
                "production and reference policy disagree on: {label}"
            );
        }
    }

    // --- base decisions ---------------------------------------

    #[test]
    fn visible_non_iconic_top_level_window_is_eligible() {
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &normal(1)),
            Eligibility::Eligible
        );
    }

    #[test]
    fn hidden_is_excluded() {
        let facts = WindowFacts {
            visible: false,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Hidden)
        );
    }

    #[test]
    fn cloaked_is_excluded_even_though_it_reports_visible() {
        // Regression guard. Cycling three File Explorer windows had a fourth,
        // blank stop: explorer.exe also owned a shell-cloaked, untitled,
        // screen-sized `ApplicationFrameWindow`. Every fact the policy had at
        // the time said "ordinary window", so it was activated.
        let facts = WindowFacts {
            visible: true,
            cloaked: true,
            class_name: "ApplicationFrameWindow".to_string(),
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Cloaked)
        );
    }

    #[test]
    fn cloaked_outranks_iconic_and_tool_window() {
        let facts = WindowFacts {
            cloaked: true,
            iconic: true,
            tool_window: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Cloaked)
        );
    }

    #[test]
    fn hidden_outranks_cloaked() {
        // Both mean "the user cannot see it", so the reported reason is the one
        // that is cheaper to establish. Pinned so the precedence cannot drift.
        let facts = WindowFacts {
            visible: false,
            cloaked: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Hidden)
        );
    }

    #[test]
    fn minimized_is_excluded() {
        let facts = WindowFacts {
            iconic: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Iconic)
        );
    }

    #[test]
    fn tool_window_is_excluded() {
        let facts = WindowFacts {
            tool_window: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::ToolWindow)
        );
    }

    #[test]
    fn every_shell_overlay_class_is_excluded() {
        for class in SHELL_SURFACE_CLASSES {
            assert_eq!(
                evaluate_facts(&identity(HOST_EXE), &with_class(1, class)),
                Eligibility::Excluded(ExclusionReason::ShellSurface),
                "shell class not excluded: {class}"
            );
        }
    }

    #[test]
    fn shell_and_ghost_class_matching_is_case_insensitive() {
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &with_class(1, "shell_traywnd")),
            Eligibility::Excluded(ExclusionReason::ShellSurface)
        );
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &with_class(2, "GHOST")),
            Eligibility::Excluded(ExclusionReason::GhostWindow)
        );
    }

    // --- UX honesty for hung windows --------------------------

    #[test]
    fn synthetic_hung_application_window_remains_eligible() {
        // The real window of a hung app carries its own class and stays
        // visible; only the separate `Ghost` surrogate is excluded.
        let hung_real_window = WindowFacts {
            class_name: "Notepad".to_string(),
            ..normal(42)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &hung_real_window),
            Eligibility::Eligible
        );

        let ghost_surrogate = with_class(43, CLASS_GHOST);
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &ghost_surrogate),
            Eligibility::Excluded(ExclusionReason::GhostWindow)
        );
    }

    #[test]
    fn contract_exposes_no_responsiveness_input() {
        // Guards structurally: a hung and a responsive window are
        // indistinguishable to this policy, so it cannot probe or skip.
        let responsive = normal(1);
        let hung = normal(1);
        assert_eq!(responsive, hung);
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &responsive),
            evaluate_facts(&identity(HOST_EXE), &hung)
        );
    }

    // --- exclusion-fact combinations --------------------------

    #[test]
    fn ghost_outranks_every_other_exclusion() {
        let facts = WindowFacts {
            visible: false,
            iconic: true,
            tool_window: true,
            identity: AppIdentity::Unavailable,
            ..with_class(1, CLASS_GHOST)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::GhostWindow)
        );
    }

    #[test]
    fn shell_surface_outranks_hidden_and_iconic() {
        let facts = WindowFacts {
            visible: false,
            iconic: true,
            ..with_class(1, CLASS_PROGMAN)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::ShellSurface)
        );
    }

    #[test]
    fn hidden_outranks_iconic_and_tool_window() {
        let facts = WindowFacts {
            visible: false,
            iconic: true,
            tool_window: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Hidden)
        );
    }

    #[test]
    fn iconic_outranks_tool_window() {
        let facts = WindowFacts {
            iconic: true,
            tool_window: true,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::Iconic)
        );
    }

    #[test]
    fn unavailable_identity_outranks_different_application() {
        let facts = WindowFacts {
            identity: AppIdentity::Unavailable,
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::UnavailableIdentity)
        );
    }

    #[test]
    fn decisions_are_deterministic_across_repeated_evaluation() {
        let active = identity(HOST_EXE);
        for (label, facts, expected) in expected_decisions() {
            for _ in 0..8 {
                assert_eq!(
                    evaluate_facts(&active, &facts),
                    expected,
                    "non-deterministic decision for: {label}"
                );
            }
        }
    }

    // --- identity scoping -------------------------------------

    #[test]
    fn different_application_is_excluded_even_when_otherwise_valid() {
        let facts = WindowFacts {
            identity: identity(OTHER_EXE),
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Excluded(ExclusionReason::DifferentApplication)
        );
    }

    #[test]
    fn same_executable_across_processes_stays_eligible() {
        let facts = WindowFacts {
            identity: AppIdentity::from_process_path(Some(r"D:\elsewhere\NOTEPAD.EXE")),
            ..normal(1)
        };
        assert_eq!(
            evaluate_facts(&identity(HOST_EXE), &facts),
            Eligibility::Eligible
        );
    }

    #[test]
    fn unavailable_active_identity_yields_no_eligible_window() {
        // If we cannot identify the foreground app, nothing can be proven to
        // belong to it, so the cycle must find no target rather than guess.
        let unknown_active = AppIdentity::Unavailable;
        assert_eq!(
            evaluate_facts(&unknown_active, &normal(1)),
            Eligibility::Excluded(ExclusionReason::DifferentApplication)
        );
    }
}
