//! Worker-side command drain (hidden-window main thread).
//! This is the composition point for cycling, context-safe spatial filtering,
//! and window arrangement. It owns no policy of its own: every decision comes
//! from a frozen contract.

use shared::{Command, Config};

use crate::arrangement::win32::{apply_plan, resolve_context, Win32WindowMover};
use crate::arrangement::{monitor, snap, stack, PlacementPlan, PlanError};
use crate::context::spatial::{enumerate_monitors, index_of_window_monitor, Win32Monitors};
use crate::context::virtual_desktop::VirtualDesktopManager;
use crate::context::{
    capture_spatial_context, collect_spatial_facts, evaluate_spatial, MonitorSource,
    VirtualDesktopSource,
};
use crate::cycling::activation::Win32Activator;
use crate::cycling::eligibility::WindowEligibility;
use crate::cycling::source::{capture_active_context, Win32CandidateSource};
use crate::cycling::{
    ActivationOutcome, Activator, ActiveContext, Candidate, CandidateSource, CycleOutcome,
    EligibilityPolicy, WindowId,
};
use crate::ring;
use crate::util::debug_log;

/// `VK_NONAME` — reserved and unassigned, so nothing acts on it.
const VK_NONAME: u16 = 0xFC;
const VK_LWIN: i32 = 0x5B;
const VK_RWIN: i32 = 0x5C;

