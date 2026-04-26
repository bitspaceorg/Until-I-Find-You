//! Inference backends. ONNX Runtime is the planned baseline; platform EPs
//! (CoreML / DirectML / NNAPI / XNNPACK / TensorRT / WebGPU) will be
//! feature-gated. A tracker loads a model by path + input/output spec; the
//! runtime picks the fastest available EP at init time.
//!
//! What's here today is the trait + a constant-output mock for tests.

use crate::camera::Frame;

/// Run a model on a frame.
pub trait Inference {
    /// Inference output type. Concrete shapes (point list, bbox list,
    /// landmark grid, …) are model-specific.
    type Output;

    /// Error type produced by [`infer`](Self::infer).
    type Error;

    /// Run the model on `frame`. Implementations are typically the slowest
    /// stage in the pipeline — schedule on the vision thread, never the
    /// audio thread.
    fn infer(&mut self, frame: &Frame<'_>) -> Result<Self::Output, Self::Error>;
}

/// Test [`Inference`] impl that returns a fixed clone of `output` on every
/// call. Useful when wiring downstream components against a deterministic
/// upstream signal.
pub struct ConstantInference<O: Clone> {
    output: O,
}

impl<O: Clone> ConstantInference<O> {
    /// New constant-output mock.
    pub fn new(output: O) -> Self {
        Self { output }
    }
}

impl<O: Clone> Inference for ConstantInference<O> {
    type Output = O;
    type Error = std::convert::Infallible;

    fn infer(&mut self, _frame: &Frame<'_>) -> Result<Self::Output, Self::Error> {
        Ok(self.output.clone())
    }
}
