use std::os::unix::prelude::AsRawFd;
use std::path::Path;
use std::{io, os::unix::prelude::RawFd};

use crate::{BaudRate, Builder, FlowControl, Parity, StopBits};

/// A UART-based serial port implementation.
///
/// The port will be closed when the value is dropped.
pub struct Uart {
    pub(super) fd: RawFd,
}

impl Uart {
    /// Open an UART device.
    ///
    /// The UART device is openend with the most common settings found on UART
    /// configurations. For more fine grained controler use the serial builder.
    ///
    /// ## Errors
    ///
    /// * `NoDevice` if the device could not be opened. This could indicate that the device is
    ///   already in use.
    /// * `InvalidInput` if `port` is not a valid device name.
    /// * `Io` for any other error while opening or initializing the device.
    pub fn open(path: &Path) -> super::Result<Self> {
        Builder::new(path)
            .unwrap()
            .set_baud_rate(BaudRate::Baud115200)
            .unwrap()
            .set_parity(Parity::ParityNone)
            .set_stop_bits(StopBits::Stop1)
            .set_flow_control(FlowControl::FlowNone)
            .build()
    }
}

impl Drop for Uart {
    fn drop(&mut self) {
        // TODO: Remove exclusive lock
        // #![allow(unused_must_use)]
        // ioctl::tiocnxcl(self.fd);

        unsafe {
            libc::close(self.fd);
        }
    }
}

impl AsRawFd for Uart {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl io::Read for Uart {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = unsafe {
            libc::read(
                self.fd,
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
                self.fd,
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
        termios::tcdrain(self.fd)
    }
}
