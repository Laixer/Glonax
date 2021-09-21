use super::{Device, DeviceDescriptor};

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
    index: usize,
}

impl DeviceManager {
    /// Construct new device manager.
    pub fn new() -> Self {
        Self {
            device_list: Vec::new(),
            index: 0,
        }
    }

    /// Register a device with the device manager.
    #[inline]
    pub fn register_device(&mut self, device: DeviceDescriptor<dyn Device>) {
        self.device_list.push(device)
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
