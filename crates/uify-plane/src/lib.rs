//! Planar pose tracker.
//!
//! The full vision (camera → SuperPoint → LightGlue → homography → PnP)
//! depends on `uify-runtime::inference`, which is still stub. What's here
//! today is the **smoother** side: it consumes already-extracted SE(3)
//! pose measurements and produces smoothed `Sample<Se3, Cov6>` via the
//! manifold EKF in `uify-core::filters::ekf_manifold`. A future turn wires
//! the detector pipeline in front of this.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use uify_core::TrackerCommon;
use uify_core::filters::ekf_manifold::{Cov6, Ekf3D};
use uify_core::manifolds::se3::Se3;
use uify_core::sample::{Confidence, Sample, Timestamp};
use uify_core::tracker::{Tracker, TrackerOutput};

/// One SE(3) pose observation.
#[derive(Copy, Clone, Debug)]
pub struct PlaneMeasurement {
    /// Host-monotonic time of capture.
    pub t: Timestamp,
    /// Observed pose.
    pub pose: Se3,
}

/// SE(3) plane-pose tracker. Random-walk predict, direct-pose update.
#[derive(Clone, Debug)]
pub struct PlaneTracker {
    common: TrackerCommon<Ekf3D>,
    process_noise: Cov6,
    measurement_noise: Cov6,
}

impl PlaneTracker {
    /// New tracker at `initial_pose` with the given covariances. `process_noise`
    /// is per-second; it's scaled by `dt` between successive measurements.
    pub fn new(
        initial_pose: Se3,
        initial_cov: Cov6,
        process_noise: Cov6,
        measurement_noise: Cov6,
    ) -> Self {
        Self {
            common: TrackerCommon::new(Ekf3D::new(initial_pose, initial_cov)),
            process_noise,
            measurement_noise,
        }
    }
}

impl Tracker for PlaneTracker {
    type Geometry = Se3;
    type Covariance = Cov6;
    type Measurement<'a> = PlaneMeasurement;

    fn step(&mut self, m: PlaneMeasurement) -> TrackerOutput<Se3, Cov6> {
        let dt = self.common.dt_since(m.t);
        if dt > 0.0 {
            self.common.filter.predict(&self.process_noise, dt);
        }

        self.common.filter.update(&m.pose, &self.measurement_noise);

        Ok(Some(Sample {
            t: m.t,
            value: *self.common.filter.state(),
            covariance: *self.common.filter.covariance(),
            confidence: Confidence::new(1.0),
        }))
    }

    fn reset(&mut self) {
        self.common.reset();
    }
}
