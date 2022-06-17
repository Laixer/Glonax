use std::{
    fmt::Display,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::device::DeviceError;

use super::{DeviceDescriptor, UserDevice};

pub(crate) struct Applicant {
    pub(super) sysname: String,
    pub(super) node_path: Option<PathBuf>,
}

impl Applicant {
    pub(crate) fn new(name: &str, node_path: Option<&Path>) -> Self {
        Self {
            sysname: name.to_owned(),
            node_path: node_path.map(|p| p.to_path_buf()),
        }
    }

    #[inline]
    pub fn sysname(&self) -> &str {
        self.sysname.as_str()
    }

    /// Try to construe a device from an I/O node.
    ///
    /// This function will return a shared handle to the device.
    /// This is the recommended way to instantiate and claim IO devices.
    pub(crate) async fn construe_device<T, F>(self, func: F) -> super::Result<DeviceDescriptor<T>>
    where
        T: UserDevice,
        F: FnOnce(&String, &Option<PathBuf>) -> T,
    {
        if let Some(node_path) = &self.node_path {
            if !node_path.exists() {
                return Err(DeviceError::no_such_device(
                    T::NAME.to_owned(),
                    node_path.as_path(),
                ));
            }
        }

        let device = func(&self.sysname, &self.node_path);

        info!("Device driver '{}' is initialized", T::NAME.to_owned());

        Ok(Arc::new(tokio::sync::Mutex::new(device)))
    }

    /// Try to construe a device from an I/O node.
    ///
    /// This function will return a shared handle to the device.
    /// This is the recommended way to instantiate and claim IO devices.
    pub(crate) async fn try_construe_device<T>(
        self,
        timeout: Duration,
    ) -> super::Result<DeviceDescriptor<T>>
    where
        T: UserDevice,
    {
        if let Some(node_path) = &self.node_path {
            if !node_path.exists() {
                return Err(DeviceError::no_such_device(
                    T::NAME.to_owned(),
                    node_path.as_path(),
                ));
            }
        }

        let mut device = if let Some(node_path) = &self.node_path {
            T::from_node_path(&self.sysname, node_path.as_path()).await?
        } else {
            T::from_sysname(&self.sysname).await?
        };

        debug!(
            "Probe device driver '{}' for applicant '{}'",
            T::NAME.to_owned(),
            self
        );

        // Only probe the I/O device for so long. If the timeout is reached the device
        // is not considered a match, even though it could have been given more time.
        if tokio::time::timeout(timeout, device.probe()).await.is_err() {
            return Err(DeviceError::timeout(T::NAME.to_owned()));
        }

        info!("Device driver '{}' is initialized", T::NAME.to_owned());

        Ok(Arc::new(tokio::sync::Mutex::new(device)))
    }
}

impl Display for Applicant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(node_path) = &self.node_path {
            write!(f, "{} [{}]", self.sysname, node_path.to_str().unwrap())
        } else {
            write!(f, "{}", self.sysname)
        }
    }
}
