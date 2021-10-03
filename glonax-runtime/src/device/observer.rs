use crate::device::{DeviceError, ErrorKind};

use super::{
    host::HostInterface, node::IoNode, DeviceDescriptor, DeviceManager, IoDevice, IoDeviceProfile,
};

pub struct Observer<'a> {
    pub(super) manager: &'a mut DeviceManager,
}

impl<'a> Observer<'a> {
    fn host_elect_io_nodes<T>(&self) -> Vec<IoNode>
    where
        T: IoDevice + 'static,
        T::DeviceProfile: IoDeviceProfile,
    {
        HostInterface::new()
            .elect::<T::DeviceProfile>()
            .filter(|p| {
                for c in self.manager.claimed() {
                    if p.as_path() == c.as_path() {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Discover device instances of the device type.
    ///
    /// Return the first matching I/O device instance or none.
    pub async fn scan_once<T>(&mut self) -> Option<DeviceDescriptor<T>>
    where
        T: IoDevice + 'static,
        T::DeviceProfile: IoDeviceProfile,
    {
        let mut io_node_list = self.host_elect_io_nodes::<T>();

        if io_node_list.is_empty() {
            return None;
        }

        let io_node = io_node_list.remove(0);

        trace!("Elected I/O node: {}", io_node);

        match io_node.try_construe_device::<T>().await {
            Ok(device) => {
                let n = IoNode::from(device.lock().await.node_path());

                self.manager.register_io_device(device.clone(), n);
                Some(device)
            }
            Err(DeviceError {
                kind: ErrorKind::InvalidDeviceFunction,
                ..
            }) => None,
            Err(e) => {
                warn!("{:?}", e);
                None
            }
        }
    }

    /// Discover device instances of the device type.
    ///
    /// Returns a list of construed I/O device instances.
    pub async fn scan<T>(&mut self) -> Vec<DeviceDescriptor<T>>
    where
        T: IoDevice + 'static,
        T::DeviceProfile: IoDeviceProfile,
    {
        let mut construed_devices = Vec::new();

        for io_node in self.host_elect_io_nodes::<T>() {
            trace!("Elected I/O node: {}", io_node);

            match io_node.try_construe_device::<T>().await {
                Ok(device) => {
                    let n = IoNode::from(device.lock().await.node_path());

                    self.manager.register_io_device(device.clone(), n);
                    construed_devices.push(device);
                }
                Err(DeviceError {
                    kind: ErrorKind::InvalidDeviceFunction,
                    ..
                }) => continue,
                Err(e) => {
                    warn!("{:?}", e);
                    continue;
                }
            }
        }

        construed_devices
    }
}
