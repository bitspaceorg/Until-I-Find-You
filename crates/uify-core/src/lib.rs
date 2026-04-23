//! uify-core: geometric primitives, manifold math, filters, and the
//! `Tracker<G>` trait that every tracker implements.
//!
//! This crate is I/O-free. It does not touch cameras, GPU, inference runtimes,
//! or plugin hosts. Everything here is pure computation and is safe to run on
//! any thread, including a real-time audio thread.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod filters;
pub mod manifolds;
pub mod pipeline;
pub mod sample;
pub mod tracker;

pub use sample::{Confidence, Sample, Timestamp};
pub use tracker::{Tracker, TrackerError, TrackerOutput};
