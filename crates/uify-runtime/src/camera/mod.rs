//! Camera abstraction. One trait, multiple backends (AVFoundation,
//! Media Foundation, V4L2, Camera2, `getUserMedia`). Backends ship in
//! follow-up turns; what's defined here is the trait + a synthetic
//! camera for tests and offline pipelines.
//!
//! # Ownership
//!
//! Cameras lend out their frame buffer:
//! `next_frame(&mut self) -> Result<Frame<'_>, _>`. The borrow checker
//! enforces that the caller releases the frame before pulling the next
//! one. Backends that need a longer-lived handle (CVPixelBuffer, dma-buf)
//! can introduce a `FrameHandle` type later — this trait covers the simple
//! "host-side bytes" case.

use uify_core::Timestamp;

/// Pixel format of a [`Frame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    /// 8 bits per channel, RGBA.
    Rgba8,
    /// 8 bits per channel, BGRA.
    Bgra8,
    /// Single 8-bit luminance channel.
    Gray8,
}

impl FrameFormat {
    /// Bytes per pixel.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            FrameFormat::Rgba8 | FrameFormat::Bgra8 => 4,
            FrameFormat::Gray8 => 1,
        }
    }
}

/// One captured frame. Borrows its pixel buffer from the producing camera.
#[derive(Debug)]
pub struct Frame<'a> {
    /// Host-monotonic time of capture.
    pub t: Timestamp,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Pixel format.
    pub format: FrameFormat,
    /// Pixel buffer. `data.len() == width * height * format.bytes_per_pixel()`.
    pub data: &'a [u8],
}

/// A frame source.
pub trait Camera {
    /// Error type produced by [`next_frame`](Self::next_frame).
    type Error;

    /// Pull the next frame. Blocks until one is available; backends decide
    /// the blocking semantics (sync vs async).
    fn next_frame(&mut self) -> Result<Frame<'_>, Self::Error>;
}

/// Test camera that emits a sequence of synthetic frames at a fixed cadence.
///
/// Each frame's pixel buffer is filled with the low byte of the frame index,
/// so a consumer can tell frames apart by inspecting any single byte.
pub struct SyntheticCamera {
    width: u32,
    height: u32,
    format: FrameFormat,
    buffer: Vec<u8>,
    frame_period_ns: u64,
    next_t_ns: u64,
    frames_emitted: u64,
}

impl SyntheticCamera {
    /// New synthetic camera. `frame_period_ns` is the per-frame timestamp
    /// increment; pass e.g. `33_333_333` for 30 Hz.
    pub fn new(width: u32, height: u32, format: FrameFormat, frame_period_ns: u64) -> Self {
        let len = (width as usize) * (height as usize) * format.bytes_per_pixel();
        Self {
            width,
            height,
            format,
            buffer: vec![0; len],
            frame_period_ns,
            next_t_ns: 0,
            frames_emitted: 0,
        }
    }

    /// Total frames emitted since construction.
    pub fn frames_emitted(&self) -> u64 {
        self.frames_emitted
    }
}

impl Camera for SyntheticCamera {
    type Error = std::convert::Infallible;

    fn next_frame(&mut self) -> Result<Frame<'_>, Self::Error> {
        let idx_byte = self.frames_emitted as u8;
        self.buffer.fill(idx_byte);

        let t = Timestamp::from_nanos(self.next_t_ns);
        self.next_t_ns = self.next_t_ns.saturating_add(self.frame_period_ns);
        self.frames_emitted += 1;

        Ok(Frame {
            t,
            width: self.width,
            height: self.height,
            format: self.format,
            data: &self.buffer,
        })
    }
}
