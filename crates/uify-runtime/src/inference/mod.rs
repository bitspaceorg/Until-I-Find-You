//! Inference backends. ONNX Runtime is the baseline; platform EPs
//! (CoreML / DirectML / NNAPI / XNNPACK / TensorRT / WebGPU) are feature-gated.
//! A tracker loads a model by path + input/output spec; the runtime picks
//! the fastest available EP at init time.
//!
//! TODO: implement.

/// Placeholder.
pub struct Inference;
