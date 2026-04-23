//! Pipeline stages. A tracker is a composition of stages connected by typed
//! channels. Each stage is pure: it maps an input to zero-or-more outputs.
//!
//! The five canonical stages:
//!
//! ```text
//! Source → Detector → Associator → Filter → Smoother → Sink
//! ```
//!
//! Sources and sinks live outside this crate (runtime / transports). Detectors,
//! associators, filters, and smoothers are all defined here against trait
//! interfaces so they can be swapped without touching the rest of the graph.

/// Generic stage trait: input → output, plus a per-stage reset.
pub trait Stage {
    /// Per-tick input type.
    type In<'a>;
    /// Per-tick output type.
    type Out;

    /// Process one input. Returns `None` if the stage has no output this tick
    /// (e.g. a detector may skip frames, a filter may wait for a seed).
    fn step(&mut self, input: Self::In<'_>) -> Option<Self::Out>;

    /// Discard any internal state.
    fn reset(&mut self);
}
