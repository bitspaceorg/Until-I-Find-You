//! uify-runtime: the I/O layer. Owns cameras, GPU inference, and the
//! lock-free bridge between the vision thread and real-time audio consumers.
//!
//! The audio thread only ever touches `ringbuf` — everything else is
//! deliberately off the hot path.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod camera;
pub mod inference;
pub mod ringbuf;
