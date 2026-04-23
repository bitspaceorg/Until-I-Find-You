//! CLAP plugin crate. Hosts the vision thread, reads from the ring buffer
//! on the audio thread, and writes parameter updates that the DAW records
//! as automation.
//!
//! The audio thread path MUST be allocation-free. See `docs/architecture/threading.mdx`.
//!
//! TODO: wire up nih-plug; expose N parameters initially bound to the point
//! tracker's x/y/z.

#![forbid(unsafe_op_in_unsafe_fn)]

/// Placeholder.
pub struct TrackerPlugin;
