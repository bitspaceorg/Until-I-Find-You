//! Planar tracking — the "virtual camera body" tracker.
//!
//! Pipeline: feature detection (SuperPoint) → description + matching
//! (LightGlue) → homography estimation (RANSAC + Levenberg-Marquardt in
//! SL(3)) → PnP lift to SE(3) given camera intrinsics.
//!
//! Output: `Sample<Se3, _>` with uncertainty propagated through PnP.
//! TODO: implement.

#![forbid(unsafe_op_in_unsafe_fn)]

/// Placeholder.
pub struct PlaneTracker;
