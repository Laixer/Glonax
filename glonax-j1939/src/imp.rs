use std::{io, mem, os::unix::prelude::*};

use libc::{
    bind, c_int, c_void, close, fcntl, getsockopt, recv, recvfrom, sendto, setsockopt, socket,
    socklen_t, AF_CAN, CAN_J1939, F_GETFL, F_SETFL, O_NONBLOCK, PF_CAN, SOCK_DGRAM, SOL_SOCKET,
    SO_BROADCAST, SO_ERROR,
};

const _J1939_MAX_UNICAST_ADDR: u8 = 0xfd;
const _J1939_IDLE_ADDR: u8 = 0xfe;
const J1939_NO_ADDR: u8 = 0xff;
const J1939_NO_NAME: u64 = 0;
const _J1939_PGN_REQUEST: u32 = 0x0ea00;
const _J1939_PGN_ADDRESS_CLAIMED: u32 = 0x0ee00;
const _J1939_PGN_ADDRESS_COMMANDED: u32 = 0x0fed8;
const _J1939_PGN_PDU1_MAX: u32 = 0x3ff00;
const _J1939_PGN_MAX: u32 = 0x3ffff;
const J1939_NO_PGN: u32 = 0x40000;

pub struct J1939Socket {
    fd: i32,
}

impl J1939Socket {
    fn iface_index(iface_name: &str) -> i32 {
        let iface_name_raw = std::ffi::CString::new(iface_name).unwrap();

        unsafe { libc::if_nametoindex(iface_name_raw.as_ptr()) as i32 }
    }

    #[inline]
    pub fn open(iface_name: &str, addr: u8) -> Result<Self, io::Error> {
        Self::open_fd(Self::iface_index(iface_name), addr)
    }

    pub fn bind(ifname: &str, addr: u8) -> Result<J1939Socket, io::Error> {
        Self::open(ifname, addr)
    }

    fn socket_address(
        ifindex: Option<i32>,
        addr: u8,
        pgn: u32,
    ) -> (libc::sockaddr_can, *mut libc::sockaddr, socklen_t) {
        let mut sockaddr_can = unsafe { std::mem::zeroed::<libc::sockaddr_can>() };
        sockaddr_can.can_family = AF_CAN as u16;
        sockaddr_can.can_addr.j1939.addr = addr;
        sockaddr_can.can_addr.j1939.name = J1939_NO_NAME;
        sockaddr_can.can_addr.j1939.pgn = pgn;

        if let Some(ifindex) = ifindex {
            sockaddr_can.can_ifindex = ifindex;
        }

        let sock_addr_ptr = &mut sockaddr_can as *mut libc::sockaddr_can as *mut libc::sockaddr;
        let sock_addr_len = std::mem::size_of_val(&sockaddr_can);
        (sockaddr_can, sock_addr_ptr, sock_addr_len as socklen_t)
    }

    fn socket_address2(ifindex: Option<i32>, addr: u8, pgn: u32) -> libc::sockaddr_can {
        let mut sockaddr_can = unsafe { std::mem::zeroed::<libc::sockaddr_can>() };
        sockaddr_can.can_family = AF_CAN as u16;
        sockaddr_can.can_addr.j1939.addr = addr;
        sockaddr_can.can_addr.j1939.name = J1939_NO_NAME;
        sockaddr_can.can_addr.j1939.pgn = pgn;

        if let Some(ifindex) = ifindex {
            sockaddr_can.can_ifindex = ifindex;
        }

        sockaddr_can
    }

