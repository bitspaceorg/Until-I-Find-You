//! Classical linear Kalman filter, generic over state dimension. Measurement
//! dimension is method-level so a single filter instance can accept different
//! measurement types if needed.
//!
//! Predict / update both use the Joseph-form covariance update so `Σ` stays
//! symmetric positive-definite under finite-precision arithmetic. Joseph
//! costs an extra matrix multiply over the textbook `(I − KH)Σ` form; for
//! state dimensions used by trackers (≤ 12) the difference is negligible
//! and the numerical guarantees are worth it.

use nalgebra::{SMatrix, SVector};

/// Linear Kalman filter on `ℝᴺ`.
#[derive(Clone, Debug)]
pub struct KalmanFilter<const N: usize> {
    state: SVector<f64, N>,
    cov: SMatrix<f64, N, N>,
}

impl<const N: usize> KalmanFilter<N> {
    /// New filter at `state` with covariance `cov`.
    pub fn new(state: SVector<f64, N>, cov: SMatrix<f64, N, N>) -> Self {
        Self { state, cov }
    }

    /// Current mean.
    pub fn state(&self) -> &SVector<f64, N> {
        &self.state
    }

    /// Current covariance.
    pub fn covariance(&self) -> &SMatrix<f64, N, N> {
        &self.cov
    }

    /// Linear predict: `x ← F · x`, `Σ ← F · Σ · Fᵀ + Q`.
    pub fn predict(&mut self, f: &SMatrix<f64, N, N>, q: &SMatrix<f64, N, N>) {
        self.state = f * self.state;
        self.cov = f * self.cov * f.transpose() + q;
    }

    /// Update against measurement `z` with model `H` and measurement noise
    /// `R`. Joseph form for the covariance.
    pub fn update<const M: usize>(
        &mut self,
        h: &SMatrix<f64, M, N>,
        z: &SVector<f64, M>,
        r: &SMatrix<f64, M, M>,
    ) {
        let innov = z - h * self.state;
        let s = h * self.cov * h.transpose() + r;
        let s_inv = s
            .try_inverse()
            .expect("Kalman innovation covariance is singular — check that R is positive-definite");
        let k = self.cov * h.transpose() * s_inv;
        self.state += k * innov;

        let i_kh = SMatrix::<f64, N, N>::identity() - k * h;
        self.cov = i_kh * self.cov * i_kh.transpose() + k * r * k.transpose();
    }
}
