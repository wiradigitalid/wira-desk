//! Frozen context-safety contract — Worker/Hook policy vocabulary.
//!
//! Two independent decisions live here, deliberately kept apart because they
//! fail in **opposite directions**:
//!
//! - **Spatial eligibility** fails *closed*. If we cannot prove a candidate is
//!   on the current monitor and virtual desktop, it is not a target. Guessing
//!   would throw focus across the user's workspace.
//! - **Foreground bypass** fails *open*. If we cannot identify the foreground
//!   application, the key passes through untouched. Guessing would swallow a
//!   keystroke inside someone's VM or RDP session.
//!
//! Neither decision introduces a Z-order cache, cross-actor shared mutable
//! state, or a second command-payload path.
//!
//! Module ownership after this story:
//! - `context/spatial.rs`, `context/virtual_desktop.rs` — spatial and desktop facts
//! - `context/vm_bypass.rs` — foreground bypass classification
//! - `context/mod.rs`, `worker.rs`, `hook.rs` — policy composition

// Published ahead of its consumers, same as the cycling contract.
#![allow(dead_code)]

pub mod spatial;
pub mod virtual_desktop;
pub mod vm_bypass;

use shared::config::VmBypassConfig;

use crate::cycling::WindowId;

/// Opaque physical-monitor handle (`HMONITOR`), kept as a plain integer so
/// fixtures need no Win32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonitorId(pub isize);

// ── Spatial eligibility ─────────────────────────────────────────────────────

/// Origin facts resolved **once** per cycle operation.
/// `None` means the lookup failed. It is not "no monitor" — it is "unknown",
/// and the contract treats unknown as ineligible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialContext {
    pub origin_monitor: Option<MonitorId>,
}

