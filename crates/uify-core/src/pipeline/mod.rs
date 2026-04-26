//! Pipeline stages. A tracker is a composition of stages connected by typed
//! channels. Each stage is pure: it maps an input to zero-or-more outputs.
//!
//! The five canonical stages:
//!
//! ```text
//! Source → Detector → Associator → Filter → Smoother → Sink
//! ```
//!
//! Sources and sinks live outside this crate (runtime / transports). Detectors,
//! associators, filters, and smoothers are all defined here against trait
//! interfaces so they can be swapped without touching the rest of the graph.

use crate::sample::{Sample, Timestamp};

/// Generic stage trait: input → output, plus a per-stage reset.
pub trait Stage {
    /// Per-tick input type.
    type In<'a>;
    /// Per-tick output type.
    type Out;

    /// Process one input. Returns `None` if the stage has no output this tick
    /// (e.g. a detector may skip frames, a filter may wait for a seed).
    fn step(&mut self, input: Self::In<'_>) -> Option<Self::Out>;

    /// Discard any internal state.
    fn reset(&mut self);
}

/// Consumer of tracker output.
///
/// Transports (OSC, MIDI, shared memory, FFI) implement this to receive
/// `Sample<G, C>` values. A tracker — or a runtime adapter wrapping one —
/// owns a `Sink` and calls `write` on it once per produced sample. Trackers
/// MUST NOT depend on concrete transport types; they only see this trait.
pub trait Sink<G, C> {
    /// Error type produced by [`write`](Self::write).
    type Error;

    /// Write one sample. Implementations should be cheap — transports often
    /// run on the vision or audio thread.
    fn write(&mut self, sample: &Sample<G, C>) -> Result<(), Self::Error>;
}

/// Shared bookkeeping for streaming trackers wrapping a recursive filter.
///
/// Every concrete tracker in this workspace stores a filter (linear KF,
/// manifold EKF, …), a snapshot of its construction-time state for `reset`,
/// and the previous timestamp for `dt` computation. `TrackerCommon` lifts
/// those three concerns into one place so that adding a new tracker only
/// requires writing the per-step measurement-to-filter glue.
#[derive(Clone, Debug)]
pub struct TrackerCommon<F: Clone> {
    /// Underlying filter. Public for direct access from the embedding tracker.
    pub filter: F,
    initial: F,
    last_t: Option<Timestamp>,
}

impl<F: Clone> TrackerCommon<F> {
    /// New tracker housekeeping around `filter`. The initial state is
    /// captured by clone so [`reset`](Self::reset) can restore it.
    pub fn new(filter: F) -> Self {
        let initial = filter.clone();
        Self {
            filter,
            initial,
            last_t: None,
        }
    }

    /// Returns the seconds elapsed since the previous call (or `0.0` on the
    /// first call) and stores `now` as the new previous timestamp.
    /// Out-of-order timestamps yield `0.0` — predictions are skipped
    /// rather than rewinding state.
    pub fn dt_since(&mut self, now: Timestamp) -> f64 {
        let dt = match self.last_t {
            Some(prev) => (now.as_nanos().saturating_sub(prev.as_nanos())) as f64 * 1e-9,
            None => 0.0,
        };
        self.last_t = Some(now);
        dt
    }

    /// Restore the filter to its construction-time state and forget the
    /// previous timestamp.
    pub fn reset(&mut self) {
        self.filter = self.initial.clone();
        self.last_t = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::{Confidence, Timestamp};

    struct VecSink {
        samples: Vec<(u64, f64, f64)>,
    }

    impl Sink<f64, f64> for VecSink {
        type Error = std::convert::Infallible;
        fn write(&mut self, s: &Sample<f64, f64>) -> Result<(), Self::Error> {
            self.samples.push((s.t.as_nanos(), s.value, s.covariance));
            Ok(())
        }
    }

    #[test]
    fn sink_trait_is_implementable() {
        let mut sink = VecSink { samples: vec![] };
        sink.write(&Sample {
            t: Timestamp::from_nanos(42),
            value: 2.5,
            covariance: 0.01,
            confidence: Confidence::new(0.9),
        })
        .unwrap();
        assert_eq!(sink.samples, vec![(42, 2.5, 0.01)]);
    }

    #[test]
    fn tracker_common_dt_first_call_is_zero() {
        let mut tc = TrackerCommon::new(0i32);
        assert_eq!(tc.dt_since(Timestamp::from_nanos(1_000_000_000)), 0.0);
    }

    #[test]
    fn tracker_common_dt_returns_seconds() {
        let mut tc = TrackerCommon::new(0i32);
        let _ = tc.dt_since(Timestamp::from_nanos(1_000_000_000));
        let dt = tc.dt_since(Timestamp::from_nanos(1_500_000_000));
        assert!((dt - 0.5).abs() < 1e-12);
    }

    #[test]
    fn tracker_common_dt_out_of_order_yields_zero() {
        let mut tc = TrackerCommon::new(0i32);
        let _ = tc.dt_since(Timestamp::from_nanos(2_000_000_000));
        let dt = tc.dt_since(Timestamp::from_nanos(1_000_000_000));
        assert_eq!(dt, 0.0);
    }

    #[test]
    fn tracker_common_reset_restores_filter_and_clears_timestamp() {
        let mut tc = TrackerCommon::new(7i32);
        tc.filter = 99;
        let _ = tc.dt_since(Timestamp::from_nanos(1));
        tc.reset();
        assert_eq!(tc.filter, 7);
        // Post-reset, `dt_since` should behave like a first call.
        assert_eq!(tc.dt_since(Timestamp::from_nanos(123)), 0.0);
    }
}
