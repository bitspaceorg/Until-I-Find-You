//! EKF on a Lie group. State lives on the manifold; covariance lives in the
//! tangent space at the current mean (right-trivialized convention).
//!
//! Reference: Solà, Deray, Atchuthan, *A micro Lie theory for state
//! estimation in robotics* (2021).
//!
//! # Scope of this first cut
//!
//! - State manifold is hardcoded to [`Se3`] with a 6×6 tangent-space
//!   covariance (`Matrix6<f64>`).
//! - Process model is a **random walk**: `predict` does not move the mean;
//!   it only inflates the covariance by `Q · dt`. A constant-velocity
//!   variant will lift the state to `(T, ξ) ∈ SE(3) × ℝ⁶`.
//! - Measurement model is a **direct pose observation**: `H = I₆`,
//!   `innovation = z ⊖ state`.
//! - Covariance reset (`Σ ← J_r⁻¹(δ) · Σ · J_r⁻¹(δ)ᵀ`) is currently
//!   approximated as identity. Residual is `O(‖δ‖²)`; safe for small
//!   updates, will be wired in once SE(3) right-Jacobians land.
//!
//! Generalization to `EkfManifold<G, P, M>` (process + measurement model
//! traits) is deferred until a second tracker exposes the right boundary —
//! trait shapes designed against a single client tend to be wrong.

use crate::manifolds::{LieGroup, se3::Se3};
use nalgebra::Matrix6;

/// SE(3) tangent-space covariance.
pub type Cov6 = Matrix6<f64>;

/// EKF on SE(3) with a random-walk process model and direct pose measurements.
#[derive(Clone, Debug)]
pub struct Ekf3D {
    state: Se3,
    cov: Cov6,
}

impl Ekf3D {
    /// New filter at `state` with initial tangent-space covariance `cov`.
    pub fn new(state: Se3, cov: Cov6) -> Self {
        Self { state, cov }
    }

    /// Current mean estimate.
    pub fn state(&self) -> &Se3 {
        &self.state
    }

    /// Current covariance, expressed in the tangent space at `state()`.
    pub fn covariance(&self) -> &Cov6 {
        &self.cov
    }

    /// Random-walk predict: `Σ ← Σ + Q · dt`. Mean is unchanged.
    pub fn predict(&mut self, q: &Cov6, dt: f64) {
        self.cov += q * dt;
    }

    /// Direct-pose update against measurement `z` with covariance `r`.
    /// Uses the Joseph form so `Σ` stays symmetric and positive-definite
    /// even with finite-precision arithmetic.
    pub fn update(&mut self, z: &Se3, r: &Cov6) {
        // Innovation in the tangent space at the current mean:
        //   ε = z ⊖ state = log(state⁻¹ ∘ z)
        let innov = self.state.minus(z);

        // H = I₆ (direct measurement). S = H Σ Hᵀ + R = Σ + R. K = Σ S⁻¹.
        let s = self.cov + r;
        let s_inv = s
            .try_inverse()
            .expect("EKF innovation covariance is singular — check that R is positive-definite");
        let k = self.cov * s_inv;

        // Apply the correction in tangent space.
        let delta = k * innov;
        self.state = self.state.plus(&delta);

        // Joseph-form covariance update: (I − K H) Σ (I − K H)ᵀ + K R Kᵀ.
        let i_kh = Cov6::identity() - k;
        self.cov = i_kh * self.cov * i_kh.transpose() + k * r * k.transpose();
    }
}
