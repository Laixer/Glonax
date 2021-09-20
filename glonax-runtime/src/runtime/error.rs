use std::{error, fmt};

use crate::device::DeviceError;

#[derive(Debug)]
pub enum Error {
    WorkspaceInUse,
    Device(DeviceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Device(e) => write!(f, "{}", e),
            Error::WorkspaceInUse => write!(f, "Workspace is in use by another instance"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