    /// Open J1939 datagram by interface index.
    pub fn open_fd(iface: i32, addr: u8) -> Result<Self, io::Error> {
        let fd = unsafe {
            let fd = socket(PF_CAN, SOCK_DGRAM, CAN_J1939);

            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            fd
        };

        let (_, local_addr_ptr, local_addr_len) =
            Self::socket_address(Some(iface), addr, J1939_NO_PGN);

        unsafe {
            if bind(fd, local_addr_ptr, local_addr_len) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(Self { fd })
    }

    pub fn close(&self) -> Result<(), io::Error> {
        unsafe {
            if close(self.fd) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Change socket to non-blocking mode
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let sock_flags = unsafe { fcntl(self.fd, F_GETFL) };

        if sock_flags == -1 {
            return Err(io::Error::last_os_error());
        }

        let new_sock_flags = if nonblocking {
            sock_flags | O_NONBLOCK
        } else {
            sock_flags & !O_NONBLOCK
        };

        unsafe {
            if fcntl(self.fd, F_SETFL, new_sock_flags) < 0 {
                return Err(io::Error::last_os_error());
            }
        };

        Ok(())
    }

    pub fn sendto(&self, frame: &j1939::Frame) -> Result<(), io::Error> {
        let pdu_ptr = frame.pdu().as_ptr();

        let (_, peer_addr_ptr, peer_addr_len) = Self::socket_address(
            None,
            frame.id().destination_address().unwrap_or(J1939_NO_ADDR),
            frame.id().pgn() as u32,
        );

        unsafe {
            if sendto(
                self.fd,
                pdu_ptr as *const c_void,
                8,
                0,
                peer_addr_ptr,
                peer_addr_len,
            ) < 0
            {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    pub fn recv(&self) -> Result<j1939::Frame, io::Error> {
        let mut frame = j1939::FrameBuilder::default();
        let pdu_ptr = frame.pdu_mut_ref().as_mut_ptr();

        unsafe {
            if recv(self.fd, pdu_ptr as *mut c_void, 8, 0) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(frame.build())
        }
    }

    pub fn recvfrom(&self) -> Result<j1939::Frame, io::Error> {
        let mut frame = j1939::FrameBuilder::default();
        let pdu_ptr = frame.pdu_mut_ref().as_mut_ptr();

        let mut peer_addr = Self::socket_address2(None, J1939_NO_ADDR, J1939_NO_PGN);

        let sock_addr_ptr = &mut peer_addr as *mut libc::sockaddr_can as *mut libc::sockaddr;
        let mut sock_addr_len = std::mem::size_of_val(&peer_addr) as socklen_t;

        unsafe {
            if recvfrom(
                self.fd,
                pdu_ptr as *mut c_void,
                8,
                0,
                sock_addr_ptr,
                &mut sock_addr_len,
            ) < 0
            {
                return Err(io::Error::last_os_error());
            }

            let id = j1939::IdBuilder::from_pgn(peer_addr.can_addr.j1939.pgn as u16)
                .sa(peer_addr.can_addr.j1939.addr)
                .build();

            Ok(frame.id(id).build())
        }
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`set_broadcast`].
    ///
    /// [`set_broadcast`]: method@Self::set_broadcast
    pub fn broadcast(&self) -> io::Result<bool> {
        let ret: c_int = self.getsockopt(SOL_SOCKET, SO_BROADCAST)?;
        Ok(if ret == 0 { false } else { true })
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.setsockopt(SOL_SOCKET, SO_BROADCAST, on as c_int)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let ret: c_int = self.getsockopt(SOL_SOCKET, SO_ERROR)?;
        if ret == 0 {
            Ok(None)
        } else {
            Ok(Some(io::Error::from_raw_os_error(ret as i32)))
        }
    }

    fn getsockopt<T: Copy>(&self, level: c_int, option_name: c_int) -> io::Result<T> {
        unsafe {
            let mut option_value: T = mem::zeroed();
            let mut option_len = mem::size_of::<T>() as socklen_t;

            if getsockopt(
                self.fd,
                level,
                option_name,
                &mut option_value as *mut T as *mut _,
                &mut option_len,
            )
            .is_negative()
            {
                Err(crate::io::Error::last_os_error())
            } else {
                Ok(option_value)
            }
        }
    }

    pub fn setsockopt<T>(
        &self,
        level: c_int,
        option_name: c_int,
        option_value: T,
    ) -> io::Result<()> {
        unsafe {
            if setsockopt(
                self.fd,
                level,
                option_name,
                &option_value as *const T as *const _,
                mem::size_of::<T>() as socklen_t,
            )
            .is_negative()
            {
                Err(crate::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }
}

impl AsRawFd for J1939Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for J1939Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> J1939Socket {
        J1939Socket { fd }
    }
}

impl IntoRawFd for J1939Socket {
    fn into_raw_fd(self) -> RawFd {
        self.fd
    }
}

impl Drop for J1939Socket {
    fn drop(&mut self) {
        self.close().ok();
    }
}