/// Per-candidate spatial facts, captured live for the current command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialFacts {
    pub candidate_monitor: Option<MonitorId>,
    /// `None` when the virtual-desktop query failed or COM was unavailable.
    pub on_current_virtual_desktop: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialRejection {
    DifferentMonitor,
    NotOnCurrentVirtualDesktop,
    MonitorUnavailable,
    VirtualDesktopUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialDecision {
    Eligible,
    Ineligible(SpatialRejection),
}

impl SpatialDecision {
    pub fn is_eligible(&self) -> bool {
        matches!(self, SpatialDecision::Eligible)
    }
}

/// Decide spatial eligibility. **Fails closed on any uncertainty.**
pub fn evaluate_spatial(ctx: &SpatialContext, facts: &SpatialFacts) -> SpatialDecision {
    let Some(origin) = ctx.origin_monitor else {
        return SpatialDecision::Ineligible(SpatialRejection::MonitorUnavailable);
    };
    let Some(candidate) = facts.candidate_monitor else {
        return SpatialDecision::Ineligible(SpatialRejection::MonitorUnavailable);
    };
    if origin != candidate {
        return SpatialDecision::Ineligible(SpatialRejection::DifferentMonitor);
    }
    match facts.on_current_virtual_desktop {
        None => SpatialDecision::Ineligible(SpatialRejection::VirtualDesktopUnavailable),
        Some(false) => SpatialDecision::Ineligible(SpatialRejection::NotOnCurrentVirtualDesktop),
        Some(true) => SpatialDecision::Eligible,
    }
}

// ── Foreground bypass classification ────────────────────────────────────────

/// What we know about the foreground window when a shortcut arrives.
/// Either field may be `None` when its lookup failed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ForegroundIdentity {
    /// Executable basename, not normalized — the policy compares
    /// case-insensitively so no allocation is needed at evaluation time.
    pub process: Option<String>,
    pub class: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BypassReason {
    ProcessMatch,
    ClassMatch,
    /// Foreground could not be identified well enough to confirm a non-match.
    IdentityUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BypassDecision {
    /// Let the physical key combination through untouched.
    Passthrough(BypassReason),
    /// Confirmed non-match — Wira Desk may handle the shortcut.
    ContinueWiraDeskMatching,
}

impl BypassDecision {
    pub fn is_passthrough(&self) -> bool {
        matches!(self, BypassDecision::Passthrough(_))
    }
}

/// Immutable, pre-normalized bypass policy owned by the Hook Thread.
/// Built **outside** the callback via [`BypassPolicy::from_config`], which is
/// where every allocation and lowercase conversion happens. [`Self::classify`]
/// then performs no parsing, allocation, file I/O, logging, or lock
/// acquisition, which is what keeps the callback bounded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BypassPolicy {
    processes: Vec<String>,
    classes: Vec<String>,
}

impl BypassPolicy {
    /// Normalize configuration into runtime policy data. Call once, off the
    /// callback path.
    pub fn from_config(cfg: &VmBypassConfig) -> Self {
        BypassPolicy {
            processes: normalize_identifiers(&cfg.bypass_processes),
            classes: normalize_identifiers(&cfg.bypass_classes),
        }
    }

    pub fn processes(&self) -> &[String] {
        &self.processes
    }

    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    /// Classify the foreground. **Fails open on uncertainty.**
    /// `ContinueWiraDeskMatching` is returned only when both identifiers are
    /// known and neither matches — a *confirmed* non-match. The name is the
    /// vocabulary; the contract was corrected here, in its owning
    /// story, rather than aliased downstream.
    /// If either lookup failed we cannot
    /// confirm anything, so the key passes through rather than risking a
    /// swallowed keystroke inside a guest session.
    pub fn classify(&self, identity: &ForegroundIdentity) -> BypassDecision {
        if let Some(process) = identity.process.as_deref() {
            if contains_ignore_case(&self.processes, process) {
                return BypassDecision::Passthrough(BypassReason::ProcessMatch);
            }
        }
        if let Some(class) = identity.class.as_deref() {
            if contains_ignore_case(&self.classes, class) {
                return BypassDecision::Passthrough(BypassReason::ClassMatch);
            }
        }
        if identity.process.is_none() || identity.class.is_none() {
            return BypassDecision::Passthrough(BypassReason::IdentityUnavailable);
        }
        BypassDecision::ContinueWiraDeskMatching
    }
}

fn normalize_identifiers(raw: &[String]) -> Vec<String> {
    raw.iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Allocation-free membership test against pre-normalized entries.
fn contains_ignore_case(haystack: &[String], needle: &str) -> bool {
    haystack.iter().any(|e| e.eq_ignore_ascii_case(needle))
}

// ── Lane seams (implemented by Stories 3.2 and 3.3) ─────────────────────────

/// Resolves a window's physical monitor..
pub trait MonitorSource {
    fn monitor_of(&self, window: WindowId) -> Option<MonitorId>;
}

/// Answers virtual-desktop membership..
pub trait VirtualDesktopSource {
    fn is_on_current_desktop(&self, window: WindowId) -> Option<bool>;
}

/// Supplies foreground process/class identity..
pub trait ForegroundIdentitySource {
    fn foreground_identity(&self) -> ForegroundIdentity;
}

/// Gather spatial facts for one candidate from the two lane adapters.
pub fn collect_spatial_facts<M, V>(monitors: &M, desktops: &V, candidate: WindowId) -> SpatialFacts
where
    M: MonitorSource + ?Sized,
    V: VirtualDesktopSource + ?Sized,
{
    SpatialFacts {
        candidate_monitor: monitors.monitor_of(candidate),
        on_current_virtual_desktop: desktops.is_on_current_desktop(candidate),
    }
}

/// Resolve the origin monitor once for a cycle operation.
pub fn capture_spatial_context<M>(monitors: &M, origin: WindowId) -> SpatialContext
where
    M: MonitorSource + ?Sized,
{
    SpatialContext {
        origin_monitor: monitors.monitor_of(origin),
    }
}

#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub const MONITOR_A: MonitorId = MonitorId(1);
    pub const MONITOR_B: MonitorId = MonitorId(2);

    /// Deterministic monitor adapter. `None` entries model a failed lookup.
    pub struct FakeMonitors(pub Vec<(WindowId, Option<MonitorId>)>);

    impl MonitorSource for FakeMonitors {
        fn monitor_of(&self, window: WindowId) -> Option<MonitorId> {
            self.0
                .iter()
                .find(|(w, _)| *w == window)
                .and_then(|(_, m)| *m)
        }
    }

    /// Deterministic virtual-desktop adapter.
    pub struct FakeDesktops(pub Vec<(WindowId, Option<bool>)>);

    impl VirtualDesktopSource for FakeDesktops {
        fn is_on_current_desktop(&self, window: WindowId) -> Option<bool> {
            self.0
                .iter()
                .find(|(w, _)| *w == window)
                .and_then(|(_, v)| *v)
        }
    }

    pub struct FakeForeground(pub ForegroundIdentity);

    impl ForegroundIdentitySource for FakeForeground {
        fn foreground_identity(&self) -> ForegroundIdentity {
            self.0.clone()
        }
    }

    pub fn identity(process: Option<&str>, class: Option<&str>) -> ForegroundIdentity {
        ForegroundIdentity {
            process: process.map(str::to_string),
            class: class.map(str::to_string),
        }
    }

    pub fn default_policy() -> BypassPolicy {
        BypassPolicy::from_config(&VmBypassConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    // --- spatial uncertainty fails closed ----------------------

    fn ctx(monitor: Option<MonitorId>) -> SpatialContext {
        SpatialContext {
            origin_monitor: monitor,
        }
    }

    fn facts(monitor: Option<MonitorId>, on_desktop: Option<bool>) -> SpatialFacts {
        SpatialFacts {
            candidate_monitor: monitor,
            on_current_virtual_desktop: on_desktop,
        }
    }

    #[test]
    fn same_monitor_and_current_desktop_is_eligible() {
        assert_eq!(
            evaluate_spatial(&ctx(Some(MONITOR_A)), &facts(Some(MONITOR_A), Some(true))),
            SpatialDecision::Eligible
        );
    }

    #[test]
    fn different_monitor_is_rejected() {
        assert_eq!(
            evaluate_spatial(&ctx(Some(MONITOR_A)), &facts(Some(MONITOR_B), Some(true))),
            SpatialDecision::Ineligible(SpatialRejection::DifferentMonitor)
        );
    }

    #[test]
    fn non_current_virtual_desktop_is_rejected() {
        assert_eq!(
            evaluate_spatial(&ctx(Some(MONITOR_A)), &facts(Some(MONITOR_A), Some(false))),
            SpatialDecision::Ineligible(SpatialRejection::NotOnCurrentVirtualDesktop)
        );
    }

    #[test]
    fn unknown_origin_monitor_fails_closed() {
        assert_eq!(
            evaluate_spatial(&ctx(None), &facts(Some(MONITOR_A), Some(true))),
            SpatialDecision::Ineligible(SpatialRejection::MonitorUnavailable)
        );
    }

    #[test]
    fn unknown_candidate_monitor_fails_closed() {
        assert_eq!(
            evaluate_spatial(&ctx(Some(MONITOR_A)), &facts(None, Some(true))),
            SpatialDecision::Ineligible(SpatialRejection::MonitorUnavailable)
        );
    }

    #[test]
    fn unknown_virtual_desktop_fails_closed() {
        assert_eq!(
            evaluate_spatial(&ctx(Some(MONITOR_A)), &facts(Some(MONITOR_A), None)),
            SpatialDecision::Ineligible(SpatialRejection::VirtualDesktopUnavailable)
        );
    }

    #[test]
    fn no_uncertainty_combination_is_ever_eligible() {
        // Exhaustive guard: eligibility requires every fact to be known.
        for origin in [None, Some(MONITOR_A)] {
            for candidate in [None, Some(MONITOR_A)] {
                for desktop in [None, Some(false), Some(true)] {
                    let decision = evaluate_spatial(&ctx(origin), &facts(candidate, desktop));
                    let fully_known = origin == Some(MONITOR_A)
                        && candidate == Some(MONITOR_A)
                        && desktop == Some(true);
                    assert_eq!(
                        decision.is_eligible(),
                        fully_known,
                        "origin={origin:?} candidate={candidate:?} desktop={desktop:?}"
                    );
                }
            }
        }
    }

    // --- bypass uncertainty fails open -------------------------

    #[test]
    fn known_bypass_process_passes_through() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("mstsc.exe"), Some("TscShellContainerClass"))),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    #[test]
    fn process_match_is_case_insensitive() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("MSTSC.EXE"), Some("Whatever"))),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    #[test]
    fn known_bypass_class_passes_through() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("unknown.exe"), Some("VMwareUnityWindow"))),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
    }

    #[test]
    fn class_match_is_case_insensitive() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("unknown.exe"), Some("vmwareunitywindow"))),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
    }

    #[test]
    fn confirmed_non_match_is_intercepted() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("notepad.exe"), Some("Notepad"))),
            BypassDecision::ContinueWiraDeskMatching
        );
    }

    #[test]
    fn unknown_process_cannot_confirm_a_non_match() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(None, Some("Notepad"))),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn unknown_class_cannot_confirm_a_non_match() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("notepad.exe"), None)),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn fully_unknown_identity_passes_through() {
        let policy = default_policy();
        assert_eq!(
            policy.classify(&ForegroundIdentity::default()),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn a_match_still_wins_when_the_other_identifier_is_unknown() {
        // Uncertainty must not mask a positive match, or a VM window whose
        // class lookup failed would stop bypassing.
        let policy = default_policy();
        assert_eq!(
            policy.classify(&identity(Some("mstsc.exe"), None)),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
        assert_eq!(
            policy.classify(&identity(None, Some("VMwareUnityWindow"))),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
    }

    // --- schema and normalization -----------------

    #[test]
    fn policy_normalizes_at_construction_not_evaluation() {
        let cfg = VmBypassConfig {
            bypass_processes: vec!["  MSTSC.EXE  ".to_string(), String::new()],
            bypass_classes: vec!["  VMwareUnityWindow ".to_string()],
        };
        let policy = BypassPolicy::from_config(&cfg);
        // Trimmed, lowercased, and empties dropped — all before any callback.
        assert_eq!(policy.processes(), ["mstsc.exe"]);
        assert_eq!(policy.classes(), ["vmwareunitywindow"]);
    }

    #[test]
    fn default_config_yields_documented_policy() {
        let policy = default_policy();
        assert!(policy.processes().contains(&"mstsc.exe".to_string()));
        assert!(policy.processes().contains(&"virtualboxvm.exe".to_string()));
        assert_eq!(policy.classes(), ["vmwareunitywindow"]);
    }

    #[test]
    fn legacy_config_without_classes_keeps_processes_and_gains_default() {
        // Mirrors the freeze from the consuming side.
        let cfg = shared::config::Config::from_toml_str(
            r#"
            [vm_bypass]
            bypass_processes = ["custom.exe"]
            "#,
        )
        .unwrap();
        let policy = BypassPolicy::from_config(&cfg.vm_bypass);
        assert_eq!(policy.processes(), ["custom.exe"]);
        assert_eq!(policy.classes(), ["vmwareunitywindow"]);
    }

    #[test]
    fn processes_and_classes_stay_independent() {
        let cfg = VmBypassConfig {
            bypass_processes: vec!["only.exe".to_string()],
            bypass_classes: vec!["OnlyClass".to_string()],
        };
        let policy = BypassPolicy::from_config(&cfg);
        assert_eq!(
            policy.classify(&identity(Some("only.exe"), Some("Other"))),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
        assert_eq!(
            policy.classify(&identity(Some("other.exe"), Some("OnlyClass"))),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
        assert_eq!(
            policy.classify(&identity(Some("other.exe"), Some("Other"))),
            BypassDecision::ContinueWiraDeskMatching
        );
    }

    // --- adapter composition -----------------------------------

    #[test]
    fn adapters_compose_into_facts_deterministically() {
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_B)),
            (WindowId(3), None),
        ]);
        let desktops = FakeDesktops(vec![
            (WindowId(1), Some(true)),
            (WindowId(2), Some(true)),
            (WindowId(3), Some(true)),
        ]);
        let origin = capture_spatial_context(&monitors, WindowId(1));

        assert_eq!(
            evaluate_spatial(
                &origin,
                &collect_spatial_facts(&monitors, &desktops, WindowId(1))
            ),
            SpatialDecision::Eligible
        );
        assert_eq!(
            evaluate_spatial(
                &origin,
                &collect_spatial_facts(&monitors, &desktops, WindowId(2))
            ),
            SpatialDecision::Ineligible(SpatialRejection::DifferentMonitor)
        );
        assert_eq!(
            evaluate_spatial(
                &origin,
                &collect_spatial_facts(&monitors, &desktops, WindowId(3))
            ),
            SpatialDecision::Ineligible(SpatialRejection::MonitorUnavailable)
        );
    }

    #[test]
    fn origin_monitor_resolved_once_is_reused_for_every_candidate() {
        let monitors = FakeMonitors(vec![(WindowId(1), Some(MONITOR_A))]);
        let origin = capture_spatial_context(&monitors, WindowId(1));
        let again = capture_spatial_context(&monitors, WindowId(1));
        assert_eq!(origin, again, "origin resolution must be deterministic");
    }

    #[test]
    fn foreground_source_feeds_the_policy_unchanged() {
        let source = FakeForeground(identity(Some("vmware.exe"), Some("Some")));
        let policy = default_policy();
        assert_eq!(
            policy.classify(&source.foreground_identity()),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    #[test]
    fn decisions_are_repeatable() {
        let policy = default_policy();
        let id = identity(Some("notepad.exe"), Some("Notepad"));
        for _ in 0..8 {
            assert_eq!(
                policy.classify(&id),
                BypassDecision::ContinueWiraDeskMatching
            );
        }
    }
}