/// Stop the shell from treating a still-held Win key as a lone press.
/// Wira Desk swallows the main key of the shortcut, so the shell would otherwise
/// see Win-down followed by Win-up with nothing between, and open Start. The
/// original fix swallowed the Win key-up instead, which left the focused
/// application believing Win was still held — every later keystroke became
/// Win+key, the sticky-modifier bug.
/// Injecting one unassigned key while Win is down makes the press a
/// combination, so the real key-up can pass through untouched.
/// **This runs on the Worker, never in the hook callback.** Calling `SendInput`
/// from inside the low-level hook raced the activation happening here and
/// stopped cycling from moving focus at all.
fn suppress_start_menu() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    };
    // SAFETY: `GetAsyncKeyState` takes no pointers and is callable from any thread.
    //
    // `zeroed` is valid for `[INPUT; 2]` because `INPUT` is a tag plus a union of
    // `MOUSEINPUT`/`KEYBDINPUT`/`HARDWAREINPUT`, all of which are plain integer structs, so
    // no bit pattern is invalid. The invariant that matters is tag/arm agreement: `r#type`
    // is set to `INPUT_KEYBOARD` and the arm written is `Anonymous.ki`, so `SendInput` reads
    // the same arm we initialised. Writing `ki` while claiming `INPUT_MOUSE` would have it
    // reinterpret those bytes as a different struct.
    //
    // `inputs.as_ptr()` is an array of exactly `inputs.len()` elements, and the third
    // argument is `size_of::<INPUT>()` — the stride Windows uses to walk the array, so a
    // mismatch there would make it read past the end. Both derive from the same type.
    //
    // Thread context is a precondition too, not just a design note: this must run on the
    // Worker. Calling `SendInput` from inside the low-level keyboard hook re-enters input
    // processing and raced the activation this function follows, which is what stopped
    // cycling from moving focus at all.
    unsafe {
        // Only if the user is still holding Win — otherwise the chord is over
        // and an injected key would be a stray keystroke.
        let held = (GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0;
        if !held {
            return;
        }

        let mut inputs: [INPUT; 2] = std::mem::zeroed();
        for (i, input) in inputs.iter_mut().enumerate() {
            input.r#type = INPUT_KEYBOARD;
            input.Anonymous.ki = KEYBDINPUT {
                wVk: VK_NONAME,
                wScan: 0,
                dwFlags: if i == 1 { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            };
        }
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Drain the Hook→Worker ring until empty.
pub fn drain_commands() {
    while let Some(raw) = ring::pop() {
        #[cfg(debug_assertions)]
        crate::metrics::DRAINED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match Command::from_u8(raw) {
            Command::Nop => {}
            Command::Cycle => {
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace("WORKER_DRAIN: cycle=1");
                execute_cycle();
            }
            Command::SnapLeft
            | Command::SnapRight
            | Command::SnapTop
            | Command::SnapBottom
            | Command::SnapMaximize => {
                execute_snap(Command::from_u8(raw));
            }
            Command::OverlappingStack => execute_stack(),
            Command::MoveToNextMonitor => execute_monitor_move(),
        }
    }
}

// ── Context-safe cycling ────────────────────────────────────────────

/// One context-safe cycle pass.
/// The active context and the origin monitor are each sampled **once** and
/// carried through the whole pass, so the result stays deterministic while
/// windows open, close, and move.
fn execute_cycle() {
    #[cfg(debug_assertions)]
    let started = crate::metrics::qpc_now();

    let active = capture_active_context();
    let monitors = Win32Monitors;
    let spatial = capture_spatial_context(&monitors, active.foreground);

    // The interface is created once and reused. Creating it per command cost
    // ~19 ms — `CoInitializeEx` + `CoCreateInstance` on every keystroke — which
    // alone put the original cycling latency target out of reach. COM
    // ownership must stay on the Worker thread with an explicit lifetime; it
    // does not require per-command construction.
    let outcome = with_virtual_desktops(|desktops| {
        run_context_safe_cycle(
            &Win32CandidateSource,
            &WindowEligibility,
            &mut Win32Activator,
            &active,
            &monitors,
            desktops,
            &spatial,
        )
    });

    // After the focus change, while Win is still down. Doing it here rather
    // than in the callback keeps `SendInput` off the input-processing path.
    suppress_start_menu();

    // The outcome is recorded with the sample so the durable trace can later be
    // filtered to the activating cycles the latency metric actually governs.
    #[cfg(debug_assertions)]
    crate::metrics::record_cycle(
        started,
        match &outcome {
            CycleOutcome::Activated(_) => "activated",
            CycleOutcome::Exhausted => "exhausted",
            CycleOutcome::NoEligibleTarget => "no_target",
        },
    );

    #[cfg(debug_assertions)]
    {
        use std::sync::atomic::Ordering;
        match &outcome {
            CycleOutcome::Activated(target) => {
                crate::metrics::ACTIVATED.fetch_add(1, Ordering::Relaxed);
                crate::util::append_debug_trace(&format!("WORKER_CYCLE: activated={}", target.0));
            }
            CycleOutcome::Exhausted => {
                crate::metrics::EXHAUSTED.fetch_add(1, Ordering::Relaxed);
                crate::util::append_debug_trace("WORKER_CYCLE: exhausted=1");
            }
            CycleOutcome::NoEligibleTarget => {
                crate::metrics::NO_TARGET.fetch_add(1, Ordering::Relaxed);
                crate::util::append_debug_trace("WORKER_CYCLE: no_target=1");
            }
        }
    }

    let _ = outcome;
}

/// Cycle driver with the spatial gate layered on top of eligibility.
/// Both filters must pass. Cycling order and eligibility rules are unchanged;
/// the spatial adapter only removes candidates, never reorders them.
#[allow(clippy::too_many_arguments)]
fn run_context_safe_cycle<S, P, A, M, V>(
    source: &S,
    policy: &P,
    activator: &mut A,
    active: &ActiveContext,
    monitors: &M,
    desktops: Option<&V>,
    spatial: &crate::context::SpatialContext,
) -> CycleOutcome
where
    S: CandidateSource + ?Sized,
    P: EligibilityPolicy + ?Sized,
    A: Activator + ?Sized,
    M: MonitorSource + ?Sized,
    V: VirtualDesktopSource + ?Sized,
{
    let candidates = source.snapshot();

    let eligible: Vec<WindowId> = candidates
        .iter()
        .filter(|c| policy.evaluate(active, c).is_eligible())
        .filter(|c| context_allows(monitors, desktops, spatial, c))
        .map(|c| c.facts.window)
        .collect();

    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!(
        "CYCLE_ELIGIBLE: active={} candidates={} eligible={:?}",
        active.foreground.0,
        candidates.len(),
        eligible.iter().map(|w| w.0).collect::<Vec<_>>()
    ));

    if eligible.is_empty() {
        return CycleOutcome::NoEligibleTarget;
    }

    for target in crate::cycling::cycle_order(&candidates, active) {
        if !eligible.contains(&target) {
            continue;
        }
        if activator.activate(target) == ActivationOutcome::Activated {
            return CycleOutcome::Activated(target);
        }
    }

    CycleOutcome::Exhausted
}

thread_local! {
 /// Worker-thread-owned virtual-desktop interface.
 /// `thread_local!` is what keeps COM ownership on the Worker thread: the
 /// value can only ever be touched from the thread that created it, and it
 /// is dropped — releasing the interface and its apartment — when that
 /// thread exits. `RefCell<Option<..>>` distinguishes "not tried yet" from
 /// "tried and unavailable", so a machine without COM is not re-probed on
 /// every keystroke.
    static VIRTUAL_DESKTOPS: std::cell::RefCell<Option<Option<VirtualDesktopManager>>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with the shared virtual-desktop adapter, creating it on first use.
fn with_virtual_desktops<R>(f: impl FnOnce(Option<&VirtualDesktopManager>) -> R) -> R {
    VIRTUAL_DESKTOPS.with(|cell| {
        let mut slot = cell.borrow_mut();
        let created = slot.get_or_insert_with(VirtualDesktopManager::create);
        f(created.as_ref())
    })
}

/// Spatial gate for one candidate. Fails closed when COM is unavailable.
fn context_allows<M, V>(
    monitors: &M,
    desktops: Option<&V>,
    spatial: &crate::context::SpatialContext,
    candidate: &Candidate,
) -> bool
where
    M: MonitorSource + ?Sized,
    V: VirtualDesktopSource + ?Sized,
{
    let Some(desktops) = desktops else {
        return false;
    };
    let facts = collect_spatial_facts(monitors, desktops, candidate.facts.window);
    evaluate_spatial(spatial, &facts).is_eligible()
}

// ── Arrangement ───────────────────────────────────────────────────────

fn execute_snap(command: Command) {
    let Some(ctx) = resolve_context() else {
        report_arrangement_failure("no platform context");
        return;
    };

    // Maximize asks Windows to maximize, rather than resizing the window to the work area.
    // The two look alike and are not: sizing leaves the window in the *normal* state, so the
    // title bar still offers Maximize rather than Restore, a double-click maximizes it again
    // to a slightly different size, and the application never receives `WM_GETMINMAXINFO` and
    // so never gets the maximized bounds it asked for.
    //
    // The geometric plan stays as the fallback for a window whose own style forbids
    // maximizing -- a fixed-size dialog, a tool palette -- where sizing it to the work area
    // is still the best available answer and is what this did before.
    if command == Command::SnapMaximize && crate::arrangement::win32::try_real_maximize(ctx.target)
    {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace("WORKER_ARRANGE: SnapMaximize real=1");
        return;
    }

    let plan = match command {
        Command::SnapLeft => snap::plan_snap_left(&ctx.work_area, ctx.target),
        Command::SnapRight => snap::plan_snap_right(&ctx.work_area, ctx.target),
        Command::SnapTop => snap::plan_snap_top(&ctx.work_area, ctx.target),
        Command::SnapBottom => snap::plan_snap_bottom(&ctx.work_area, ctx.target),
        Command::SnapMaximize => snap::plan_snap_maximize(&ctx.work_area, ctx.target),
        _ => return,
    };

    apply_or_report(plan, command);
}

/// Move the active window to the next monitor, keeping its share of the work area.
///
/// The display set is enumerated **here**, on the Worker, and never on the Hook thread: the
/// hook callback is budgeted under 10 ms and must not allocate, and the monitor list is only
/// needed once a command has already been accepted. The Hook answers *whose chord is this*;
/// the Worker answers *what is a legal target and where does it go* — the same split
/// `DEC-006` drew for target eligibility.
fn execute_monitor_move() {
    // `resolve_context` first, because it is the one gate that refuses a Wira Desk window of
    // our own (`LBR-WM-6`, `DEC-006`). Enumerating before that check would do work for a
    // target that is about to be refused.
    let Some(ctx) = resolve_context() else {
        report_arrangement_failure("no platform context");
        return;
    };
    let hwnd = ctx.target.0;

    // Fresh every invocation, cached nowhere: an `HMONITOR` is a handle rather than an
    // identity, so a list kept between keypresses outlives the configuration it described.
    let monitors = enumerate_monitors();
    let Some(from) = index_of_window_monitor(hwnd, &monitors) else {
        report_arrangement_failure("active window resolved to no enumerated monitor");
        return;
    };
    let Some(to) = monitor::next_monitor_index(monitors.len(), from) else {
        // A single attached monitor. This is a **successful no-op**, not a failure: nothing
        // moves, nothing is shown, and nothing is logged beyond the debug trace. Warning here
        // would train the user to ignore a log that is meant to carry real problems.
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(
            "WORKER_ARRANGE: MoveToNextMonitor noop=1 reason=one_monitor",
        );
        return;
    };

    let Some(window_rect) = current_window_rect(hwnd) else {
        report_arrangement_failure("could not read the active window rect");
        return;
    };

    // No restore here any more: `apply_plan` does it for every placement of every command,
    // which is what this comment used to claim about the snap path without it being true.
    apply_or_report(
        monitor::plan_move_to_monitor(
            &monitors[from].work,
            &monitors[to].work,
            ctx.target,
            window_rect,
        ),
        Command::MoveToNextMonitor,
    );
}

/// The window's current outer rect, as the planner's proportional source.
fn current_window_rect(hwnd: isize) -> Option<crate::arrangement::Rect> {
    use windows_sys::Win32::Foundation::{FALSE, RECT};
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: `hwnd` came from `resolve_context`, which validated it with `IsWindow`, and
    // `&mut r` is a unique pointer to a live local of exactly the type the API writes. A
    // failed call leaves `r` as initialised above, which is why the result is checked rather
    // than trusted — a zeroed rect would otherwise read as a degenerate window.
    if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut r) } == FALSE
    {
        return None;
    }
    crate::arrangement::win32::rect_from_win32(r).ok()
}

// The Worker's own configuration, replaced only by an accepted reload.
// `thread_local!` rather than a `static`: the Worker is the main thread and the
// `WM_APP_RELOAD_CONFIG` handler runs on it, so installing a snapshot is a
// plain assignment on the owning thread. Single-threaded by construction means
// no lock on the arrangement path, and it makes the Hook/Worker isolation the
// AC demands true in both directions rather than by convention.
thread_local! {
    static WORKER_CONFIG: std::cell::RefCell<Option<crate::config::WorkerSnapshot>> =
        const { std::cell::RefCell::new(None) };
}

/// Install the snapshot handed over by an accepted reload.
pub fn install_config_snapshot(snapshot: crate::config::WorkerSnapshot) {
    WORKER_CONFIG.with(|slot| *slot.borrow_mut() = Some(snapshot));
}

/// The layout configuration currently in force.
/// Falls back to a single read at first use so a daemon that has not yet
/// received a reload behaves exactly as it did before this story. That read
/// happens once per process, not once per command as it used to: the product
/// forbids watching and polling, and re-reading the file on every
/// keystroke-driven arrangement was closer to both than it needed to be.
fn layout_config() -> shared::config::LayoutConfig {
    WORKER_CONFIG.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            let cfg = Config::load_or_default(&shared::config_path());
            *slot = Some(crate::config::WorkerSnapshot { layout: cfg.layout });
        }
        slot.as_ref()
            .expect("populated immediately above")
            .layout
            .clone()
    })
}

