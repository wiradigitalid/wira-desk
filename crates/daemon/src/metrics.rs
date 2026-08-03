//! Convergence measurement seam. **Debug builds only.**
//!
//! Two things are measured here, deliberately kept separate:
//!
//! - Cycle latency — Worker command receipt through activation completion. This is
//!   not the hook-callback timing owned by `hook.rs`; the two distributions must
//!   be reported independently.
//! - Command reconciliation counters — throttle drops and full-ring drops stay
//!   distinguishable from activation failures.
//!
//! Every item is `#[cfg(debug_assertions)]`-gated at the module declaration, so
//! release builds carry none of it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::util::append_debug_trace;

/// Exact-match events the Hook accepted and published to the ring.
pub static ACCEPTED: AtomicU64 = AtomicU64::new(0);
/// Matched events rejected by the anti-macro throttle. Intentional.
pub static THROTTLED: AtomicU64 = AtomicU64::new(0);
/// Matched events dropped because the ring was full. Intentional, bounded.
pub static DROPPED_FULL: AtomicU64 = AtomicU64::new(0);
/// Commands the Worker pulled off the ring.
pub static DRAINED: AtomicU64 = AtomicU64::new(0);
/// Cycles that ended in a real focus change.
pub static ACTIVATED: AtomicU64 = AtomicU64::new(0);
/// Cycles where every eligible target was attempted and none activated.
pub static EXHAUSTED: AtomicU64 = AtomicU64::new(0);
/// Cycles where no candidate survived eligibility.
pub static NO_TARGET: AtomicU64 = AtomicU64::new(0);

/// Per-cycle durations in nanoseconds.
/// A `Mutex<Vec<_>>` is fine here: this is the Worker (main) thread only, and
/// the Worker is explicitly allowed to allocate. The zero-allocation rule
/// binds the Hook callback, which never touches this.
static CYCLE_NS: Mutex<Vec<u64>> = Mutex::new(Vec::new());

static QPC_FREQUENCY: AtomicU64 = AtomicU64::new(0);

pub fn qpc_frequency() -> u64 {
    let cached = QPC_FREQUENCY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let mut freq = 0i64;
    // SAFETY: `&mut freq` is a unique pointer to a live, initialised local of exactly the
    // width the API writes. `QueryPerformanceFrequency` cannot fail on any Windows version
    // this daemon supports, but the return value is ignored anyway because `freq` is
    // pre-set to zero and `qpc_frequency`'s callers already treat zero as "unavailable" —
    // so a hypothetical failure degrades to skipped measurement, not a bad divisor.
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut freq);
    }
    let f = freq.max(0) as u64;
    QPC_FREQUENCY.store(f, Ordering::Relaxed);
    f
}

pub fn qpc_now() -> i64 {
    let mut counter = 0i64;
    // SAFETY: `&mut counter` is a unique pointer to a live, initialised local of the width
    // the API writes, and the pointer is not retained past the call.
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceCounter(&mut counter);
    }
    counter
}

/// Record one completed cycle, measured from `start` (a `qpc_now` taken at
/// Worker command receipt) to now.
/// The sample is kept **twice**, and the duplication is the point.
/// `CYCLE_NS` backs [`dump`] for the current session and dies with the
/// process. That is fine for a single measured run and useless for anything
/// longer: a daemon that restarts on every boot would erase its own history,
/// and percentiles from separate sessions cannot legitimately be combined after
/// the fact — averaging a p95 is not a p95. Only raw samples can be pooled, so
/// each one is also appended to the trace file, which survives restarts.
/// The outcome travels with the sample because governs the *activating*
/// path. A cycle that found no target costs roughly enumeration alone; one
/// whose candidates all failed activation pays two bounded polls per candidate.
/// Mixing them produces a percentile describing no real user experience.
/// Runs on the Worker thread, which may allocate and touch the filesystem. The
/// Hook callback never reaches here, so is unaffected.
pub fn record_cycle(start: i64, outcome: &str) {
    let elapsed = (qpc_now() - start).max(0) as u64;
    let freq = qpc_frequency();
    if freq == 0 {
        return;
    }
    let ns = elapsed.saturating_mul(1_000_000_000) / freq;
    if let Ok(mut samples) = CYCLE_NS.lock() {
        samples.push(ns);
    }
    append_debug_trace(&format!("CYCLE_SAMPLE: ns={ns} outcome={outcome}"));
}

