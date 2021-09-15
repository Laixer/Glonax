use std::{
    ffi::CString,
    os::unix::prelude::{OsStrExt, RawFd},
    path::Path,
    time::Duration,
};

use libc::EINVAL;
use termios::{tcflush, tcsetattr, Termios};

use crate::{BaudRate, CharSize, FlowControl, Parity, StopBits};

pub struct Builder {
    fd: RawFd,
    termios: Termios,
    timeout: Duration,
    exclusive: bool,
}

impl Builder {
    pub(crate) fn new(path: &Path) -> super::Result<Self> {
        use libc::{O_NONBLOCK, O_RDWR};

        let cstr = match CString::new(path.as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(super::error::from_raw_os_error(EINVAL)),
        };

        let fd = unsafe { libc::open(cstr.as_ptr(), O_RDWR | O_NONBLOCK, 0) };
        if fd < 0 {
            return Err(super::error::last_os_error());
        }

        Self::from_fd(fd)
    }

    fn from_fd(fd: RawFd) -> super::Result<Self> {
        use libc::{
            CLOCAL, CREAD, ECHO, ECHOE, ECHOK, ECHONL, ICANON, ICRNL, IEXTEN, IGNBRK, IGNCR, INLCR,
            ISIG, OPOST, VMIN, VTIME,
        };

        let mut termios = match Termios::from_fd(fd) {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        // Setup TTY for binary serial port access.
        termios.c_iflag &= !(INLCR | IGNCR | ICRNL | IGNBRK);
        termios.c_oflag &= !OPOST;
        termios.c_cflag |= CREAD | CLOCAL;
        termios.c_lflag &= !(ICANON | ECHO | ECHOE | ECHOK | ECHONL | ISIG | IEXTEN);

        termios.c_cc[VMIN] = 1;
        termios.c_cc[VTIME] = 0;

        Ok(Self {
            fd,
            termios,
            timeout: Duration::from_millis(100),
            exclusive: true,
        })
    }

    #[inline]
    pub fn set_timeout(mut self, timeout: Duration) -> super::Result<Self> {
        self.timeout = timeout;
        Ok(self)
    }

    #[inline]
    pub fn set_exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    pub fn set_baud_rate(mut self, baud_rate: BaudRate) -> super::Result<Self> {
        use termios::cfsetspeed;
        use termios::os::linux::{
            B1000000, B1152000, B1500000, B2000000, B2500000, B3000000, B3500000, B4000000,
            B460800, B500000, B576000, B921600,
        };
        use termios::os::target::{B115200, B230400, B57600};
        use termios::{
            B110, B1200, B134, B150, B1800, B19200, B200, B2400, B300, B38400, B4800, B50, B600,
            B75, B9600,
        };

        let baud = match baud_rate {
            BaudRate::BaudOther(50) => B50,
            BaudRate::BaudOther(75) => B75,
            BaudRate::Baud110 => B110,
            BaudRate::BaudOther(134) => B134,
            BaudRate::BaudOther(150) => B150,
            BaudRate::BaudOther(200) => B200,
            BaudRate::Baud300 => B300,
            BaudRate::Baud600 => B600,
            BaudRate::Baud1200 => B1200,
            BaudRate::BaudOther(1800) => B1800,
            BaudRate::Baud2400 => B2400,
            BaudRate::Baud4800 => B4800,
            BaudRate::Baud9600 => B9600,
            BaudRate::Baud19200 => B19200,
            BaudRate::Baud38400 => B38400,
            BaudRate::Baud57600 => B57600,
            BaudRate::Baud115200 => B115200,
            BaudRate::BaudOther(230400) => B230400,
            BaudRate::BaudOther(460800) => B460800,
            BaudRate::BaudOther(500000) => B500000,
            BaudRate::BaudOther(576000) => B576000,
            BaudRate::BaudOther(921600) => B921600,
            BaudRate::BaudOther(1000000) => B1000000,
            BaudRate::BaudOther(1152000) => B1152000,
            BaudRate::BaudOther(1500000) => B1500000,
            BaudRate::BaudOther(2000000) => B2000000,
            BaudRate::BaudOther(2500000) => B2500000,
            BaudRate::BaudOther(3000000) => B3000000,
            BaudRate::BaudOther(3500000) => B3500000,
            BaudRate::BaudOther(4000000) => B4000000,
            BaudRate::BaudOther(_) => return Err(super::error::from_raw_os_error(EINVAL)),
        };

        match cfsetspeed(&mut self.termios, baud) {
            Ok(()) => Ok(self),
            Err(err) => Err(err.into()),
        }
    }

    pub fn set_char_size(mut self, char_size: CharSize) -> Self {
        use termios::{CS5, CS6, CS7, CS8, CSIZE};

        let size = match char_size {
            CharSize::Bits5 => CS5,
            CharSize::Bits6 => CS6,
            CharSize::Bits7 => CS7,
            CharSize::Bits8 => CS8,
        };

        self.termios.c_cflag &= !CSIZE;
        self.termios.c_cflag |= size;

        self
    }

    pub fn set_parity(mut self, parity: Parity) -> Self {
        use termios::{IGNPAR, INPCK, PARENB, PARODD};

        match parity {
            Parity::ParityNone => {
                self.termios.c_cflag &= !(PARENB | PARODD);
                self.termios.c_iflag &= !INPCK;
                self.termios.c_iflag |= IGNPAR;
            }
            Parity::ParityOdd => {
                self.termios.c_cflag |= PARENB | PARODD;
                self.termios.c_iflag |= INPCK;
                self.termios.c_iflag &= !IGNPAR;
            }
            Parity::ParityEven => {
                self.termios.c_cflag &= !PARODD;
                self.termios.c_cflag |= PARENB;
                self.termios.c_iflag |= INPCK;
                self.termios.c_iflag &= !IGNPAR;
            }
        };

        self
    }

    pub fn set_stop_bits(mut self, stop_bits: StopBits) -> Self {
        use termios::CSTOPB;

        match stop_bits {
            StopBits::Stop1 => self.termios.c_cflag &= !CSTOPB,
            StopBits::Stop2 => self.termios.c_cflag |= CSTOPB,
        };

        self
    }

    pub fn set_flow_control(mut self, flow_control: FlowControl) -> Self {
        use termios::os::target::CRTSCTS;
        use termios::{IXOFF, IXON};

        match flow_control {
            FlowControl::FlowNone => {
                self.termios.c_iflag &= !(IXON | IXOFF);
                self.termios.c_cflag &= !CRTSCTS;
            }
            FlowControl::FlowSoftware => {
                self.termios.c_iflag |= IXON | IXOFF;
                self.termios.c_cflag &= !CRTSCTS;
            }
            FlowControl::FlowHardware => {
                self.termios.c_iflag &= !(IXON | IXOFF);
                self.termios.c_cflag |= CRTSCTS;
            }
        };

        self
    }

    pub fn build(self) -> super::Result<crate::Uart> {
        use libc::{ioctl, TCIOFLUSH, TCSANOW, TIOCEXCL};

        // Claim exclusive access to serial device.
        if self.exclusive {
            let ret = unsafe { ioctl(self.fd, TIOCEXCL) };
            if ret < 0 {
                return Err(super::error::last_os_error());
            }
        }

        // Write the terminal settings.
        if let Err(err) = tcsetattr(self.fd, TCSANOW, &self.termios) {
            return Err(err.into());
        }

        // Flush buffers.
        if let Err(err) = tcflush(self.fd, TCIOFLUSH) {
            return Err(err.into());
        }

        crate::Uart::from_impl(crate::imp::Uart(self.fd))
    }
}
