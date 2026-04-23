//! Single-point tracker (ball tracking, generic 2D/3D point).
//!
//! Geometry: `Point2<f32>` or `Point3<f32>`. State: constant-velocity model
//! in the chosen dimension; the filter is a plain linear KF (Euclidean, so
//! no Lie-group machinery needed here).

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

/// Placeholder — implementation lands in step 2 of the roadmap (see
/// `docs/development.mdx`).
pub struct PointTracker;
