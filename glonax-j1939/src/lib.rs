use std::io;

pub use j1939::{decode, protocol, Frame, FrameBuilder, IdBuilder, PGN};
pub use socket::J1939Socket;

mod socket;

mod sys {
    pub(super) fn if_nametoindex(iface_name: &str) -> i32 {
        let iface_name_raw = std::ffi::CString::new(iface_name).unwrap();

        unsafe { libc::if_nametoindex(iface_name_raw.as_ptr()) as i32 }
    }
}

impl From<&j1939::Id> for socket::SockAddrJ1939 {
    fn from(value: &j1939::Id) -> Self {
        socket::SockAddrJ1939::send(
            value.destination_address().unwrap_or(libc::J1939_NO_ADDR),
            value.pgn_raw(),
        )
    }
}

impl From<socket::SockAddrJ1939> for j1939::Id {
    fn from(value: socket::SockAddrJ1939) -> Self {
        IdBuilder::from_pgn(value.pgn.into()).sa(value.addr).build()
    }
}

pub struct J1939Stream(J1939Socket);

impl J1939Stream {
    pub fn bind(ifname: &str, addr: u8) -> io::Result<Self> {
        let address = socket::SockAddrJ1939::bind(addr, ifname);
        J1939Socket::bind(&address).map(J1939Stream)
    }

    /// Read frame from network stream.
    pub async fn read(&self) -> io::Result<Frame> {
        let mut frame = FrameBuilder::default();

        let (frame_size, peer_addr) = self.0.recv_from(frame.as_mut()).await?;

        frame = frame.set_len(frame_size);

        Ok(frame.id(peer_addr.into()).build())
    }

    /// Write frame over the network stream.
    #[inline]
    pub async fn write(&self, frame: &Frame) -> io::Result<usize> {
        self.0.send_to(frame.pdu(), &frame.id().into()).await
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value.
    #[inline]
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`set_broadcast`].
    ///
    /// [`set_broadcast`]: method@Self::set_broadcast
    #[inline]
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.broadcast()
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    #[inline]
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.0.set_broadcast(on)
    }

    /// Sets the value of the `SO_J1939_PROMISC` option for this socket.
    ///
    /// When enabled, this socket clears all filters set by the bind and connect
    /// methods. In promiscuous mode the socket receives all packets including
    /// the packets sent from this socket.
    #[inline]
    pub fn set_promisc_mode(&self, on: bool) -> io::Result<()> {
        self.0.set_promisc_mode(on)
    }

    /// Returns the value of the `SO_ERROR` option.
    #[inline]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }
}

impl From<J1939Socket> for J1939Stream {
    fn from(value: J1939Socket) -> Self {
        J1939Stream(value)
    }
}
