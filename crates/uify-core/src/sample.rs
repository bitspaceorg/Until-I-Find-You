//! Time-indexed tracker output.
//!
//! Every tracker produces `Sample<G>` values, where `G` is the tracker's
//! geometric primitive (point, SE(3), homography, contour, etc.). The sample
//! carries the value, an uncertainty estimate, a confidence score, and a
//! monotonic timestamp. Consumers can gate on any of these.

use core::fmt;

/// Host-monotonic timestamp in nanoseconds since an arbitrary fixed epoch.
/// The same clock MUST be used by the capture, filter, and consumer sides —
/// otherwise latency compensation is meaningless.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Nanoseconds since epoch.
    #[inline]
    pub const fn from_nanos(ns: u64) -> Self {
        Self(ns)
    }

    /// Raw nanosecond value.
    #[inline]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}

/// Normalized confidence score in `[0, 1]`. Separate from covariance: a
/// tracker can be geometrically certain (low covariance) but semantically
/// unsure the object it is tracking is still the correct one (low confidence).
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Confidence(f32);

impl Confidence {
    /// Construct a confidence, clamped into the unit interval.
    #[inline]
    pub fn new(x: f32) -> Self {
        Self(x.clamp(0.0, 1.0))
    }

    /// Raw value.
    #[inline]
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}

/// One tracker output.
///
/// `value` is the geometric estimate. `covariance` lives in the tangent
/// space of `G` (for group-valued trackers — SE(3), SL(3), etc.); for
/// vector-valued trackers it is the usual Euclidean covariance.
#[derive(Clone, Debug)]
pub struct Sample<G, C> {
    /// When this sample was produced (host-monotonic clock).
    pub t: Timestamp,
    /// Geometric estimate.
    pub value: G,
    /// Tangent-space covariance. Shape is `G`-specific.
    pub covariance: C,
    /// Semantic confidence, `[0, 1]`.
    pub confidence: Confidence,
}
