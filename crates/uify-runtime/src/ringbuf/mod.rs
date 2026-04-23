//! Lock-free SPSC ring buffer bridging the vision thread and the audio thread.
//!
//! The audio thread MUST NOT allocate, lock, or block. `rtrb` gives us a
//! wait-free reader; the writer is the single vision-thread producer.
//!
//! The ring carries `Sample<G, C>` values. For variable-size geometries
//! (contours, landmark sets) we serialize into a fixed-capacity arena and
//! the ring carries offsets — allocation happens off the audio thread.
//!
//! TODO: implement.

/// Placeholder.
pub struct Ring;
