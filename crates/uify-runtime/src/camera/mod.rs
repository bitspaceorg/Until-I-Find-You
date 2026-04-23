//! Camera abstraction. One trait, five backends:
//!
//! - macOS / iOS: AVFoundation (AVCaptureSession)
//! - Windows: Media Foundation
//! - Linux: V4L2
//! - Android: Camera2 (JNI)
//! - Web: `getUserMedia` (via `web-sys`)
//!
//! Frames are delivered zero-copy where the platform supports it (CVPixelBuffer
//! on Apple, DXGI-shared textures on Windows, dma-buf on Linux). The runtime
//! pipes the frame handle into inference without a CPU round-trip.
//!
//! TODO: implement.

/// Placeholder.
pub struct Camera;
