//! End-to-end pipeline assembled from existing workspace crates.
//!
//! Data flow:
//!
//! ```text
//!   SyntheticCamera
//!     → ConstantInference  (Vector2<f64> detection)
//!     → PointTracker2D     (smooths into Sample<Vector2, Matrix2>)
//!     → RingbufSink        (vision-thread producer half)
//!     → RingbufSource      (audio-thread-equivalent consumer half)
//!     → OscPoint2DSink     (UDP)
//! ```
//!
//! Single-threaded for determinism. The producer half writes the whole run
//! into the ringbuf, then the consumer half drains it to OSC. In a real
//! deployment the two halves live on different threads; the data flow and
//! the trait surface they bind to are identical.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use std::net::SocketAddr;

use nalgebra::{Matrix2, Matrix4, Vector2, Vector4};
use uify_core::{Sink, Tracker};
use uify_point::{PointMeasurement, PointTracker2D};
use uify_runtime::camera::{Camera, FrameFormat, SyntheticCamera};
use uify_runtime::inference::{ConstantInference, Inference};
use uify_runtime::ringbuf::channel as ringbuf_channel;
use uify_transport_osc::OscPoint2DSink;

/// Pipeline configuration.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Number of frames to pull through the pipeline.
    pub frame_count: u64,
    /// Per-frame timestamp increment (e.g. `33_333_333` for 30 Hz).
    pub frame_period_ns: u64,
    /// Synthetic frame dimensions in pixels.
    pub frame_size: (u32, u32),
    /// Local UDP address for the OSC sink to bind. Use `127.0.0.1:0` to let
    /// the OS pick a port.
    pub osc_local: SocketAddr,
    /// Remote UDP address the OSC sink targets.
    pub osc_remote: SocketAddr,
    /// OSC address pattern emitted with each sample.
    pub osc_path: String,
    /// Ringbuf capacity. Must be at least `frame_count` for this single-
    /// threaded assembly so no samples are dropped.
    pub ring_capacity: usize,
    /// Detection returned by the mock inference for every frame.
    pub fixed_detection: Vector2<f64>,
}

/// Outcome of one pipeline run.
#[derive(Debug, Clone, Copy)]
pub struct PipelineRun {
    /// Samples successfully pushed into the ringbuf by the producer half.
    pub samples_buffered: u64,
    /// Samples successfully shipped over OSC by the consumer half.
    pub samples_emitted: u64,
}

/// Run the pipeline for `cfg.frame_count` frames. Returns counts at the
/// two coarsest checkpoints (ringbuf and OSC).
pub fn run_pipeline(cfg: &PipelineConfig) -> std::io::Result<PipelineRun> {
    let mut cam = SyntheticCamera::new(
        cfg.frame_size.0,
        cfg.frame_size.1,
        FrameFormat::Rgba8,
        cfg.frame_period_ns,
    );
    let mut inf: ConstantInference<Vector2<f64>> = ConstantInference::new(cfg.fixed_detection);
    let mut tracker = PointTracker2D::new(
        Vector4::zeros(),
        Matrix4::identity(),
        Matrix4::identity() * 1e-3,
        Matrix2::identity() * 1e-3,
    );

    let (mut sink, mut source) = ringbuf_channel::<Vector2<f64>, Matrix2<f64>>(cfg.ring_capacity);

    let mut osc = OscPoint2DSink::new(cfg.osc_local, cfg.osc_remote, cfg.osc_path.clone())?;

    let mut samples_buffered = 0u64;
    for _ in 0..cfg.frame_count {
        let frame = cam.next_frame().expect("synthetic camera is infallible");
        let detection = inf.infer(&frame).expect("constant inference is infallible");
        let measurement = PointMeasurement {
            t: frame.t,
            position: detection,
        };
        let sample = tracker
            .step(measurement)
            .expect("point tracker is infallible")
            .expect("tracker emits one sample per measurement");
        if sink.write(&sample).is_ok() {
            samples_buffered += 1;
        }
    }

    let mut samples_emitted = 0u64;
    while let Some(sample) = source.pop() {
        // OSC failures (encode error, UDP send error) just drop the sample;
        // the example surfaces the count rather than the cause.
        if osc.write(&sample).is_ok() {
            samples_emitted += 1;
        }
    }

    Ok(PipelineRun {
        samples_buffered,
        samples_emitted,
    })
}
