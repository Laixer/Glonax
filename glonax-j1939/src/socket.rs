use std::{io, os::unix::prelude::*};

use socket2::SockAddr;
use tokio::io::unix::AsyncFd;

pub struct SockAddrJ1939 {
    pub name: u64,
    pub pgn: u32,
    pub addr: u8,
    pub ifindex: Option<i32>,
}

impl SockAddrJ1939 {
    pub fn bind(addr: u8, ifname: &str) -> Self {
        Self {
            name: libc::J1939_NO_NAME,
            pgn: libc::J1939_NO_PGN,
            addr,
            ifindex: Some(crate::sys::if_nametoindex(ifname)),
        }
    }

    pub fn send(addr: u8, pgn: u32) -> Self {
        Self {
            name: libc::J1939_NO_NAME,
            pgn,
            addr,
            ifindex: None,
        }
    }
}

impl From<&SockAddrJ1939> for SockAddr {
    fn from(value: &SockAddrJ1939) -> SockAddr {
        let mut sockaddr_can =
            unsafe { std::mem::MaybeUninit::<libc::sockaddr_can>::zeroed().assume_init() };
        sockaddr_can.can_family = libc::AF_CAN as u16;
        sockaddr_can.can_addr.j1939.addr = value.addr;
        sockaddr_can.can_addr.j1939.name = value.name;
        sockaddr_can.can_addr.j1939.pgn = value.pgn;

        if let Some(ifindex) = value.ifindex {
            sockaddr_can.can_ifindex = ifindex;
        }

        let mut storage = std::mem::MaybeUninit::<libc::sockaddr_storage>::zeroed();
        unsafe { (storage.as_mut_ptr() as *mut libc::sockaddr_can).write(sockaddr_can) };

        unsafe {
            SockAddr::new(
                storage.assume_init(),
                std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t,
            )
        }
    }
}

impl From<SockAddr> for SockAddrJ1939 {
    fn from(value: socket2::SockAddr) -> Self {
        let sockaddr_can = unsafe { *(value.as_ptr() as *const libc::sockaddr_can) };

        unsafe {
            Self {
                addr: sockaddr_can.can_addr.j1939.addr,
                pgn: sockaddr_can.can_addr.j1939.pgn,
                name: sockaddr_can.can_addr.j1939.name,
                ifindex: None,
            }
        }
    }
}

pub struct J1939Socket(AsyncFd<socket2::Socket>);

impl J1939Socket {
    /// Binds this socket to the specified address and interface.
    pub fn bind(address: &SockAddrJ1939) -> io::Result<Self> {
        let socket = socket2::Socket::new_raw(
            libc::AF_CAN.into(),
            socket2::Type::DGRAM,
            Some(libc::CAN_J1939.into()),
        )?;

        socket.bind(&address.into())?;
        socket.set_nonblocking(true)?;

        Ok(Self(AsyncFd::new(socket)?))
    }

    /// Sends data on the socket to a connected peer.
    ///
    /// On success returns the number of bytes that were sent.
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|inner| inner.get_ref().send(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    ///
    /// This is typically used on UDP or datagram-oriented sockets.
    pub async fn send_to(&self, buf: &[u8], addr: &SockAddrJ1939) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|inner| inner.get_ref().send_to(buf, &addr.into())) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected.
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            let buf_uninit = unsafe {
                std::slice::from_raw_parts_mut(
                    buf.as_mut_ptr() as *mut std::mem::MaybeUninit<u8>,
                    buf.len() * std::mem::size_of::<u8>(),
                )
            };

            match guard.try_io(|inner| inner.get_ref().recv(buf_uninit)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Receives a single J1939 frame on the socket from the remote address
    /// to which it is connected. On success, returns the J1939 frame.
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SockAddrJ1939)> {
        loop {
            let mut guard = self.0.readable().await?;

            let buf_uninit = unsafe {
                std::slice::from_raw_parts_mut(
                    buf.as_mut_ptr() as *mut std::mem::MaybeUninit<u8>,
                    buf.len() * std::mem::size_of::<u8>(),
                )
            };

            match guard.try_io(|inner| inner.get_ref().recv_from(buf_uninit)) {
                Ok(result) => return result.map(|(size, sockaddr)| (size, sockaddr.into())),
                Err(_would_block) => continue,
            }
        }
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value.
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.0.get_ref().shutdown(how)
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

impl AsRawFd for J1939Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}