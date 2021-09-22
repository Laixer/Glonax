use std::{error, fmt};

use crate::device::DeviceError;

#[derive(Debug)]
pub enum Error {
    /// Workspace is in use by another instance. Another instance can run but
    /// must use another workspace.
    WorkspaceInUse,
    /// No motion device was found on the network.
    MotionDeviceNotFound,
    /// Indicates an unhandled error with a device.
    Device(DeviceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Device(e) => write!(f, "{}", e),
            Error::MotionDeviceNotFound => write!(f, "no motion device was found on the network"),
            Error::WorkspaceInUse => write!(f, "workspace is in use by another instance"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
