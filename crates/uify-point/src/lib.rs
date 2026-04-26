//! 2D point tracker with a constant-velocity model and a linear Kalman
//! filter. State is `(px, py, vx, vy) ∈ ℝ⁴`; measurements are 2D positions.
//! No Lie-group machinery is needed because position+velocity is Euclidean.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use nalgebra::{Matrix2, Matrix2x4, Matrix4, Vector2, Vector4};
use uify_core::TrackerCommon;
use uify_core::filters::kalman::KalmanFilter;
use uify_core::sample::{Confidence, Sample, Timestamp};
use uify_core::tracker::{Tracker, TrackerOutput};

/// One observation fed into [`PointTracker2D`].
#[derive(Copy, Clone, Debug)]
pub struct PointMeasurement {
    /// Host-monotonic time of capture.
    pub t: Timestamp,
    /// Observed 2D position.
    pub position: Vector2<f64>,
}

/// Constant-velocity 2D point tracker.
#[derive(Clone, Debug)]
pub struct PointTracker2D {
    common: TrackerCommon<KalmanFilter<4>>,
    process_noise: Matrix4<f64>,
    measurement_noise: Matrix2<f64>,
}

impl PointTracker2D {
    /// Build a tracker with explicit initial state, initial covariance, and
    /// noise spectral densities. `process_noise` is per-second, scaled by
    /// `dt` inside `step`.
    pub fn new(
        initial_state: Vector4<f64>,
        initial_cov: Matrix4<f64>,
        process_noise: Matrix4<f64>,
        measurement_noise: Matrix2<f64>,
    ) -> Self {
        Self {
            common: TrackerCommon::new(KalmanFilter::new(initial_state, initial_cov)),
            process_noise,
            measurement_noise,
        }
    }

    /// Current velocity estimate `(vx, vy)`.
    pub fn velocity(&self) -> Vector2<f64> {
        let st = self.common.filter.state();
        Vector2::new(st[2], st[3])
    }
}

impl Tracker for PointTracker2D {
    type Geometry = Vector2<f64>;
    type Covariance = Matrix2<f64>;
    type Measurement<'a> = PointMeasurement;

    fn step(&mut self, m: PointMeasurement) -> TrackerOutput<Vector2<f64>, Matrix2<f64>> {
        let dt = self.common.dt_since(m.t);
        if dt > 0.0 {
            self.common
                .filter
                .predict(&transition(dt), &(self.process_noise * dt));
        }

        self.common
            .filter
            .update(&MEASUREMENT_MODEL, &m.position, &self.measurement_noise);

        let st = self.common.filter.state();
        let position = Vector2::new(st[0], st[1]);
        let pos_cov = self
            .common
            .filter
            .covariance()
            .fixed_view::<2, 2>(0, 0)
            .into_owned();

        Ok(Some(Sample {
            t: m.t,
            value: position,
            covariance: pos_cov,
            confidence: Confidence::new(1.0),
        }))
    }

    fn reset(&mut self) {
        self.common.reset();
    }
}

/// `H` selects position from the 4D state.
const MEASUREMENT_MODEL: Matrix2x4<f64> = Matrix2x4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

/// Constant-velocity transition matrix `[I, dt·I; 0, I]`.
fn transition(dt: f64) -> Matrix4<f64> {
    Matrix4::new(
        1.0, 0.0, dt, 0.0, 0.0, 1.0, 0.0, dt, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    )
}
