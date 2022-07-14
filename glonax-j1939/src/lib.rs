use std::io;

use tokio::io::unix::AsyncFd;

pub use j1939;

mod imp;

// TODO: Rename to J1939Socket
pub struct J1939Listener(AsyncFd<imp::J1939Socket>);

impl J1939Listener {
    pub fn bind(ifname: &str, addr: u8) -> Result<J1939Listener, io::Error> {
        let sock = imp::J1939Socket::bind(ifname, addr)?;
        sock.set_nonblocking(true)?;

        Ok(Self(AsyncFd::new(sock)?))
    }

    // TODO:
    // - send()
    // - local_addr()
    // - peer_addr()

    /// Receives a single J1939 frame on the socket from the remote address
    /// to which it is connected. On success, returns the J1939 frame.
    pub async fn recv(&self) -> io::Result<j1939::Frame> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|inner| inner.get_ref().recv()) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Receives a single J1939 frame on the socket from the remote address
    /// to which it is connected. On success, returns the J1939 frame.
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

    /// Returns the value of the `SO_ERROR` option.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.get_ref().take_error()
    }
}