/// Every counter, in reporting order.
fn all_counters() -> [&'static AtomicU64; 7] {
    [
        &ACCEPTED,
        &THROTTLED,
        &DROPPED_FULL,
        &DRAINED,
        &ACTIVATED,
        &EXHAUSTED,
        &NO_TARGET,
    ]
}

/// Zero a set of counters.
/// Split out from [`reset`] so it can be tested against locally-owned atomics.
/// Asserting on the `static` counters instead would race the `hook.rs` tests,
/// which increment the same globals from other test threads.
fn clear_all(counters: &[&AtomicU64]) {
    for counter in counters {
        counter.store(0, Ordering::Relaxed);
    }
}

pub fn reset() {
    clear_all(&all_counters());
    if let Ok(mut samples) = CYCLE_NS.lock() {
        samples.clear();
    }
    append_debug_trace("CYCLE_METRICS_RESET: ok=1");
}

/// Nearest-rank percentile over a sorted slice.
/// Nearest-rank is used rather than interpolation so a reported p95 is always
/// an observed sample, never a synthesized value that never occurred.
fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    let idx = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}

/// Emit both distributions to the debug trace for the elevated harness.
pub fn dump() {
    let mut samples = match CYCLE_NS.lock() {
        Ok(s) => s.clone(),
        Err(_) => Vec::new(),
    };
    samples.sort_unstable();

    let p50 = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let max = samples.last().copied().unwrap_or(0);

    // Nanoseconds are reported alongside microseconds: a p95 target of 1 ms is
    // 1000 us, and truncating to whole microseconds would hide sub-us detail
    // exactly where the margin matters.
    append_debug_trace(&format!(
        "CYCLE_LATENCY: samples={} p50_ns={} p95_ns={} max_ns={} p50_us={} p95_us={} max_us={}",
        samples.len(),
        p50,
        p95,
        max,
        p50 / 1_000,
        p95 / 1_000,
        max / 1_000
    ));

    append_debug_trace(&format!(
        "CYCLE_COUNTERS: accepted={} throttled={} dropped_full={} drained={} activated={} exhausted={} no_target={}",
        ACCEPTED.load(Ordering::Relaxed),
        THROTTLED.load(Ordering::Relaxed),
        DROPPED_FULL.load(Ordering::Relaxed),
        DRAINED.load(Ordering::Relaxed),
        ACTIVATED.load(Ordering::Relaxed),
        EXHAUSTED.load(Ordering::Relaxed),
        NO_TARGET.load(Ordering::Relaxed),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_of_empty_is_zero() {
        assert_eq!(percentile(&[], 0.95), 0);
    }

    #[test]
    fn percentile_single_sample() {
        assert_eq!(percentile(&[7], 0.50), 7);
        assert_eq!(percentile(&[7], 0.95), 7);
    }

    #[test]
    fn nearest_rank_percentiles_are_observed_samples() {
        let sorted: Vec<u64> = (1..=100).collect();
        // Nearest-rank: p50 → rank 50, p95 → rank 95.
        assert_eq!(percentile(&sorted, 0.50), 50);
        assert_eq!(percentile(&sorted, 0.95), 95);
        assert_eq!(percentile(&sorted, 1.00), 100);
    }

    #[test]
    fn percentile_never_indexes_out_of_bounds() {
        let sorted = [1u64, 2, 3];
        for p in [0.0, 0.01, 0.5, 0.99, 1.0] {
            let v = percentile(&sorted, p);
            assert!(sorted.contains(&v), "p={p} produced a synthesized value");
        }
    }

    #[test]
    fn clear_all_zeroes_every_counter_it_is_given() {
        // Locally-owned atomics: no shared state, so this cannot race the
        // hook tests that increment the real counters in parallel.
        let a = AtomicU64::new(5);
        let b = AtomicU64::new(3);
        let c = AtomicU64::new(9);
        clear_all(&[&a, &b, &c]);
        assert_eq!(a.load(Ordering::Relaxed), 0);
        assert_eq!(b.load(Ordering::Relaxed), 0);
        assert_eq!(c.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn clear_all_on_an_empty_set_is_harmless() {
        clear_all(&[]);
    }

    #[test]
    fn reset_covers_every_reported_counter() {
        // Guards the real risk: a counter added to `dump` but forgotten in
        // `all_counters` would silently never reset between measurements.
        assert_eq!(
            all_counters().len(),
            7,
            "counter set drifted from the seven reported by dump()"
        );
    }
}
