use std::error;

pub type Result<T> = std::result::Result<T, DeviceError>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The device is not available.
    ///
    /// This could indicate that the device is in use by another process or is
    /// not connected to the host.
    NoSuchDevice(std::path::PathBuf),

    /// One or multiple parameters were incorrect.
    InvalidInput,

    /// An I/O error occured.
    ///
    /// The type of I/O error is determined by the inner `io::ErrorKind`.
    Io(std::io::ErrorKind),
}

#[derive(Debug)]
pub struct DeviceError {
    /// Device name.
    device: String,
    /// Error kind.
    kind: ErrorKind,
}

impl DeviceError {
    // TODO: limit to crate
    pub fn no_such_device(device: String, path: &std::path::Path) -> Self {
        Self {
            device,
            kind: ErrorKind::NoSuchDevice(path.to_path_buf()),
        }
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::NoSuchDevice(path) => {
                write!(
                    fmt,
                    "{}: no such device: {}",
                    self.device,
                    path.to_str().unwrap()
                )
            }
            ErrorKind::InvalidInput => fmt.write_str("invalid device parameters"),
            ErrorKind::Io(_) => fmt.write_str("io error"),
        }
    }
}

impl error::Error for DeviceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl DeviceError {
    /// Map error from `serial::Error` onto device error.
    pub(super) fn from_serial(
        device: String,
        path: &std::path::Path,
        error: glonax_serial::Error,
    ) -> Self {
        Self {
            device,
            kind: match error.kind() {
                glonax_serial::ErrorKind::NoDevice => ErrorKind::NoSuchDevice(path.to_path_buf()),
                glonax_serial::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
                glonax_serial::ErrorKind::Io(ioe) => ErrorKind::Io(ioe),
            },
        }
    }

    pub(super) fn from_session(device: String, error: glonax_ice::SessionError) -> Self {
        Self {
            device,
            kind: match error {
                glonax_ice::SessionError::SpuriousAddress => ErrorKind::InvalidInput,
                glonax_ice::SessionError::Incomplete => ErrorKind::InvalidInput,
                glonax_ice::SessionError::InvalidData => ErrorKind::InvalidInput,
                glonax_ice::SessionError::FrameParseError(_) => ErrorKind::InvalidInput,
                glonax_ice::SessionError::IoError(ioe) => ErrorKind::Io(ioe.kind()),
            },
        }
    }
}
