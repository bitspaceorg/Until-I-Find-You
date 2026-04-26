//! Property tests for every Lie group in `uify_core::manifolds`.
//!
//! Each new group MUST add a `mod <group>` here that exercises the four
//! identities documented in `manifolds/mod.rs`. Tolerances target double
//! precision: `1e-9` is comfortable for SO(3); SE(3) / SL(3) may need 1e-8.

use nalgebra::{Vector3, Vector6};
use proptest::prelude::*;
use uify_core::manifolds::{LieGroup, se2::Se2, se3::Se3, so2::So2, so3::So3};

const TOL: f64 = 1e-9;
// SE(2) and SE(3) tolerances are looser: V(ω) and V⁻¹(ω) introduce extra
// arithmetic, and the translation magnitude is unbounded in the strategies
// below, so absolute error scales with translation size.
const TOL_SE2: f64 = 1e-7;
const TOL_SE3: f64 = 1e-7;

/// Random SO(3) element. Constructed via `exp` on a random tangent vector
/// drawn from the disk of radius `< π` so we land somewhere in the group
/// without sampling the boundary where `log` is multi-valued.
fn so3_element() -> impl Strategy<Value = So3> {
    so3_tangent_under_pi().prop_map(|xi| So3::exp(&xi))
}

/// Tangent vector with magnitude strictly less than π. Avoids the antipodal
/// boundary where `log(exp(ξ))` would wrap.
fn so3_tangent_under_pi() -> impl Strategy<Value = Vector3<f64>> {
    (
        -1.0f64..=1.0f64,
        -1.0f64..=1.0f64,
        -1.0f64..=1.0f64,
        0.0f64..(std::f64::consts::PI - 1e-3),
    )
        .prop_map(|(x, y, z, target_norm)| {
            let v = Vector3::new(x, y, z);
            let n = v.norm();
            if n < 1e-12 {
                Vector3::zeros()
            } else {
                v * (target_norm / n)
            }
        })
}

/// Geodesic distance between two SO(3) elements: ‖log(a⁻¹ ∘ b)‖.
fn distance(a: &So3, b: &So3) -> f64 {
    a.minus(b).norm()
}

proptest! {
    /// `exp(log(x)) ≈ x`.
    #[test]
    fn so3_exp_log_roundtrip(g in so3_element()) {
        let xi = g.log();
        let g2 = So3::exp(&xi);
        prop_assert!(distance(&g, &g2) < TOL, "distance = {}", distance(&g, &g2));
    }

    /// `log(exp(ξ)) ≈ ξ`. Restricted to ‖ξ‖ < π — beyond that the log
    /// returns the equivalent rotation in the canonical hemisphere, which
    /// is a different (but valid) tangent vector.
    #[test]
    fn so3_log_exp_roundtrip(xi in so3_tangent_under_pi()) {
        let g = So3::exp(&xi);
        let xi2 = g.log();
        prop_assert!((xi - xi2).norm() < TOL, "‖xi - xi2‖ = {}", (xi - xi2).norm());
    }

    /// `x ⊖ x ≈ 0`.
    #[test]
    fn so3_minus_self_is_zero(g in so3_element()) {
        let xi = g.minus(&g);
        prop_assert!(xi.norm() < TOL);
    }

    /// `x ⊕ (y ⊖ x) ≈ y`. With the trait's `minus(&self, other) = log(self⁻¹ ∘ other)`
    /// convention this reads as `x.plus(&x.minus(&y)) ≈ y`.
    #[test]
    fn so3_plus_minus_inverse(x in so3_element(), y in so3_element()) {
        let delta = x.minus(&y);
        let recovered = x.plus(&delta);
        prop_assert!(distance(&recovered, &y) < TOL, "distance = {}", distance(&recovered, &y));
    }
}

/// SE(3) tangent: rotation magnitude < π (so log is single-valued), translation
/// magnitude bounded so absolute test tolerance has meaning.
fn se3_tangent_under_pi() -> impl Strategy<Value = Vector6<f64>> {
    (
        -10.0f64..=10.0f64,
        -10.0f64..=10.0f64,
        -10.0f64..=10.0f64,
        so3_tangent_under_pi(),
    )
        .prop_map(|(rx, ry, rz, omega)| Vector6::new(rx, ry, rz, omega.x, omega.y, omega.z))
}

fn se3_element() -> impl Strategy<Value = Se3> {
    se3_tangent_under_pi().prop_map(|xi| Se3::exp(&xi))
}

fn se3_distance(a: &Se3, b: &Se3) -> f64 {
    a.minus(b).norm()
}

