//! Integration tests for `PointTracker2D`.
//!
//! Deterministic, RNG-free: noise is a closed-form sin/cos sequence with
//! mean ≈ 0 over the test horizon, so the filter must average it out for
//! the assertions to hold.

use nalgebra::{Matrix2, Matrix4, Vector2, Vector4};
use uify_core::Tracker;
use uify_point::{PointMeasurement, PointTracker2D};

const DT_NS: u64 = 10_000_000; // 10 ms = 100 Hz
const DT: f64 = 1e-2;

fn det_noise(i: u32) -> Vector2<f64> {
    // Two incommensurate frequencies → low-correlation, mean ≈ 0 over many samples.
    let a = (i as f64 * 0.12345).sin() * 0.02;
    let b = (i as f64 * 0.67890).cos() * 0.02;
    Vector2::new(a, b)
}

fn make_tracker(initial_pos: Vector2<f64>) -> PointTracker2D {
    let initial_state = Vector4::new(initial_pos.x, initial_pos.y, 0.0, 0.0);
    let initial_cov = Matrix4::identity() * 1.0;
    // Process noise: small per-second velocity drift.
    let process_noise = {
        let mut q = Matrix4::zeros();
        q[(2, 2)] = 0.1;
        q[(3, 3)] = 0.1;
        q
    };
    let measurement_noise = Matrix2::identity() * 0.01;
    PointTracker2D::new(initial_state, initial_cov, process_noise, measurement_noise)
}

fn step_at(tracker: &mut PointTracker2D, i: u32, position: Vector2<f64>) -> Vector2<f64> {
    let t = uify_core::Timestamp::from_nanos((i as u64) * DT_NS);
    let m = PointMeasurement { t, position };
    let sample = tracker.step(m).unwrap().unwrap();
    sample.value
}

/// Stationary truth + zero-mean noise: filter should converge to truth.
#[test]
fn converges_to_stationary_truth() {
    let truth = Vector2::new(5.0, 3.0);
    let mut tracker = make_tracker(Vector2::zeros());

    let mut last = Vector2::zeros();
    for i in 0..200 {
        last = step_at(&mut tracker, i, truth + det_noise(i));
    }

    let err = (last - truth).norm();
    assert!(err < 0.05, "did not converge: ‖estimate − truth‖ = {err}");
    let v = tracker.velocity();
    assert!(v.norm() < 0.1, "velocity should be ≈ 0, got {v}");
}

/// Linearly moving truth: filter must recover both position and velocity.
#[test]
fn tracks_constant_velocity() {
    let p0 = Vector2::new(0.0, 0.0);
    let true_v = Vector2::new(2.5, -1.0);
    let mut tracker = make_tracker(p0);

    let mut last = Vector2::zeros();
    for i in 0..200 {
        let truth_t = p0 + true_v * (i as f64 * DT);
        last = step_at(&mut tracker, i, truth_t + det_noise(i));
    }

    let truth_final = p0 + true_v * (199.0 * DT);
    let pos_err = (last - truth_final).norm();
    let vel_err = (tracker.velocity() - true_v).norm();
    assert!(pos_err < 0.1, "position error: {pos_err}");
    assert!(vel_err < 0.2, "velocity error: {vel_err}");
}

/// `reset()` must roll the filter back to the constructor's initial state.
#[test]
fn reset_restores_initial_state() {
    let p0 = Vector2::new(0.0, 0.0);
    let mut tracker = make_tracker(p0);

    for i in 0..50 {
        step_at(&mut tracker, i, Vector2::new(10.0, 10.0));
    }

    tracker.reset();

    // First post-reset step should not "see" any prior history: the dt-based
    // predict is skipped because `last_t` is None.
    let after = step_at(&mut tracker, 0, Vector2::new(0.0, 0.0));
    assert!(
        after.norm() < 0.1,
        "after reset+update at origin, expected ≈ 0, got {after}"
    );
    assert!(
        tracker.velocity().norm() < 0.1,
        "velocity after reset should be ≈ 0"
    );
}
