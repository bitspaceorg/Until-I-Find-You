//! Integration tests for `OscPoint2DSink`.
//!
//! Each test binds a real UDP listener on `127.0.0.1` and an `OscPoint2DSink`
//! that targets it, then exercises the full encode → send → recv → decode
//! path. No mocks; no faked sockets.

use std::net::UdpSocket;
use std::time::Duration;

use nalgebra::{Matrix2, Matrix4, Vector2, Vector4};
use rosc::{OscPacket, OscType};
use uify_core::{Confidence, Sample, Sink, Timestamp, Tracker};
use uify_point::{PointMeasurement, PointTracker2D};
use uify_transport_osc::OscPoint2DSink;

fn alloc_listener() -> UdpSocket {
    let s = UdpSocket::bind("127.0.0.1:0").expect("bind listener");
    s.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set timeout");
    s
}

fn recv_one_message(listener: &UdpSocket) -> rosc::OscMessage {
    let mut buf = [0u8; 1024];
    let (n, _) = listener.recv_from(&mut buf).expect("recv");
    let (_, packet) = rosc::decoder::decode_udp(&buf[..n]).expect("decode");
    match packet {
        OscPacket::Message(m) => m,
        OscPacket::Bundle(_) => panic!("unexpected bundle"),
    }
}

#[test]
fn single_message_roundtrip_preserves_all_fields() {
    let listener = alloc_listener();
    let mut sink = OscPoint2DSink::new(
        "127.0.0.1:0",
        listener.local_addr().unwrap(),
        "/uify/point/2d",
    )
    .unwrap();

    let sample = Sample {
        t: Timestamp::from_nanos(123_456_789),
        value: Vector2::new(1.5, -2.5),
        covariance: Matrix2::identity(),
        confidence: Confidence::new(0.875),
    };
    sink.write(&sample).unwrap();

    let msg = recv_one_message(&listener);
    assert_eq!(msg.addr, "/uify/point/2d");
    assert_eq!(msg.args.len(), 4);

    let OscType::Long(t_ns) = msg.args[0] else {
        panic!("arg 0 must be Long; got {:?}", msg.args[0]);
    };
    let OscType::Float(x) = msg.args[1] else {
        panic!("arg 1 must be Float");
    };
    let OscType::Float(y) = msg.args[2] else {
        panic!("arg 2 must be Float");
    };
    let OscType::Float(conf) = msg.args[3] else {
        panic!("arg 3 must be Float");
    };

    assert_eq!(t_ns, 123_456_789);
    assert!((x - 1.5).abs() < 1e-6, "x = {x}");
    assert!((y - (-2.5)).abs() < 1e-6, "y = {y}");
    assert!((conf - 0.875).abs() < 1e-3, "conf = {conf}");
}

#[test]
fn custom_path_is_preserved() {
    let listener = alloc_listener();
    let mut sink = OscPoint2DSink::new(
        "127.0.0.1:0",
        listener.local_addr().unwrap(),
        "/foo/bar/baz",
    )
    .unwrap();

    sink.write(&Sample {
        t: Timestamp::from_nanos(1),
        value: Vector2::zeros(),
        covariance: Matrix2::identity(),
        confidence: Confidence::new(1.0),
    })
    .unwrap();

    assert_eq!(recv_one_message(&listener).addr, "/foo/bar/baz");
}

#[test]
fn local_addr_reports_a_bound_port() {
    let listener = alloc_listener();
    let sink = OscPoint2DSink::new("127.0.0.1:0", listener.local_addr().unwrap(), "/x").unwrap();

    let local = sink.local_addr().expect("local_addr");
    assert_ne!(local.port(), 0, "OS should have picked a non-zero port");
    assert!(local.ip().is_loopback(), "expected loopback bind");
}

/// Drives the full pipeline: tracker produces samples; sink ships them over
/// UDP; listener decodes; assertions check count + content.
#[test]
fn tracker_to_listener_e2e() {
    let listener = alloc_listener();
    let mut sink = OscPoint2DSink::new(
        "127.0.0.1:0",
        listener.local_addr().unwrap(),
        "/uify/point/2d",
    )
    .unwrap();

    let mut tracker = PointTracker2D::new(
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Matrix4::identity(),
        Matrix4::identity() * 1e-3,
        Matrix2::identity() * 1e-3,
    );

    let truth = Vector2::new(2.0, 1.0);
    let n_steps = 10u64;
    for i in 0..n_steps {
        let m = PointMeasurement {
            t: Timestamp::from_nanos(i * 10_000_000),
            position: truth,
        };
        let s = tracker.step(m).unwrap().unwrap();
        sink.write(&s).unwrap();
    }

    let mut received = Vec::with_capacity(n_steps as usize);
    for _ in 0..n_steps {
        received.push(recv_one_message(&listener));
    }

    assert_eq!(received.len(), n_steps as usize);
    for (i, msg) in received.iter().enumerate() {
        assert_eq!(msg.addr, "/uify/point/2d");
        assert_eq!(msg.args.len(), 4);

        let OscType::Long(t_ns) = msg.args[0] else {
            panic!("arg 0 must be Long");
        };
        assert_eq!(t_ns, (i as i64) * 10_000_000);
    }

    // The last sample should be close to truth — same convergence guarantee
    // as the tracker's own tests, but checked against decoded wire-format
    // bytes to prove no precision was lost in the OSC round trip.
    let last = received.last().unwrap();
    let OscType::Float(x) = last.args[1] else {
        panic!()
    };
    let OscType::Float(y) = last.args[2] else {
        panic!()
    };
    assert!((x as f64 - truth.x).abs() < 0.5, "x last = {x}");
    assert!((y as f64 - truth.y).abs() < 0.5, "y last = {y}");
}