proptest! {
    #[test]
    fn se3_exp_log_roundtrip(g in se3_element()) {
        let xi = g.log();
        let g2 = Se3::exp(&xi);
        prop_assert!(se3_distance(&g, &g2) < TOL_SE3, "distance = {}", se3_distance(&g, &g2));
    }

    #[test]
    fn se3_log_exp_roundtrip(xi in se3_tangent_under_pi()) {
        let g = Se3::exp(&xi);
        let xi2 = g.log();
        let err = (xi - xi2).norm();
        prop_assert!(err < TOL_SE3, "‖xi − xi2‖ = {}", err);
    }

    #[test]
    fn se3_minus_self_is_zero(g in se3_element()) {
        let xi = g.minus(&g);
        prop_assert!(xi.norm() < TOL_SE3);
    }

    #[test]
    fn se3_plus_minus_inverse(x in se3_element(), y in se3_element()) {
        let delta = x.minus(&y);
        let recovered = x.plus(&delta);
        prop_assert!(se3_distance(&recovered, &y) < TOL_SE3, "distance = {}", se3_distance(&recovered, &y));
    }
}

/// SO(2) tangent: scalar in `(-π+ε, π−ε)`. The boundary is excluded because
/// `log` canonicalizes to `(-π, π]`, so a tangent of exactly `±π` would
/// round-trip to its canonical sibling — a different (but valid) value.
fn so2_tangent_under_pi() -> impl Strategy<Value = f64> {
    let bound = std::f64::consts::PI - 1e-3;
    -bound..=bound
}

fn so2_element() -> impl Strategy<Value = So2> {
    so2_tangent_under_pi().prop_map(|omega| So2::exp(&omega))
}

fn so2_distance(a: &So2, b: &So2) -> f64 {
    a.minus(b).abs()
}

proptest! {
    #[test]
    fn so2_exp_log_roundtrip(g in so2_element()) {
        let omega = g.log();
        let g2 = So2::exp(&omega);
        prop_assert!(so2_distance(&g, &g2) < TOL, "distance = {}", so2_distance(&g, &g2));
    }

    #[test]
    fn so2_log_exp_roundtrip(omega in so2_tangent_under_pi()) {
        let g = So2::exp(&omega);
        let omega2 = g.log();
        prop_assert!((omega - omega2).abs() < TOL, "Δ = {}", (omega - omega2).abs());
    }

    #[test]
    fn so2_minus_self_is_zero(g in so2_element()) {
        let omega = g.minus(&g);
        prop_assert!(omega.abs() < TOL);
    }

    #[test]
    fn so2_plus_minus_inverse(x in so2_element(), y in so2_element()) {
        let delta = x.minus(&y);
        let recovered = x.plus(&delta);
        prop_assert!(so2_distance(&recovered, &y) < TOL, "distance = {}", so2_distance(&recovered, &y));
    }
}

fn se2_tangent_under_pi() -> impl Strategy<Value = Vector3<f64>> {
    (
        -10.0f64..=10.0f64,
        -10.0f64..=10.0f64,
        so2_tangent_under_pi(),
    )
        .prop_map(|(rx, ry, omega)| Vector3::new(rx, ry, omega))
}

fn se2_element() -> impl Strategy<Value = Se2> {
    se2_tangent_under_pi().prop_map(|xi| Se2::exp(&xi))
}

fn se2_distance(a: &Se2, b: &Se2) -> f64 {
    a.minus(b).norm()
}

proptest! {
    #[test]
    fn se2_exp_log_roundtrip(g in se2_element()) {
        let xi = g.log();
        let g2 = Se2::exp(&xi);
        prop_assert!(se2_distance(&g, &g2) < TOL_SE2, "distance = {}", se2_distance(&g, &g2));
    }

    #[test]
    fn se2_log_exp_roundtrip(xi in se2_tangent_under_pi()) {
        let g = Se2::exp(&xi);
        let xi2 = g.log();
        let err = (xi - xi2).norm();
        prop_assert!(err < TOL_SE2, "‖xi − xi2‖ = {}", err);
    }

    #[test]
    fn se2_minus_self_is_zero(g in se2_element()) {
        let xi = g.minus(&g);
        prop_assert!(xi.norm() < TOL_SE2);
    }

    #[test]
    fn se2_plus_minus_inverse(x in se2_element(), y in se2_element()) {
        let delta = x.minus(&y);
        let recovered = x.plus(&delta);
        prop_assert!(se2_distance(&recovered, &y) < TOL_SE2, "distance = {}", se2_distance(&recovered, &y));
    }
}
