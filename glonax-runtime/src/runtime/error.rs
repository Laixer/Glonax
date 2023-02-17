use std::{error, fmt, io};

use crate::device::DeviceError;

#[derive(Debug)]
pub enum Error {
    /// Generic error ocurring from another subsystem.
    Generic(String),
    /// No motion device was found on the network.
    MotionDeviceNotFound,
    /// No core device was found on the network.
    CoreDeviceNotFound,
    /// Timeout reached while contacting network nodes.
    NetworkTimeout,
    /// Indicates an unhandled error with a device.
    Device(DeviceError),
    /// An I/O error occured.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Generic(e) => write!(f, "{}", e),
            Error::Device(e) => write!(f, "{}", e),
            Error::MotionDeviceNotFound => write!(f, "no motion device was found on the network"),
            Error::CoreDeviceNotFound => write!(f, "no core device was found on the network"),
            Error::NetworkTimeout => write!(f, "timeout reached while contacting network nodes"),
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
