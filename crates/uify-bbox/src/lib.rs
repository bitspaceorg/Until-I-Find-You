//! Axis-aligned bounding-box tracker. Constant-velocity on position
//! `(cx, cy)`, random-walk on size `(w, h)`. State is 6D Euclidean, so the
//! linear `KalmanFilter` from `uify_core` does the work — no Lie-group
//! machinery here.
//!
//! Oriented boxes (`SE(2) × ℝ²`) will land separately once SE(2) is in.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use nalgebra::{Matrix4, Matrix4x6, Matrix6, Vector2, Vector4, Vector6};
use uify_core::TrackerCommon;
use uify_core::filters::kalman::KalmanFilter;
use uify_core::sample::{Confidence, Sample, Timestamp};
use uify_core::tracker::{Tracker, TrackerOutput};

/// Axis-aligned bounding box. `cx`, `cy` are the box center; `w`, `h` its
/// full extent (not half-extent).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bbox {
    /// Center x.
    pub cx: f64,
    /// Center y.
    pub cy: f64,
    /// Full width.
    pub w: f64,
    /// Full height.
    pub h: f64,
}

impl Bbox {
    /// Construct from center + size.
    pub fn new(cx: f64, cy: f64, w: f64, h: f64) -> Self {
        Self { cx, cy, w, h }
    }
}

/// Single bbox observation.
#[derive(Copy, Clone, Debug)]
pub struct BboxMeasurement {
    /// Host-monotonic time of capture.
    pub t: Timestamp,
    /// Observed bounding box.
    pub bbox: Bbox,
}

/// AABB tracker: 6D state `(cx, cy, w, h, vcx, vcy)`, 4D measurement
/// `(cx, cy, w, h)`.
#[derive(Clone, Debug)]
pub struct BboxTracker2D {
    common: TrackerCommon<KalmanFilter<6>>,
    process_noise: Matrix6<f64>,
    measurement_noise: Matrix4<f64>,
}

impl BboxTracker2D {
    /// Build a tracker with explicit initial state, initial covariance, and
    /// noise matrices. `process_noise` is per-second and is scaled by `dt`
    /// each step.
    pub fn new(
        initial_state: Vector6<f64>,
        initial_cov: Matrix6<f64>,
        process_noise: Matrix6<f64>,
        measurement_noise: Matrix4<f64>,
    ) -> Self {
        Self {
            common: TrackerCommon::new(KalmanFilter::new(initial_state, initial_cov)),
            process_noise,
            measurement_noise,
        }
    }

    /// Current center-velocity estimate `(vcx, vcy)`.
    pub fn velocity(&self) -> Vector2<f64> {
        let st = self.common.filter.state();
        Vector2::new(st[4], st[5])
    }
}

impl Tracker for BboxTracker2D {
    type Geometry = Bbox;
    type Covariance = Matrix4<f64>;
    type Measurement<'a> = BboxMeasurement;

    fn step(&mut self, m: BboxMeasurement) -> TrackerOutput<Bbox, Matrix4<f64>> {
        let dt = self.common.dt_since(m.t);
        if dt > 0.0 {
            self.common
                .filter
                .predict(&transition(dt), &(self.process_noise * dt));
        }

        let z = Vector4::new(m.bbox.cx, m.bbox.cy, m.bbox.w, m.bbox.h);
        self.common
            .filter
            .update(&MEASUREMENT_MODEL, &z, &self.measurement_noise);

        let st = self.common.filter.state();
        let bbox = Bbox::new(st[0], st[1], st[2], st[3]);
        let cov = self
            .common
            .filter
            .covariance()
            .fixed_view::<4, 4>(0, 0)
            .into_owned();

        Ok(Some(Sample {
            t: m.t,
            value: bbox,
            covariance: cov,
            confidence: Confidence::new(1.0),
        }))
    }

    fn reset(&mut self) {
        self.common.reset();
    }
}

/// `H` = top-left 4×4 identity, zeros to the right (measure cx, cy, w, h).
const MEASUREMENT_MODEL: Matrix4x6<f64> = Matrix4x6::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0,
);

/// State transition `[I₂ 0₂ dt·I₂; 0₂ I₂ 0₂; 0₂ 0₂ I₂]` arranged for the
/// state ordering `(cx, cy, w, h, vcx, vcy)`. Width and height are
/// random-walk; only the position has velocity dynamics.
fn transition(dt: f64) -> Matrix6<f64> {
    let mut f = Matrix6::identity();
    f[(0, 4)] = dt; // cx += vcx · dt
    f[(1, 5)] = dt; // cy += vcy · dt
    f
}
