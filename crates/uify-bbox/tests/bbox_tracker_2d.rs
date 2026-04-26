//! Integration tests for `BboxTracker2D`.
//!
//! Same recipe as the point-tracker tests: deterministic sin/cos noise, no
//! RNG, assertions framed around mean-zero noise averaging out.

use nalgebra::{Matrix4, Matrix6, Vector2, Vector6};
use uify_bbox::{Bbox, BboxMeasurement, BboxTracker2D};
use uify_core::{Timestamp, Tracker};

const DT_NS: u64 = 10_000_000;
const DT: f64 = 1e-2;

fn det_noise(i: u32) -> [f64; 4] {
    [
        (i as f64 * 0.12345).sin() * 0.05,
        (i as f64 * 0.67890).cos() * 0.05,
        (i as f64 * 0.31415).sin() * 0.05,
        (i as f64 * 0.27182).cos() * 0.05,
    ]
}

fn make_tracker(seed: Bbox) -> BboxTracker2D {
    let initial_state = Vector6::new(seed.cx, seed.cy, seed.w, seed.h, 0.0, 0.0);
    let initial_cov = Matrix6::identity() * 1.0;
    let mut process_noise = Matrix6::zeros();
    process_noise[(2, 2)] = 0.01; // w random walk
    process_noise[(3, 3)] = 0.01; // h random walk
    process_noise[(4, 4)] = 0.5; // vcx process noise
    process_noise[(5, 5)] = 0.5; // vcy process noise
    let measurement_noise = Matrix4::identity() * 0.01;
    BboxTracker2D::new(initial_state, initial_cov, process_noise, measurement_noise)
}

fn step_at(tracker: &mut BboxTracker2D, i: u32, bbox: Bbox) -> Bbox {
    let m = BboxMeasurement {
        t: Timestamp::from_nanos((i as u64) * DT_NS),
        bbox,
    };
    tracker.step(m).unwrap().unwrap().value
}

#[test]
fn converges_to_stationary_truth() {
    let truth = Bbox::new(100.0, 80.0, 50.0, 40.0);
    let mut tracker = make_tracker(Bbox::new(0.0, 0.0, 1.0, 1.0));

    let mut last = Bbox::new(0.0, 0.0, 0.0, 0.0);
    for i in 0..200 {
        let n = det_noise(i);
        last = step_at(
            &mut tracker,
            i,
            Bbox::new(
                truth.cx + n[0],
                truth.cy + n[1],
                truth.w + n[2],
                truth.h + n[3],
            ),
        );
    }

    let cx_err = (last.cx - truth.cx).abs();
    let cy_err = (last.cy - truth.cy).abs();
    let w_err = (last.w - truth.w).abs();
    let h_err = (last.h - truth.h).abs();
    assert!(cx_err < 0.1, "cx error: {cx_err}");
    assert!(cy_err < 0.1, "cy error: {cy_err}");
    assert!(w_err < 0.1, "w error: {w_err}");
    assert!(h_err < 0.1, "h error: {h_err}");
    assert!(
        tracker.velocity().norm() < 0.5,
        "stationary velocity should be small, got {:?}",
        tracker.velocity()
    );
}

#[test]
fn tracks_constant_velocity_position() {
    let p0 = Vector2::new(10.0, 20.0);
    let true_v = Vector2::new(5.0, -2.0);
    let size = (50.0, 30.0);
    let mut tracker = make_tracker(Bbox::new(p0.x, p0.y, size.0, size.1));

    let mut last = Bbox::new(0.0, 0.0, 0.0, 0.0);
    for i in 0..200 {
        let p = p0 + true_v * (i as f64 * DT);
        let n = det_noise(i);
        last = step_at(
            &mut tracker,
            i,
            Bbox::new(p.x + n[0], p.y + n[1], size.0 + n[2], size.1 + n[3]),
        );
    }

    let truth_pos_final = p0 + true_v * (199.0 * DT);
    let pos_err =
        ((last.cx - truth_pos_final.x).powi(2) + (last.cy - truth_pos_final.y).powi(2)).sqrt();
    let vel_err = (tracker.velocity() - true_v).norm();
    assert!(pos_err < 0.5, "position error: {pos_err}");
    assert!(vel_err < 0.5, "velocity error: {vel_err}");
}

#[test]
fn size_random_walk_smooths_noisy_measurements() {
    // Truth size is fixed; noisy measurements should average out.
    let truth = Bbox::new(0.0, 0.0, 100.0, 50.0);
    let mut tracker = make_tracker(truth);

    let mut last = truth;
    for i in 0..500 {
        let n = det_noise(i);
        last = step_at(
            &mut tracker,
            i,
            Bbox::new(
                truth.cx,
                truth.cy,
                truth.w + n[2] * 4.0,
                truth.h + n[3] * 4.0,
            ),
        );
    }

    assert!(
        (last.w - truth.w).abs() < 0.5,
        "w residual {}",
        last.w - truth.w
    );
    assert!(
        (last.h - truth.h).abs() < 0.5,
        "h residual {}",
        last.h - truth.h
    );
}

#[test]
fn reset_restores_initial_state() {
    let initial = Bbox::new(0.0, 0.0, 10.0, 10.0);
    let mut tracker = make_tracker(initial);

    for i in 0..50 {
        step_at(&mut tracker, i, Bbox::new(500.0, 500.0, 200.0, 200.0));
    }

    tracker.reset();

    // Post-reset, an update at the initial bbox should leave us close to it.
    let after = step_at(&mut tracker, 0, initial);
    assert!(
        (after.cx - initial.cx).abs() < 0.1,
        "cx after reset: {}",
        after.cx
    );
    assert!(
        (after.cy - initial.cy).abs() < 0.1,
        "cy after reset: {}",
        after.cy
    );
    assert!(tracker.velocity().norm() < 0.1);
}
