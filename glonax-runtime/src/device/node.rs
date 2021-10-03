use std::path::{Path, PathBuf};

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

    /// Checks to see if the node path exists in the operating system.
    ///
    /// This methdd only checks that the IO resource exist, but not if it is
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
    ) -> super::Result<DeviceDescriptor<T>> {
        // FUTURE: path.try_exists()
        // Every IO device must have an IO resource on disk. If that node does
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
            self.node_path.to_str().unwrap()
        );

        io_device.probe().await?;

        info!("I/O Device '{}' is construed", T::NAME.to_owned());

        Ok(std::sync::Arc::new(tokio::sync::Mutex::new(io_device)))
    }
}
