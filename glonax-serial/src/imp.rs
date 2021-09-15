use std::os::unix::prelude::AsRawFd;
use std::{io, os::unix::prelude::RawFd};

pub struct Uart(pub(super) RawFd);

impl Drop for Uart {
    fn drop(&mut self) {
        // TODO: Remove exclusive lock
        // #![allow(unused_must_use)]
        // ioctl::tiocnxcl(self.fd);

        unsafe {
            libc::close(self.0.as_raw_fd());
        }
    }
}

impl AsRawFd for Uart {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl io::Read for Uart {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = unsafe {
            libc::read(
                self.0.as_raw_fd(),
                buf.as_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
            )
        };

        if len >= 0 {
            Ok(len as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl io::Write for Uart {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = unsafe {
            libc::write(
                self.0.as_raw_fd(),
                buf.as_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
            )
        };

        if len >= 0 {
            Ok(len as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        termios::tcdrain(self.0.as_raw_fd())
    }
}
