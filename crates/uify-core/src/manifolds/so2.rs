//! SO(2): planar rotation, stored as a raw angle in radians.
//!
//! # Conventions
//! - Internal storage is a `f64` angle. It can grow without bound under
//!   composition; `log` canonicalizes to `(-π, π]`.
//! - `exp(ω) = So2(ω)` and `log(So2(θ)) = canonicalize(θ)`. There is no
//!   small-angle Taylor expansion needed: SO(2) is one-parameter.

use crate::manifolds::LieGroup;
use nalgebra::Vector2;

/// Element of SO(2), stored as a raw angle (radians).
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct So2(f64);

impl So2 {
    /// Construct from an angle (radians, any magnitude).
    pub fn from_angle(theta: f64) -> Self {
        Self(theta)
    }

    /// Raw stored angle (un-normalized).
    pub fn angle(&self) -> f64 {
        self.0
    }

    /// Rotate a 2-vector by this rotation.
    pub fn rotate(&self, v: &Vector2<f64>) -> Vector2<f64> {
        let c = self.0.cos();
        let s = self.0.sin();
        Vector2::new(c * v.x - s * v.y, s * v.x + c * v.y)
    }
}

/// Canonicalize an angle to `(-π, π]`.
fn wrap(theta: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut t = theta % two_pi;
    if t > std::f64::consts::PI {
        t -= two_pi;
    } else if t <= -std::f64::consts::PI {
        t += two_pi;
    }
    t
}

impl LieGroup for So2 {
    /// SO(2)'s tangent is one-dimensional. Carrying it as `f64` (rather than
    /// `Vector1<f64>`) keeps callers simpler.
    type Tangent = f64;

    fn identity() -> Self {
        Self(0.0)
    }

    fn compose(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }

    fn inverse(&self) -> Self {
        Self(-self.0)
    }

    fn exp(omega: &f64) -> Self {
        Self(*omega)
    }

    fn log(&self) -> f64 {
        wrap(self.0)
    }
}