/// `OverlappingStack` reuses the candidate contract for live
/// same-application windows, then keeps only those on the active target
/// monitor — without requiring the full spatial contract.
fn execute_stack() {
    let layout = layout_config();

    let Some(ctx) = resolve_context() else {
        report_arrangement_failure("no platform context");
        return;
    };

    let active = capture_active_context();
    let monitors = Win32Monitors;
    let origin = monitors.monitor_of(active.foreground);

    let candidates: Vec<WindowId> = Win32CandidateSource
        .snapshot()
        .into_iter()
        .filter(|c| WindowEligibility.evaluate(&active, c).is_eligible())
        // Same monitor only. This is a plain monitor comparison, deliberately
        // not the spatial contract: stacking must work before spatial filtering
        // converges.
        .filter(|c| origin.is_some() && monitors.monitor_of(c.facts.window) == origin)
        .map(|c| c.facts.window)
        .collect();

    apply_or_report(
        stack::plan_stack(&layout, &ctx.work_area, &candidates),
        Command::OverlappingStack,
    );
}

fn apply_or_report(plan: Result<PlacementPlan, PlanError>, command: Command) {
    match plan {
        Ok(plan) => {
            if plan.is_noop() {
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace(&format!("WORKER_ARRANGE: {command:?} noop=1"));
                return;
            }
            let (applied, skipped) = apply_plan(&mut Win32WindowMover, &plan.placements);
            #[cfg(debug_assertions)]
            crate::util::append_debug_trace(&format!(
                "WORKER_ARRANGE: {command:?} applied={applied} skipped={skipped}"
            ));
            let _ = (applied, skipped);
        }
        Err(err) => report_arrangement_failure(&format!("{command:?} {err:?}")),
    }
}

