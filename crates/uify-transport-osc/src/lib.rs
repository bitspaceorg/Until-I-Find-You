//! OSC transport. `rosc` under the hood. Address schema is documented at
//! `docs/reference/transports/osc.mdx`.
//!
//! Each [`Sink::write`] sends one OSC message over UDP:
//!
//! ```text
//! /<path>  ,hfff  <t_ns> <x> <y> <confidence>
//! ```
//!
//! The `t_ns` payload is the host-monotonic timestamp in nanoseconds — it is
//! **not** an OSC time-tag. Listeners must not interpret it as wall-clock
//! time; the same monotonic clock must be used at all ends of the pipeline.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use nalgebra::{Matrix2, Vector2};
use rosc::{OscMessage, OscPacket, OscType, encoder};
use uify_core::{Sample, Sink};

/// Errors from [`OscPoint2DSink::write`].
#[derive(Debug, thiserror::Error)]
pub enum OscError {
    /// `rosc` failed to encode the packet.
    #[error("OSC encode: {0}")]
    Encode(#[from] rosc::OscError),
    /// UDP send failed.
    #[error("UDP send: {0}")]
    Io(#[from] std::io::Error),
}

/// OSC sink for `Sample<Vector2<f64>, Matrix2<f64>>`.
pub struct OscPoint2DSink {
    socket: UdpSocket,
    remote: SocketAddr,
    path: String,
}

impl OscPoint2DSink {
    /// Bind a UDP socket at `local`, target `remote`, and use `path` as the
    /// OSC address for every emitted message.
    pub fn new(
        local: impl ToSocketAddrs,
        remote: impl ToSocketAddrs,
        path: impl Into<String>,
    ) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(local)?;
        let remote = remote.to_socket_addrs()?.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "no remote address")
        })?;
        Ok(Self {
            socket,
            remote,
            path: path.into(),
        })
    }

    /// Address actually bound (useful when the OS picked the port).
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}

impl Sink<Vector2<f64>, Matrix2<f64>> for OscPoint2DSink {
    type Error = OscError;

    fn write(&mut self, sample: &Sample<Vector2<f64>, Matrix2<f64>>) -> Result<(), Self::Error> {
        let msg = OscMessage {
            addr: self.path.clone(),
            args: vec![
                OscType::Long(sample.t.as_nanos() as i64),
                OscType::Float(sample.value.x as f32),
                OscType::Float(sample.value.y as f32),
                OscType::Float(sample.confidence.get()),
            ],
        };
        let bytes = encoder::encode(&OscPacket::Message(msg))?;
        self.socket.send_to(&bytes, self.remote)?;
        Ok(())
    }
}
