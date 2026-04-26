//! Lock-free SPSC ring buffer bridging the vision thread and the audio thread.
//!
//! The audio thread MUST NOT allocate, lock, or block. `rtrb` gives a
//! wait-free reader; the writer is the single vision-thread producer.
//!
//! Carries `Sample<G, C>` values. For fixed-size geometries (`Vector2`,
//! `Vector3`, `Bbox`, etc.) the whole sample is `Copy`-friendly and `push`
//! is allocation-free after `channel` is constructed.
//!
//! Variable-size geometries (contours, landmark sets) will need a separate
//! arena strategy — not yet implemented.

use rtrb::{Consumer, Producer, RingBuffer};
use uify_core::{Sample, Sink};

/// Errors from [`RingbufSink::write`].
#[derive(Debug, thiserror::Error)]
pub enum RingbufError {
    /// The buffer is full and the consumer has not made room. The sample
    /// was dropped.
    #[error("ringbuf full — consumer fell behind")]
    Full,
}

/// Producer half: implements [`Sink`] so a tracker can be wired straight
/// into a real-time-safe queue without knowing the consumer's identity.
pub struct RingbufSink<G, C> {
    producer: Producer<Sample<G, C>>,
}

/// Consumer half: pops samples in FIFO order. Wait-free; safe to call from
/// the audio thread.
pub struct RingbufSource<G, C> {
    consumer: Consumer<Sample<G, C>>,
}

/// Allocate a SPSC ring of fixed `capacity`. The two halves are Send when
/// `Sample<G, C>` is Send, so they can be moved into separate threads.
pub fn channel<G, C>(capacity: usize) -> (RingbufSink<G, C>, RingbufSource<G, C>) {
    let (producer, consumer) = RingBuffer::new(capacity);
    (RingbufSink { producer }, RingbufSource { consumer })
}

impl<G, C> Sink<G, C> for RingbufSink<G, C>
where
    G: Clone,
    C: Clone,
{
    type Error = RingbufError;

    fn write(&mut self, sample: &Sample<G, C>) -> Result<(), Self::Error> {
        self.producer
            .push(sample.clone())
            .map_err(|_| RingbufError::Full)
    }
}

impl<G, C> RingbufSink<G, C> {
    /// Number of slots remaining before [`Sink::write`] returns
    /// [`RingbufError::Full`].
    pub fn slots(&self) -> usize {
        self.producer.slots()
    }
}

impl<G, C> RingbufSource<G, C> {
    /// Pop the oldest sample, or `None` if empty.
    pub fn pop(&mut self) -> Option<Sample<G, C>> {
        self.consumer.pop().ok()
    }

    /// True if there are no samples to read.
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// Number of samples currently buffered.
    pub fn len(&self) -> usize {
        self.consumer.slots()
    }
}
