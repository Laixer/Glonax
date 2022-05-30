use super::{observer::Observer, Device, DeviceDescriptor};

// TODO: Generate I/O node from device.
/// Device manager.
///
/// The device manager keeps track of registered devices. Methods on the devices
/// are available on the device manager. On ever device method call we'll select
/// a new device from the manager. This allows the caller to automatically cycle
/// through all devices when the same method is called repeatedly.
///
/// By default devices selection is based on a simple round robin distribution.
pub struct DeviceManager {
    driver_list: Vec<DeviceDescriptor<dyn Device>>,
    device_list: Vec<String>,
}

impl DeviceManager {
    /// Construct new device manager.
    pub fn new() -> Self {
        Self {
            driver_list: Vec::new(),
            device_list: Vec::new(),
        }
    }

    /// Create a new I/O node observer.
    #[inline]
    pub fn observer(&mut self) -> Observer {
        Observer { manager: self }
    }

    #[inline]
    pub fn device_claimed(&self, name: &str) -> bool {
        self.device_list.contains(&name.to_owned())
    }

    /// Register a device with the device manager.
    pub(crate) fn register_io_device(
        &mut self,
        device: DeviceDescriptor<dyn Device>,
        device_sysname: &str,
    ) {
        self.driver_list.push(device);
        self.device_list.push(device_sysname.to_owned());
    }

    /// Run the device health check.
    ///
    /// The device health check removes any dead I/O nodes so that new devices
    /// can be identified using the same I/O node path.
    pub(crate) async fn health_check(&mut self) {
        // let evict: Vec<usize> = self
        //     .io_node_list
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, node)| !node.exists())
        //     .map(|(idx, _)| idx)
        //     .collect();

        // for idx in evict {
        //     trace!("Evict dead I/O node: {}", self.io_node_list[idx]);

        //     self.device_list.remove(idx);
        //     self.io_node_list.remove(idx);
        // }
    }
}
