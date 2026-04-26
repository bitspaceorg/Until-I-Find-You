//! Default-config demo: pulls 100 frames through the pipeline, ships
//! samples to `127.0.0.1:9000`. If nothing is listening the UDP packets
//! are silently dropped — that's fine for a demo.

use nalgebra::Vector2;
use uify_pipeline_example::{PipelineConfig, run_pipeline};

fn main() -> std::io::Result<()> {
    let cfg = PipelineConfig {
        frame_count: 100,
        frame_period_ns: 33_333_333, // ~30 Hz
        frame_size: (640, 480),
        osc_local: "127.0.0.1:0".parse().unwrap(),
        osc_remote: "127.0.0.1:9000".parse().unwrap(),
        osc_path: "/uify/point/2d".into(),
        ring_capacity: 256,
        fixed_detection: Vector2::new(320.0, 240.0),
    };

    let run = run_pipeline(&cfg)?;
    println!(
        "uify-pipeline-example: {} buffered, {} emitted to {} ({})",
        run.samples_buffered, run.samples_emitted, cfg.osc_remote, cfg.osc_path
    );
    Ok(())
}
