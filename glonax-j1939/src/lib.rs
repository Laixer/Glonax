use std::io;

mod imp;

pub use j1939;

pub struct J1939Listener(tokio::io::unix::AsyncFd<imp::J1939Socket>);

impl J1939Listener {
    pub fn bind(ifname: &str, addr: u8) -> Result<J1939Listener, io::Error> {
        let sock = imp::J1939Socket::bind(ifname, addr)?;
        sock.set_nonblocking(true)?;

        Ok(Self(tokio::io::unix::AsyncFd::new(sock)?))
    }

    // TODO:
    // - send()
    // - local_addr()
    // - peer_addr()

    pub async fn recv(&self) -> io::Result<j1939::Frame> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|inner| inner.get_ref().recv()) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn recv_from(&self) -> io::Result<j1939::Frame> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|inner| inner.get_ref().recvfrom()) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn send_to(&self, frame: &j1939::Frame) -> io::Result<()> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|inner| inner.get_ref().sendto(frame)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    // pub async fn connect(&self, addr: u8) -> io::Result<()> {
    //     self.0.get_ref().connect();
    // }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`set_broadcast`].
    ///
    /// [`set_broadcast`]: method@Self::set_broadcast
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.get_ref().broadcast()
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.0.get_ref().set_broadcast(on)
    }
}