/// Arrangement failures follow the Tier-2 path: a diagnostic, never a popup,
/// and never a downgrade of an existing Critical tray state.
fn report_arrangement_failure(detail: &str) {
    debug_log(&format!("Wira Desk: arrangement failed — {detail}"));
    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!("WORKER_ARRANGE: failed detail={detail}"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::fixtures::{FakeDesktops, FakeMonitors, MONITOR_A, MONITOR_B};
    use crate::context::SpatialContext;
    use crate::cycling::fixtures::{normal, ordered, ScriptedActivator, StaticSource};
    use crate::cycling::ActivationOutcome;

    fn active_ctx(w: isize) -> ActiveContext {
        crate::cycling::fixtures::active(w)
    }

    #[test]
    fn candidate_on_another_monitor_is_never_activated() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_B)),
            (WindowId(3), Some(MONITOR_A)),
        ]);
        let desktops = FakeDesktops(vec![
            (WindowId(1), Some(true)),
            (WindowId(2), Some(true)),
            (WindowId(3), Some(true)),
        ]);
        let spatial = SpatialContext {
            origin_monitor: Some(MONITOR_A),
        };
        let mut activator = ScriptedActivator::always(ActivationOutcome::Activated);
        let outcome = run_context_safe_cycle(
            &StaticSource(candidates),
            &WindowEligibility,
            &mut activator,
            &active_ctx(1),
            &monitors,
            Some(&desktops),
            &spatial,
        );
        assert_eq!(outcome, CycleOutcome::Activated(WindowId(3)));
        assert!(!activator.attempts.contains(&WindowId(2)));
    }

    #[test]
    fn candidate_on_another_virtual_desktop_is_never_activated() {
        let candidates = ordered(vec![normal(1), normal(2), normal(3)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_A)),
            (WindowId(3), Some(MONITOR_A)),
        ]);
        let desktops = FakeDesktops(vec![
            (WindowId(1), Some(true)),
            (WindowId(2), Some(false)),
            (WindowId(3), Some(true)),
        ]);
        let spatial = SpatialContext {
            origin_monitor: Some(MONITOR_A),
        };
        let mut activator = ScriptedActivator::always(ActivationOutcome::Activated);
        let outcome = run_context_safe_cycle(
            &StaticSource(candidates),
            &WindowEligibility,
            &mut activator,
            &active_ctx(1),
            &monitors,
            Some(&desktops),
            &spatial,
        );
        assert_eq!(outcome, CycleOutcome::Activated(WindowId(3)));
        assert!(!activator.attempts.contains(&WindowId(2)));
    }

    #[test]
    fn missing_com_leaves_focus_unchanged() {
        // No virtual-desktop adapter means nothing can be proven eligible.
        let candidates = ordered(vec![normal(1), normal(2)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_A)),
        ]);
        let spatial = SpatialContext {
            origin_monitor: Some(MONITOR_A),
        };
        let mut activator = ScriptedActivator::always(ActivationOutcome::Activated);
        let outcome = run_context_safe_cycle(
            &StaticSource(candidates),
            &WindowEligibility,
            &mut activator,
            &active_ctx(1),
            &monitors,
            None::<&FakeDesktops>,
            &spatial,
        );
        assert_eq!(outcome, CycleOutcome::NoEligibleTarget);
        assert!(activator.attempts.is_empty());
    }

    #[test]
    fn unknown_origin_monitor_leaves_focus_unchanged() {
        let candidates = ordered(vec![normal(1), normal(2)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_A)),
        ]);
        let desktops = FakeDesktops(vec![(WindowId(1), Some(true)), (WindowId(2), Some(true))]);
        let spatial = SpatialContext {
            origin_monitor: None,
        };
        let mut activator = ScriptedActivator::always(ActivationOutcome::Activated);
        let outcome = run_context_safe_cycle(
            &StaticSource(candidates),
            &WindowEligibility,
            &mut activator,
            &active_ctx(1),
            &monitors,
            Some(&desktops),
            &spatial,
        );
        assert_eq!(outcome, CycleOutcome::NoEligibleTarget);
    }

    #[test]
    fn epic_two_ordering_survives_the_spatial_gate() {
        // Spatial filtering removes candidates; it must never reorder them.
        let candidates = ordered(vec![normal(1), normal(2), normal(3), normal(4)]);
        let monitors = FakeMonitors(vec![
            (WindowId(1), Some(MONITOR_A)),
            (WindowId(2), Some(MONITOR_A)),
            (WindowId(3), Some(MONITOR_B)),
            (WindowId(4), Some(MONITOR_A)),
        ]);
        let desktops = FakeDesktops(vec![
            (WindowId(1), Some(true)),
            (WindowId(2), Some(true)),
            (WindowId(3), Some(true)),
            (WindowId(4), Some(true)),
        ]);
        let spatial = SpatialContext {
            origin_monitor: Some(MONITOR_A),
        };
        let mut activator = ScriptedActivator::always(ActivationOutcome::InvalidTarget);
        let outcome = run_context_safe_cycle(
            &StaticSource(candidates),
            &WindowEligibility,
            &mut activator,
            &active_ctx(1),
            &monitors,
            Some(&desktops),
            &spatial,
        );
        assert_eq!(outcome, CycleOutcome::Exhausted);
        // 4 then 2 - least-recently-used first - with 3 removed by the
        // spatial gate and 1 (active) never retried.
        assert_eq!(activator.attempts, vec![WindowId(4), WindowId(2)]);
    }
}
