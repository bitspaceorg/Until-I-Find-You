//! SE(2): planar rigid motion as `(rotation, translation)`.
//!
//! # Conventions
//! - Storage: `So2` (1-DOF rotation) + `Vector2<f64>` (translation).
//! - Tangent: `ξ = (ρ, ω) ∈ ℝ³` with translation tangent `ρ` first
//!   (components 0..2) and rotation tangent `ω` last (component 2). Same
//!   ordering as SE(3).
//! - `exp((ρ, ω)) = (exp_so2(ω), V(ω) · ρ)` with the SO(2) left-Jacobian
//!   `V(ω) = a·I + b·J`, where `a = sin(ω)/ω`, `b = (1−cos ω)/ω`,
//!   `J = [[0, −1], [1, 0]]`.
//! - `log((R, t)) = (V⁻¹(ω) · t, log_so2(R))` with
//!   `V⁻¹(ω) = α·I − β·J`, `α = (ω/2)·cot(ω/2)`, `β = ω/2`.
//! - Right-trivialization for `⊕` / `⊖` is inherited from `LieGroup`'s
//!   default methods.

use crate::manifolds::{LieGroup, so2::So2};
use nalgebra::{Vector2, Vector3};

/// Element of SE(2).
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Se2 {
    rotation: So2,
    translation: Vector2<f64>,
}

impl Se2 {
    /// Construct from a rotation and a translation.
    pub fn from_parts(rotation: So2, translation: Vector2<f64>) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// Rotation component.
    pub fn rotation(&self) -> &So2 {
        &self.rotation
    }

    /// Translation component.
    pub fn translation(&self) -> &Vector2<f64> {
        &self.translation
    }
}

impl LieGroup for Se2 {
    type Tangent = Vector3<f64>;

    fn identity() -> Self {
        Self {
            rotation: So2::identity(),
            translation: Vector2::zeros(),
        }
    }

    fn compose(&self, other: &Self) -> Self {
        Self {
            rotation: self.rotation.compose(&other.rotation),
            translation: self.rotation.rotate(&other.translation) + self.translation,
        }
    }

    fn inverse(&self) -> Self {
        let r_inv = self.rotation.inverse();
        let t_inv = -r_inv.rotate(&self.translation);
        Self {
            rotation: r_inv,
            translation: t_inv,
        }
    }

    fn exp(xi: &Self::Tangent) -> Self {
        let rho = Vector2::new(xi.x, xi.y);
        let omega = xi.z;

        let (a, b) = jacobian_coefficients(omega);
        // V(ω) · ρ = a·ρ + b·J·ρ = (a·ρx − b·ρy, a·ρy + b·ρx)
        let translation = Vector2::new(a * rho.x - b * rho.y, a * rho.y + b * rho.x);

        Self {
            rotation: So2::exp(&omega),
            translation,
        }
    }

    fn log(&self) -> Self::Tangent {
        let omega = self.rotation.log();
        let (alpha, beta) = inverse_jacobian_coefficients(omega);
        let t = self.translation;
        // V⁻¹(ω) · t = α·t − β·J·t = (α·tx + β·ty, α·ty − β·tx)
        let rho = Vector2::new(alpha * t.x + beta * t.y, alpha * t.y - beta * t.x);
        Vector3::new(rho.x, rho.y, omega)
    }
}

/// Coefficients of the SO(2) left-Jacobian: `V(ω) = a·I + b·J`.
///
/// `a = sin(ω)/ω`, `b = (1 − cos ω)/ω`, with Taylor cutover at small ω.
fn jacobian_coefficients(omega: f64) -> (f64, f64) {
    const SMALL: f64 = 1e-6;
    let omega_sq = omega * omega;
    if omega_sq < SMALL {
        // a = 1 − ω²/6 + ω⁴/120 − …
        // b = ω/2 − ω³/24 + ω⁵/720 − …
        let a = 1.0 - omega_sq / 6.0 + omega_sq * omega_sq / 120.0;
        let b = omega * (0.5 - omega_sq / 24.0 + omega_sq * omega_sq / 720.0);
        (a, b)
    } else {
        (omega.sin() / omega, (1.0 - omega.cos()) / omega)
    }
}

/// Coefficients of `V⁻¹(ω) = α·I − β·J`.
///
/// `α = (ω/2) · cot(ω/2)`, `β = ω/2`. The diagonal coefficient α has a 0/0
/// limit at ω → 0 (`cot(ω/2)` blows up but the leading `ω/2` cancels it);
/// expand by Taylor below the cutover.
fn inverse_jacobian_coefficients(omega: f64) -> (f64, f64) {
    const SMALL: f64 = 1e-6;
    let omega_sq = omega * omega;
    let alpha = if omega_sq < SMALL {
        // (x · cot(x)) at x = ω/2: 1 − x²/3 − x⁴/45 − …
        // ⇒ α = 1 − ω²/12 − ω⁴/720 − …
        1.0 - omega_sq / 12.0 - omega_sq * omega_sq / 720.0
    } else {
        let half = 0.5 * omega;
        half * half.cos() / half.sin()
    };
    let beta = 0.5 * omega;
    (alpha, beta)
}
