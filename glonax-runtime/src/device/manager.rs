use super::{node::IoNode, observer::Observer, Device, DeviceDescriptor};

/// Device manager.
///
/// The device manager keeps track of registered devices. Methods on the devices
/// are available on the device manager. On ever device method call we'll select
/// a new device from the manager. This allows the caller to automatically cycle
/// through all devices when the same method is called repeatedly.
///
/// By default devices selection is based on a simple round robin distribution.
pub struct DeviceManager {
    device_list: Vec<DeviceDescriptor<dyn Device>>,
    io_node_list: Vec<IoNode>,
    index: usize,
}

impl DeviceManager {
    /// Construct new device manager.
    pub fn new() -> Self {
        Self {
            device_list: Vec::new(),
            io_node_list: Vec::new(),
            index: 0,
        }
    }

    /// Create a new I/O node observer.
    #[inline]
    pub fn observer(&mut self) -> Observer {
        Observer { manager: self }
    }

    /// Returned claimed I/O devices.
    #[inline]
    pub(crate) fn claimed(&self) -> &Vec<IoNode> {
        &self.io_node_list
    }

    /// Register a device with the device manager.
    pub(crate) fn register_io_device(
        &mut self,
        device: DeviceDescriptor<dyn Device>,
        io_node: IoNode,
    ) {
        self.device_list.push(device);
        self.io_node_list.push(io_node);
    }

    /// Select the next device from the device list.
    ///
    /// Re-entering this method is likely to yield a different result.
    fn next(&mut self) -> &DeviceDescriptor<dyn Device> {
        self.index += 1;
        self.device_list
            .get(self.index % self.device_list.len())
            .unwrap()
    }
}
