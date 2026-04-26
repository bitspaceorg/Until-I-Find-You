//! Integration tests for the SPSC ringbuf.
//!
//! Single-process, single-thread tests verify FIFO semantics and capacity
//! limits. The cross-thread test exercises the actual SPSC contract — one
//! thread owns the producer, the main thread owns the consumer.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nalgebra::{Matrix2, Vector2};
use uify_core::{Confidence, Sample, Sink, Timestamp};
use uify_runtime::ringbuf::{RingbufError, channel};

type S = Sample<Vector2<f64>, Matrix2<f64>>;

fn sample(i: u64) -> S {
    Sample {
        t: Timestamp::from_nanos(i),
        value: Vector2::new(i as f64, -(i as f64)),
        covariance: Matrix2::identity() * (i as f64 + 1.0),
        confidence: Confidence::new(1.0),
    }
}

#[test]
fn push_pop_roundtrip() {
    let (mut tx, mut rx) = channel::<Vector2<f64>, Matrix2<f64>>(8);

    let s = sample(42);
    tx.write(&s).unwrap();

    assert_eq!(rx.len(), 1);
    let popped = rx.pop().expect("non-empty");
    assert_eq!(popped.t.as_nanos(), 42);
    assert_eq!(popped.value, Vector2::new(42.0, -42.0));
    assert!(rx.is_empty());
    assert_eq!(rx.pop().map(|_| ()), None);
}

#[test]
fn fifo_order_preserved() {
    let (mut tx, mut rx) = channel::<Vector2<f64>, Matrix2<f64>>(16);

    for i in 0..10 {
        tx.write(&sample(i)).unwrap();
    }
    for i in 0..10 {
        let s = rx.pop().expect("non-empty");
        assert_eq!(s.t.as_nanos(), i);
    }
    assert!(rx.is_empty());
}

#[test]
fn capacity_full_returns_error() {
    let cap = 4;
    let (mut tx, _rx) = channel::<Vector2<f64>, Matrix2<f64>>(cap);

    for i in 0..cap as u64 {
        tx.write(&sample(i)).unwrap();
    }
    let err = tx.write(&sample(99)).unwrap_err();
    assert!(matches!(err, RingbufError::Full));
}

#[test]
fn slots_decreases_then_recovers() {
    let cap = 4;
    let (mut tx, mut rx) = channel::<Vector2<f64>, Matrix2<f64>>(cap);

    assert_eq!(tx.slots(), cap);
    tx.write(&sample(1)).unwrap();
    assert_eq!(tx.slots(), cap - 1);
    rx.pop().unwrap();
    // Queue is logically empty; producer eventually sees free space again.
    // rtrb may reflect this immediately after a pop on the same thread.
    assert!(tx.slots() >= cap - 1);
}

/// Producer in a worker thread; consumer in the main thread. Validates the
/// `Send` story and FIFO semantics across a real thread boundary.
#[test]
fn producer_thread_consumer_main_thread() {
    let n = 1000u64;
    let cap = 64;
    let (mut tx, mut rx) = channel::<Vector2<f64>, Matrix2<f64>>(cap);

    let producer_done = Arc::new(AtomicBool::new(false));
    let producer_done_writer = producer_done.clone();

    let handle = thread::spawn(move || {
        for i in 0..n {
            // Spin until the slot is free; in real-time code the producer
            // would drop instead, but for this test we want every sample
            // to land.
            while tx.write(&sample(i)).is_err() {
                std::hint::spin_loop();
            }
        }
        producer_done_writer.store(true, Ordering::Release);
    });

    let mut received = Vec::with_capacity(n as usize);
    let deadline = Instant::now() + Duration::from_secs(5);
    while received.len() < n as usize {
        if let Some(s) = rx.pop() {
            received.push(s.t.as_nanos());
        } else if producer_done.load(Ordering::Acquire) && rx.is_empty() {
            break;
        } else if Instant::now() > deadline {
            panic!("timed out: received {} of {}", received.len(), n);
        }
    }

    handle.join().expect("producer thread panicked");

    assert_eq!(received.len(), n as usize);
    for (i, &t) in received.iter().enumerate() {
        assert_eq!(t, i as u64, "out-of-order at index {i}");
    }
}
