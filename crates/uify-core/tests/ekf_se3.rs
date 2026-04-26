//! Integration tests for the SE(3) EKF.
//!
//! These tests exercise the predict/update cycle with hand-crafted
//! deterministic inputs, no RNG. The filter is small enough that property
//! tests are overkill; what we want is unambiguous numerical assertions.

use nalgebra::{Matrix6, Vector3, Vector6};
use uify_core::filters::ekf_manifold::Ekf3D;
use uify_core::manifolds::{LieGroup, se3::Se3};

fn pose(rho: Vector3<f64>, omega: Vector3<f64>) -> Se3 {
    Se3::exp(&Vector6::new(
        rho.x, rho.y, rho.z, omega.x, omega.y, omega.z,
    ))
}

fn distance(a: &Se3, b: &Se3) -> f64 {
    a.minus(b).norm()
}

/// If the prior already equals the measurement, an update is a no-op on the
/// mean. Covariance must shrink — `(I − K) · Σ · (I − K)ᵀ + K R Kᵀ` with
/// `K = Σ / (Σ + R)` is strictly less than `Σ` for `R > 0`.
#[test]
fn update_at_truth_does_not_move_mean() {
    let truth = pose(Vector3::new(1.0, 2.0, 3.0), Vector3::new(0.1, -0.2, 0.05));
    let mut ekf = Ekf3D::new(truth, Matrix6::identity());

    let r = Matrix6::identity() * 0.01;
    let initial_trace = ekf.covariance().trace();

    ekf.update(&truth, &r);

    assert!(
        distance(ekf.state(), &truth) < 1e-12,
        "mean should not drift when measurement equals prior; got {}",
        distance(ekf.state(), &truth)
    );
    assert!(
        ekf.covariance().trace() < initial_trace,
        "covariance trace should shrink: before={}, after={}",
        initial_trace,
        ekf.covariance().trace()
    );
}

/// Repeated noise-free measurements at `z` should drive the estimate to `z`
/// and shrink the covariance toward zero.
#[test]
fn converges_to_noise_free_measurement() {
    let truth = pose(Vector3::new(0.5, 0.0, -0.3), Vector3::new(0.0, 0.2, 0.0));
    // Initial estimate offset from truth.
    let initial = pose(Vector3::new(2.0, 1.5, 0.0), Vector3::new(0.3, 0.0, -0.4));
    let mut ekf = Ekf3D::new(initial, Matrix6::identity() * 4.0);

    // Tiny R so the filter trusts the measurement; small Q so it doesn't
    // re-inflate between steps.
    let q = Matrix6::identity() * 1e-6;
    let r = Matrix6::identity() * 1e-3;
    let dt = 0.01;

    for _ in 0..200 {
        ekf.predict(&q, dt);
        ekf.update(&truth, &r);
    }

    let err = distance(ekf.state(), &truth);
    assert!(err < 1e-3, "did not converge: ‖estimate ⊖ truth‖ = {err}");
    assert!(
        ekf.covariance().trace() < 0.1,
        "covariance did not shrink: trace = {}",
        ekf.covariance().trace()
    );
}

/// A single update with a measurement halfway between prior and truth, when
/// `Σ = R`, should give a state exactly halfway between prior and measurement.
/// This is the textbook scalar-Kalman gain `K = Σ / (Σ + R) = 1/2`.
#[test]
fn balanced_kalman_gain_yields_midpoint() {
    let prior = Se3::identity();
    let measurement = pose(Vector3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    let mut ekf = Ekf3D::new(prior, Matrix6::identity());
    ekf.update(&measurement, &Matrix6::identity());

    let expected = pose(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    let err = distance(ekf.state(), &expected);
    assert!(
        err < 1e-9,
        "midpoint mismatch: ‖estimate ⊖ expected‖ = {err}"
    );
}

/// Covariance must remain symmetric positive-definite after an update.
/// Joseph-form is supposed to enforce this even when `K H` is far from `I`.
#[test]
fn covariance_stays_spd_after_update() {
    let mut ekf = Ekf3D::new(Se3::identity(), Matrix6::identity() * 1e3);
    let z = pose(Vector3::new(0.5, 0.5, 0.5), Vector3::new(0.05, 0.05, 0.05));
    let r = Matrix6::identity();

    for _ in 0..50 {
        ekf.update(&z, &r);
    }

    let cov = ekf.covariance();
    let asymmetry = (cov - cov.transpose()).abs().max();
    assert!(
        asymmetry < 1e-12,
        "covariance not symmetric: max(|Σ - Σᵀ|) = {asymmetry}"
    );

    // Cholesky test for positive-definiteness.
    assert!(
        cov.cholesky().is_some(),
        "covariance is not positive-definite"
    );
}

/// Under predict only, the covariance must grow monotonically.
#[test]
fn predict_only_inflates_covariance() {
    let mut ekf = Ekf3D::new(Se3::identity(), Matrix6::identity());
    let q = Matrix6::identity();
    let prev_trace = ekf.covariance().trace();
    ekf.predict(&q, 1.0);
    assert!(ekf.covariance().trace() > prev_trace);
}
