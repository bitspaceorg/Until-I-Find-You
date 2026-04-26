//! SO(3): 3D rotations as unit quaternions.
//!
//! # Conventions
//! - Tangent space is a rotation vector `ω ∈ ℝ³`. Direction is the rotation
//!   axis, magnitude is the angle in radians.
//! - `exp(ω)` produces the unit quaternion `[cos(θ/2), sin(θ/2)·ω̂]` where
//!   `θ = ‖ω‖`.
//! - `log(q)` returns the rotation vector in `[-π, π] · axis`. The canonical
//!   hemisphere `w ≥ 0` is forced before extraction so that two quaternions
//!   representing the same rotation (`q` and `-q`) produce the same log.
//! - Right-trivialization for `⊕` / `⊖` is inherited from `LieGroup`'s default
//!   methods: `x ⊕ ξ = x ∘ exp(ξ)`, `y ⊖ x = log(x⁻¹ ∘ y)`.

use crate::manifolds::LieGroup;
use nalgebra::{Matrix3, Quaternion, UnitQuaternion, Vector3};

/// Element of SO(3), stored as a unit quaternion.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct So3(UnitQuaternion<f64>);

impl So3 {
    /// Wrap an existing unit quaternion.
    pub fn from_quaternion(q: UnitQuaternion<f64>) -> Self {
        Self(q)
    }

    /// Underlying unit quaternion.
    pub fn as_quaternion(&self) -> &UnitQuaternion<f64> {
        &self.0
    }

    /// Equivalent 3×3 rotation matrix.
    pub fn to_matrix(&self) -> Matrix3<f64> {
        self.0.to_rotation_matrix().into_inner()
    }
}

impl Default for So3 {
    fn default() -> Self {
        <Self as LieGroup>::identity()
    }
}

impl LieGroup for So3 {
    type Tangent = Vector3<f64>;

    fn identity() -> Self {
        Self(UnitQuaternion::identity())
    }

    fn compose(&self, other: &Self) -> Self {
        Self(self.0 * other.0)
    }

    fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }

    fn exp(xi: &Self::Tangent) -> Self {
        let theta_sq = xi.norm_squared();

        // Taylor cutover near zero angle to avoid 0/0 in sin(θ/2)/θ.
        // Quartic terms are kept so the residual after the switch is O(θ⁶),
        // well below f64 round-off for θ < 1e-3.
        const SMALL: f64 = 1e-6;
        let (w, sin_half_over_theta) = if theta_sq < SMALL {
            // cos(θ/2)   = 1 - θ²/8 + θ⁴/384 - …
            // sin(θ/2)/θ = 1/2 - θ²/48 + θ⁴/3840 - …
            let t4 = theta_sq * theta_sq;
            let w = 1.0 - theta_sq / 8.0 + t4 / 384.0;
            let s = 0.5 - theta_sq / 48.0 + t4 / 3840.0;
            (w, s)
        } else {
            let theta = theta_sq.sqrt();
            let half = 0.5 * theta;
            (half.cos(), half.sin() / theta)
        };

        let q = Quaternion::new(
            w,
            sin_half_over_theta * xi.x,
            sin_half_over_theta * xi.y,
            sin_half_over_theta * xi.z,
        );
        Self(UnitQuaternion::from_quaternion(q))
    }

    fn log(&self) -> Self::Tangent {
        // Force canonical hemisphere (w ≥ 0). Quaternions q and -q represent
        // the same rotation; choosing w ≥ 0 gives the shorter geodesic and
        // makes the logarithm a function rather than a relation.
        let raw = self.0.into_inner();
        let (w, v) = if raw.scalar() < 0.0 {
            (-raw.scalar(), -raw.imag())
        } else {
            (raw.scalar(), raw.imag())
        };

        // ω = 2 · atan2(‖v‖, w) · v / ‖v‖.
        let v_norm_sq = v.norm_squared();
        const SMALL: f64 = 1e-6;
        let factor = if v_norm_sq < SMALL {
            // For small ‖v‖ (so w ≈ 1):
            //   atan2(s, w)/s ≈ 1/w − s²/(3 w³) + …
            // hence 2 · atan2(s, w)/s ≈ 2/w · (1 − s²/(3 w²)).
            2.0 / w * (1.0 - v_norm_sq / (3.0 * w * w))
        } else {
            let v_norm = v_norm_sq.sqrt();
            2.0 * v_norm.atan2(w) / v_norm
        };

        v * factor
    }
}
