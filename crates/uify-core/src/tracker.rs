//! The `Tracker<G>` trait — the uniform interface every tracker implements.

use crate::sample::Sample;

/// Output of a single tracker `step`: either an updated sample, `None` while
/// the tracker has not yet locked on, or a `TrackerError`.
pub type TrackerOutput<G, C> = Result<Option<Sample<G, C>>, TrackerError>;

/// Generic tracker. `G` is the geometric primitive (e.g. `Vec2`, `SE3`,
/// `Homography`, contour, landmarks). `C` is the tangent-space covariance
/// type associated with `G`.
///
/// A tracker is stateful: it is a streaming estimator, not a one-shot detector.
/// Callers feed it frames (or frame-derived measurements) via a higher-level
/// pipeline; in exchange they get a stream of `Sample<G, C>` values.
pub trait Tracker {
    /// Geometric primitive produced by this tracker.
    type Geometry;

    /// Covariance type for `Geometry` (tangent-space for group-valued `G`).
    type Covariance;

    /// Measurement type accepted by this tracker. For most vision trackers
    /// this is a reference to a captured frame or a set of per-frame
    /// detections; runtime adapters wrap the concrete type for them.
    type Measurement<'a>;

    /// Incorporate a new measurement. Returns the updated estimate, or
    /// `None` if the tracker has not yet converged / locked onto a target.
    fn step(
        &mut self,
        measurement: Self::Measurement<'_>,
    ) -> TrackerOutput<Self::Geometry, Self::Covariance>;

    /// Reset tracker state. After reset the tracker behaves as freshly
    /// constructed — prior history is discarded.
    fn reset(&mut self);
}

/// Tracker error type.
#[derive(Debug, thiserror::Error)]
pub enum TrackerError {
    /// Lost the target (occlusion, out-of-frame, detector miss).
    #[error("target lost")]
    Lost,
    /// Numerical failure in the filter / manifold update.
    #[error("numerical failure: {0}")]
    Numerical(&'static str),
    /// Backend-specific failure.
    #[error("backend error: {0}")]
    Backend(String),
}
