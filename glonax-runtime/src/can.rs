use std::{io, os::unix::prelude::*};

use socket2::SockAddr;
use tokio::io::unix::AsyncFd;

mod sys {
    pub(super) fn if_nametoindex(iface_name: &str) -> i32 {
        let iface_name_raw = std::ffi::CString::new(iface_name).unwrap();

        unsafe { libc::if_nametoindex(iface_name_raw.as_ptr()) as i32 }
    }
}

pub struct SockAddrJ1939 {
    pub name: u64,
    pub pgn: u32,
    pub addr: u8,
    pub ifindex: Option<i32>,
}

impl SockAddrJ1939 {
    pub fn new(addr: u8, ifname: &str) -> Self {
        Self {
            name: libc::J1939_NO_NAME,
            pgn: libc::J1939_NO_PGN,
            addr,
            ifindex: Some(sys::if_nametoindex(ifname)),
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
                ifindex: Some(sockaddr_can.can_ifindex),
            }
        }
    }
}

pub struct SockAddrCAN {
    pub ifindex: i32,
}

impl SockAddrCAN {
    pub fn new(ifname: &str) -> Self {
        Self {
            ifindex: sys::if_nametoindex(ifname),
        }
    }
}

impl From<&SockAddrCAN> for SockAddr {
    fn from(value: &SockAddrCAN) -> SockAddr {
        let mut sockaddr_can =
            unsafe { std::mem::MaybeUninit::<libc::sockaddr_can>::zeroed().assume_init() };
        sockaddr_can.can_family = libc::AF_CAN as u16;
        sockaddr_can.can_ifindex = value.ifindex;

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

impl From<SockAddr> for SockAddrCAN {
    fn from(value: socket2::SockAddr) -> Self {
        let sockaddr_can = unsafe { *(value.as_ptr() as *const libc::sockaddr_can) };

        Self {
            ifindex: sockaddr_can.can_ifindex,
        }
    }
}

pub struct CANSocket(AsyncFd<socket2::Socket>);

impl CANSocket {
    /// Binds this socket to the specified address and interface.
    pub fn bind(address: impl Into<SockAddr>) -> io::Result<Self> {
        let socket = socket2::Socket::new_raw(
            libc::AF_CAN.into(),
            socket2::Type::RAW,
            Some(libc::CAN_RAW.into()),
        )?;

        socket.bind(&address.into())?;
        socket.set_nonblocking(true)?;
        socket.set_broadcast(true)?;

        Ok(Self(AsyncFd::new(socket)?))
    }

    /// Binds this socket to the specified address and interface.
    pub fn bind_j1939(address: impl Into<SockAddr>) -> io::Result<Self> {
        let socket = socket2::Socket::new_raw(
            libc::AF_CAN.into(),
            socket2::Type::DGRAM,
            Some(libc::CAN_J1939.into()),
        )?;

        socket.bind(&address.into())?;
        socket.set_nonblocking(true)?;
        socket.set_broadcast(true)?;

        Ok(Self(AsyncFd::new(socket)?))
    }

    /// Sends data on the socket to a connected peer.
    ///
    /// On success returns the number of bytes that were sent.
    #[allow(dead_code)]
    pub async fn send_raw(&self, buf: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|inner| inner.get_ref().send(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Sends a single J1939 frame on the socket to the CAN bus. On success,
    /// returns the number of bytes written.
    pub async fn send(&self, frame: &j1939::Frame) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            let mut can_frame =
                unsafe { std::mem::MaybeUninit::<libc::can_frame>::zeroed().assume_init() };

            can_frame.can_id = frame.id().as_raw() | 0x80000000;
            can_frame.can_dlc = frame.len() as u8;
            can_frame.data[..frame.len()].copy_from_slice(frame.pdu());

            let buf2 = unsafe {
                std::slice::from_raw_parts(
                    &can_frame as *const libc::can_frame as *const u8,
                    std::mem::size_of::<libc::can_frame>(),
                )
            };

            match guard.try_io(|inner| inner.get_ref().send(buf2)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    /// Send a vector of frames over the network.
    pub async fn send_vectored(&self, frames: &Vec<j1939::Frame>) -> io::Result<Vec<usize>> {
        let mut v = vec![];
        for frame in frames {
            v.push(self.send(frame).await?);
        }
        Ok(v)
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected.
    #[allow(dead_code)]
    pub async fn recv_raw(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            let buf_uninit = unsafe {
                std::slice::from_raw_parts_mut(
                    buf.as_mut_ptr() as *mut std::mem::MaybeUninit<u8>,
                    std::mem::size_of_val(buf),
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
    pub async fn recv(&self) -> io::Result<j1939::Frame> {
        loop {
            let mut guard = self.0.readable().await?;

            let mut storage = std::mem::MaybeUninit::<libc::can_frame>::zeroed();

            let buf_uninit = unsafe {
                std::slice::from_raw_parts_mut(
                    storage.as_mut_ptr() as *mut std::mem::MaybeUninit<u8>,
                    std::mem::size_of::<libc::can_frame>(),
                )
            };

            match guard.try_io(|inner| inner.get_ref().recv(buf_uninit)) {
                Ok(result) => {
                    let can_frame = unsafe { storage.assume_init() };

                    return result.map(|_size| {
                        j1939::FrameBuilder::new(j1939::Id::new(can_frame.can_id & 0x1fffffff))
                            .copy_from_slice(&can_frame.data[..can_frame.can_dlc as usize])
                            .build()
                    });
                }
                Err(_would_block) => continue,
            }
        }
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value.
    #[inline]
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.0.get_ref().shutdown(how)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`set_broadcast`].
    ///
    /// [`set_broadcast`]: method@Self::set_broadcast
    #[inline]
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.get_ref().broadcast()
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    #[inline]
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.0.get_ref().set_broadcast(on)
    }

    /// Sets the value of the `SO_J1939_PROMISC` option for this socket.
    ///
    /// When enabled, this socket clears all filters set by the bind and connect
    /// methods. In promiscuous mode the socket receives all packets including
    /// the packets sent from this socket.
    pub fn set_promisc_mode(&self, on: bool) -> io::Result<()> {
        unsafe {
            let optval: libc::c_int = on.into();

            if libc::setsockopt(
                self.0.as_raw_fd(),
                libc::SOL_CAN_J1939,
                libc::SO_J1939_PROMISC,
                &optval as *const _ as *const libc::c_void,
                std::mem::size_of_val(&optval) as libc::socklen_t,
            ) < 0
            {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Get the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    #[inline]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.get_ref().take_error()
    }
}

impl AsRawFd for CANSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
