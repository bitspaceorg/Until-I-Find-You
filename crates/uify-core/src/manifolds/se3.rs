//! SE(3): 3D rigid motion as `(rotation, translation)`.
//!
//! # Conventions
//! - Internal representation: [`So3`] for rotation, [`Vector3`] for
//!   translation. No 4×4 matrix is materialized in the hot path.
//! - Tangent space: `ξ = (ρ, ω) ∈ ℝ⁶` with translation tangent `ρ` first
//!   (components 0..3) and rotation tangent `ω` second (components 3..6).
//!   This matches Solà, Deray, Atchuthan, *A micro Lie theory for state
//!   estimation in robotics* (2021), which is also the reference cited in
//!   `filters/ekf_manifold.rs`.
//! - `exp((ρ, ω)) = (exp_so3(ω), V(ω) · ρ)` where `V` is the SO(3)
//!   left-Jacobian (a.k.a. *Q-matrix*).
//! - `log((R, t)) = (V⁻¹(ω) · t, log_so3(R))`.
//! - Right-trivialization for `⊕` / `⊖` is inherited from the `LieGroup`
//!   default methods.

use crate::manifolds::{LieGroup, so3::So3};
use nalgebra::{Matrix3, Vector3, Vector6};

/// Element of SE(3).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Se3 {
    rotation: So3,
    translation: Vector3<f64>,
}

impl Se3 {
    /// Construct from a rotation and a translation.
    pub fn from_parts(rotation: So3, translation: Vector3<f64>) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// Rotation component.
    pub fn rotation(&self) -> &So3 {
        &self.rotation
    }

    /// Translation component.
    pub fn translation(&self) -> &Vector3<f64> {
        &self.translation
    }
}

impl Default for Se3 {
    fn default() -> Self {
        <Self as LieGroup>::identity()
    }
}

/// Skew-symmetric matrix of `v` (the `[v]×` operator). Maps `ω ↦ v × ω`.
fn skew(v: &Vector3<f64>) -> Matrix3<f64> {
    Matrix3::new(0.0, -v.z, v.y, v.z, 0.0, -v.x, -v.y, v.x, 0.0)
}

/// SO(3) left-Jacobian `V(ω) = I + a·[ω]× + b·[ω]×²` where
/// `a = (1 - cos θ)/θ²`, `b = (θ - sin θ)/θ³`. Maps a translation tangent
/// `ρ` to the SE(3) translation `V(ω)·ρ`.
fn left_jacobian(omega: &Vector3<f64>) -> Matrix3<f64> {
    let theta_sq = omega.norm_squared();
    const SMALL: f64 = 1e-6;

    let (a, b) = if theta_sq < SMALL {
        // a = 1/2 − θ²/24 + θ⁴/720 − …
        // b = 1/6 − θ²/120 + θ⁴/5040 − …
        let t4 = theta_sq * theta_sq;
        (
            0.5 - theta_sq / 24.0 + t4 / 720.0,
            1.0 / 6.0 - theta_sq / 120.0 + t4 / 5040.0,
        )
    } else {
        let theta = theta_sq.sqrt();
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        (
            (1.0 - cos_t) / theta_sq,
            (theta - sin_t) / (theta_sq * theta),
        )
    };

    let omega_hat = skew(omega);
    let omega_hat_sq = omega_hat * omega_hat;
    Matrix3::identity() + a * omega_hat + b * omega_hat_sq
}

/// Inverse SO(3) left-Jacobian `V⁻¹(ω) = I − ½·[ω]× + c·[ω]×²` where
/// `c = 1/θ² − cot(θ/2)/(2θ)`.
fn left_jacobian_inv(omega: &Vector3<f64>) -> Matrix3<f64> {
    let theta_sq = omega.norm_squared();
    const SMALL: f64 = 1e-6;

    let c = if theta_sq < SMALL {
        // c = 1/12 + θ²/720 + θ⁴/30240 + …
        let t4 = theta_sq * theta_sq;
        1.0 / 12.0 + theta_sq / 720.0 + t4 / 30240.0
    } else {
        let theta = theta_sq.sqrt();
        let half = 0.5 * theta;
        // 1/θ² − cos(θ/2) / (2θ · sin(θ/2)).
        // Stable up to θ = π since sin(π/2) = 1.
        1.0 / theta_sq - half.cos() / (2.0 * theta * half.sin())
    };

    let omega_hat = skew(omega);
    let omega_hat_sq = omega_hat * omega_hat;
    Matrix3::identity() - 0.5 * omega_hat + c * omega_hat_sq
}

impl LieGroup for Se3 {
    type Tangent = Vector6<f64>;

    fn identity() -> Self {
        Self {
            rotation: So3::identity(),
            translation: Vector3::zeros(),
        }
    }

    fn compose(&self, other: &Self) -> Self {
        Self {
            rotation: self.rotation.compose(&other.rotation),
            translation: self.rotation.to_matrix() * other.translation + self.translation,
        }
    }

    fn inverse(&self) -> Self {
        let r_inv = self.rotation.inverse();
        let t_inv = -(r_inv.to_matrix() * self.translation);
        Self {
            rotation: r_inv,
            translation: t_inv,
        }
    }

    fn exp(xi: &Self::Tangent) -> Self {
        let rho = Vector3::new(xi[0], xi[1], xi[2]);
        let omega = Vector3::new(xi[3], xi[4], xi[5]);
        let rotation = So3::exp(&omega);
        let translation = left_jacobian(&omega) * rho;
        Self {
            rotation,
            translation,
        }
    }

    fn log(&self) -> Self::Tangent {
        let omega = self.rotation.log();
        let rho = left_jacobian_inv(&omega) * self.translation;
        Vector6::new(rho.x, rho.y, rho.z, omega.x, omega.y, omega.z)
    }
}
