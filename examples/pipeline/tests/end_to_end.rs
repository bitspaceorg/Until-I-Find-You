//! End-to-end integration test. Binds a real UDP listener, runs the
//! pipeline at it, decodes every packet, and asserts:
//!
//! - exactly `frame_count` packets arrived,
//! - the OSC address matches `osc_path`,
//! - timestamps are strictly monotonic,
//! - the last-decoded position is close to the fixed detection (the
//!   tracker should have converged through 100 noise-free measurements).

use std::net::UdpSocket;
use std::time::Duration;

use nalgebra::Vector2;
use uify_pipeline_example::{PipelineConfig, run_pipeline};

#[test]
fn pipeline_to_osc_listener_decodes_n_packets() {
    let listener = UdpSocket::bind("127.0.0.1:0").expect("bind listener");
    listener
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let frame_count = 100u64;
    let frame_period_ns = 10_000_000u64;
    let truth = Vector2::new(123.5, -42.25);

    let cfg = PipelineConfig {
        frame_count,
        frame_period_ns,
        frame_size: (32, 24),
        osc_local: "127.0.0.1:0".parse().unwrap(),
        osc_remote: listener.local_addr().unwrap(),
        osc_path: "/uify/pipeline/test".into(),
        ring_capacity: frame_count as usize + 1,
        fixed_detection: truth,
    };

    let run = run_pipeline(&cfg).expect("pipeline run");
    assert_eq!(
        run.samples_buffered, frame_count,
        "every frame should land in the ringbuf"
    );
    assert_eq!(
        run.samples_emitted, frame_count,
        "every buffered sample should ship to OSC"
    );

    let mut buf = [0u8; 1024];
    let mut decoded: Vec<rosc::OscMessage> = Vec::with_capacity(frame_count as usize);
    for _ in 0..frame_count {
        let (n, _) = listener.recv_from(&mut buf).expect("recv");
        let (_, packet) = rosc::decoder::decode_udp(&buf[..n]).expect("decode");
        match packet {
            rosc::OscPacket::Message(m) => decoded.push(m),
            rosc::OscPacket::Bundle(_) => panic!("unexpected bundle"),
        }
    }
    assert_eq!(decoded.len(), frame_count as usize);

    // OSC address survives the pipeline.
    for m in &decoded {
        assert_eq!(m.addr, "/uify/pipeline/test");
        assert_eq!(m.args.len(), 4);
    }

    // Timestamps are strictly monotonic and match the camera's cadence.
    let timestamps: Vec<i64> = decoded
        .iter()
        .map(|m| match m.args[0] {
            rosc::OscType::Long(v) => v,
            _ => panic!("arg 0 must be Long"),
        })
        .collect();
    for (i, &t) in timestamps.iter().enumerate() {
        assert_eq!(t, (i as i64) * (frame_period_ns as i64));
    }

    // Last decoded position should be very close to truth — 100 noise-free
    // measurements at fixed coordinates push the tracker to convergence.
    let last = decoded.last().unwrap();
    let x = match last.args[1] {
        rosc::OscType::Float(v) => v as f64,
        _ => panic!("arg 1 must be Float"),
    };
    let y = match last.args[2] {
        rosc::OscType::Float(v) => v as f64,
        _ => panic!("arg 2 must be Float"),
    };
    let err = ((x - truth.x).powi(2) + (y - truth.y).powi(2)).sqrt();
    assert!(err < 0.5, "‖estimate − truth‖ = {err}");
}

#[test]
fn pipeline_drops_excess_samples_when_ringbuf_undersized() {
    let listener = UdpSocket::bind("127.0.0.1:0").expect("bind listener");
    listener
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();

    // Ring smaller than the run: producer fills it up and then `write`
    // returns `Full`. The consumer drains exactly capacity samples.
    let frame_count = 20u64;
    let capacity = 5usize;

    let cfg = PipelineConfig {
        frame_count,
        frame_period_ns: 1_000_000,
        frame_size: (8, 8),
        osc_local: "127.0.0.1:0".parse().unwrap(),
        osc_remote: listener.local_addr().unwrap(),
        osc_path: "/uify/pipeline/test".into(),
        ring_capacity: capacity,
        fixed_detection: Vector2::new(0.0, 0.0),
    };

    let run = run_pipeline(&cfg).expect("pipeline run");
    assert_eq!(run.samples_buffered, capacity as u64);
    assert_eq!(run.samples_emitted, capacity as u64);
}
