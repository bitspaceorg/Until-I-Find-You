//! Integration tests for `PlaneTracker`.
//!
//! Same noise recipe as the other trackers: deterministic sin/cos
//! perturbations in the SE(3) tangent space, mean ≈ 0, the filter must
//! average them out.

use nalgebra::{Matrix6, Vector3, Vector6};
use uify_core::filters::ekf_manifold::Cov6;
use uify_core::manifolds::{LieGroup, se3::Se3};
use uify_core::{Timestamp, Tracker};
use uify_plane::{PlaneMeasurement, PlaneTracker};

const DT_NS: u64 = 33_333_333; // 30 Hz
const DT: f64 = 1.0 / 30.0;

fn truth_pose() -> Se3 {
    let t = Vector3::new(0.5, -0.3, 1.2);
    let omega = Vector3::new(0.1, 0.2, -0.05);
    Se3::exp(&Vector6::new(t.x, t.y, t.z, omega.x, omega.y, omega.z))
}

fn det_perturb(i: u32) -> Vector6<f64> {
    Vector6::new(
        (i as f64 * 0.12345).sin() * 0.01,
        (i as f64 * 0.67890).cos() * 0.01,
        (i as f64 * 0.31415).sin() * 0.01,
        (i as f64 * 0.27182).cos() * 0.005,
        (i as f64 * 0.41421).sin() * 0.005,
        (i as f64 * 0.57721).cos() * 0.005,
    )
}

fn make_tracker(initial: Se3) -> PlaneTracker {
    let initial_cov: Cov6 = Matrix6::identity() * 1.0;
    let process_noise: Cov6 = Matrix6::identity() * 1e-3;
    let measurement_noise: Cov6 = Matrix6::identity() * 1e-2;
    PlaneTracker::new(initial, initial_cov, process_noise, measurement_noise)
}

fn step_at(tracker: &mut PlaneTracker, i: u32, pose: Se3) -> Se3 {
    let m = PlaneMeasurement {
        t: Timestamp::from_nanos((i as u64) * DT_NS),
        pose,
    };
    tracker.step(m).unwrap().unwrap().value
}

#[test]
fn converges_to_stationary_truth() {
    let truth = truth_pose();
    let mut tracker = make_tracker(Se3::identity());

    let mut last = Se3::identity();
    for i in 0..200 {
        let measured = truth.compose(&Se3::exp(&det_perturb(i)));
        last = step_at(&mut tracker, i, measured);
    }

    let err = truth.minus(&last).norm();
    assert!(err < 0.02, "did not converge: ‖truth ⊖ estimate‖ = {err}");
}

#[test]
fn tracks_slowly_drifting_pose() {
    // Truth drifts at constant tangent velocity ω̇ in the rotation, ρ̇ in
    // translation. The filter must follow within a small lag.
    let drift = Vector6::new(0.005, 0.0, 0.0, 0.0, 0.0, 0.002);
    let mut tracker = make_tracker(Se3::identity());

    let mut last = Se3::identity();
    let mut truth_final = Se3::identity();
    for i in 0..200 {
        let truth_i = Se3::exp(&(drift * (i as f64 * DT)));
        let measured = truth_i.compose(&Se3::exp(&det_perturb(i)));
        last = step_at(&mut tracker, i, measured);
        truth_final = truth_i;
    }

    let err = truth_final.minus(&last).norm();
    assert!(err < 0.05, "drift tracking error: {err}");
}

#[test]
fn reset_restores_initial_pose() {
    let initial = truth_pose();
    let mut tracker = make_tracker(initial);

    // Drag the filter far away.
    let far = Se3::exp(&Vector6::new(10.0, 10.0, 10.0, 1.0, 1.0, 1.0));
    for i in 0..50 {
        step_at(&mut tracker, i, far);
    }

    tracker.reset();

    // After reset, an update at the initial pose should leave us close to it.
    let after = step_at(&mut tracker, 0, initial);
    let err = initial.minus(&after).norm();
    assert!(err < 0.05, "after reset+update at initial, err = {err}");
}
