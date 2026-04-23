//! Recursive filters for tracking.
//!
//! All filters are generic over a state manifold. For Euclidean states this
//! collapses to a standard Kalman filter; for SE(3) etc. the same code paths
//! run in the tangent space via the `LieGroup` trait.

pub mod ekf_manifold;
pub mod kalman;
pub mod rts_smoother;
