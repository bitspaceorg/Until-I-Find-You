//! Integration tests for the camera and inference traits + their reference
//! implementations.

use uify_runtime::camera::{Camera, FrameFormat, SyntheticCamera};
use uify_runtime::inference::{ConstantInference, Inference};

#[test]
fn synthetic_camera_emits_frames_with_monotonic_timestamps() {
    let period_ns = 33_333_333u64;
    let mut cam = SyntheticCamera::new(64, 48, FrameFormat::Rgba8, period_ns);

    let mut prev = None;
    for i in 0..10u64 {
        let frame = cam.next_frame().unwrap();
        let t = frame.t.as_nanos();
        assert_eq!(t, i * period_ns, "frame {i} at unexpected timestamp");
        if let Some(p) = prev {
            assert!(t > p);
        }
        prev = Some(t);
    }
    assert_eq!(cam.frames_emitted(), 10);
}

#[test]
fn synthetic_camera_buffer_size_matches_format() {
    let cases = [
        (FrameFormat::Rgba8, 4usize),
        (FrameFormat::Bgra8, 4),
        (FrameFormat::Gray8, 1),
    ];
    for (fmt, expected_bpp) in cases {
        assert_eq!(fmt.bytes_per_pixel(), expected_bpp);
        let mut cam = SyntheticCamera::new(32, 16, fmt, 1_000_000);
        let frame = cam.next_frame().unwrap();
        assert_eq!(frame.format, fmt);
        assert_eq!(frame.width, 32);
        assert_eq!(frame.height, 16);
        assert_eq!(frame.data.len(), 32 * 16 * expected_bpp);
    }
}

#[test]
fn synthetic_camera_frames_have_distinguishable_data() {
    let mut cam = SyntheticCamera::new(8, 8, FrameFormat::Gray8, 1_000_000);
    let mut indices = Vec::with_capacity(5);
    for _ in 0..5 {
        let f = cam.next_frame().unwrap();
        // Buffer is filled with the low byte of the frame index.
        indices.push(f.data[0]);
    }
    assert_eq!(indices, vec![0, 1, 2, 3, 4]);
}

#[test]
fn synthetic_camera_buffer_is_uniform_per_frame() {
    let mut cam = SyntheticCamera::new(16, 16, FrameFormat::Rgba8, 1_000_000);
    let f = cam.next_frame().unwrap();
    let first = f.data[0];
    assert!(
        f.data.iter().all(|&b| b == first),
        "expected uniform fill within a frame"
    );
}

#[test]
fn constant_inference_returns_fixed_value() {
    let mut cam = SyntheticCamera::new(8, 8, FrameFormat::Gray8, 1_000_000);
    let mut inf: ConstantInference<i32> = ConstantInference::new(42);
    let f = cam.next_frame().unwrap();
    assert_eq!(inf.infer(&f).unwrap(), 42);
}

#[test]
fn constant_inference_returns_same_value_across_frames() {
    let mut cam = SyntheticCamera::new(8, 8, FrameFormat::Gray8, 1_000_000);
    let mut inf: ConstantInference<f64> = ConstantInference::new(2.5);
    for _ in 0..10 {
        let f = cam.next_frame().unwrap();
        assert_eq!(inf.infer(&f).unwrap(), 2.5);
    }
}
