use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::device::DeviceError;

use super::{DeviceDescriptor, IoDevice};

pub(crate) struct IoNode {
    pub(super) node_path: PathBuf,
}

impl From<&Path> for IoNode {
    fn from(value: &Path) -> Self {
        Self {
            node_path: value.to_path_buf(),
        }
    }
}

impl IoNode {
    #[inline]
    pub fn as_path(&self) -> &Path {
        &self.node_path
    }

    // FUTURE: path.try_exists()
    /// Checks to see if the node path exists in the operating system.
    ///
    /// This methdd only checks that the I/O resource exist, but not if it is
    /// accessible.
    #[inline]
    pub(crate) fn exists(&self) -> bool {
        self.node_path.exists()
    }

    /// Try to construe a device from an I/O node.
    ///
    /// This function will return a shared handle to the device.
    /// This is the recommended way to instantiate and claim IO devices.
    pub(crate) async fn try_construe_device<T: IoDevice>(
        self,
        timeout: Duration,
    ) -> super::Result<DeviceDescriptor<T>> {
        // Every IO device must have an I/O resource on disk. If that node does
        // not exist then exit right here. Doing this early on will ensure that
        // every IO device returns the same error if the IO resource was not found.
        if !self.exists() {
            return Err(DeviceError::no_such_device(
                T::NAME.to_owned(),
                &self.node_path,
            ));
        }

        let mut io_device = T::from_node_path(&self.node_path).await?;

        debug!(
            "Probe I/O device '{}' from I/O node: {}",
            T::NAME.to_owned(),
            self
        );

        // Only probe the device for so long.
        if let Err(_) = tokio::time::timeout(timeout, io_device.probe()).await {
            return Err(DeviceError::timeout(T::NAME.to_owned()));
        }

        info!("I/O Device '{}' is construed", T::NAME.to_owned());

        Ok(std::sync::Arc::new(tokio::sync::Mutex::new(io_device)))
    }
}

impl Display for IoNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node_path.to_str().unwrap())
    }
}
