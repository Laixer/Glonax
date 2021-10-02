use super::{node::IoNode, Device, DeviceDescriptor};

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

    pub fn claimed(&self) -> &Vec<ResourceClaim> {
        &self.io_node_list
    }

    /// Returned claimed I/O devices.
    pub(crate) fn claimed(&self) -> &Vec<IoNode> {
        &self.io_node_list
    }

    /// Register a device with the device manager.
    #[inline]
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

    /// Call `idle_time` method on the next device.
    pub async fn idle_time(&mut self) {
        // Ignore any held device locks.
        if let Ok(mut device) = self.next().try_lock() {
            trace!("Appoint idle time slice to device: {}", device.name());

            device.idle_time().await;
        }
    }
}
