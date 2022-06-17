use crate::device::host::HostInterface;

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

    pub(crate) fn startup(&self) {
        // TODO: List all drivers.
        trace!("Registered device drivers: {}", self.driver_list.len());
    }

    #[inline]
    pub fn device_claimed(&self, name: &str) -> bool {
        self.device_list.contains(&name.to_owned())
    }

    pub(crate) fn register_driver<T>(&mut self, dev: T) -> DeviceDescriptor<T>
    where
        T: Device + 'static,
    {
        let device = std::sync::Arc::new(tokio::sync::Mutex::new(dev));

        trace!("Register driver without device");

        self.driver_list.push(device.clone());
        device
    }

    pub(crate) async fn register_device_driver_first<T, F>(
        &mut self,
        func: F,
    ) -> Option<DeviceDescriptor<T>>
    where
        T: super::UserDevice + 'static,
        T::DeviceRuleset: super::IoDeviceProfile,
        F: FnOnce(&String, &Option<std::path::PathBuf>) -> T + Copy,
    {
        for applicant in self.host_elect_applicants::<T>() {
            trace!("Elected applicant: {}", applicant);

            match applicant.construe_device::<T, F>(func).await {
                Ok(device) => {
                    {
                        let devx = &device.lock().await;
                        let sysname = devx.sysname();

                        // TODO: Also want the device name.
                        trace!("Register driver with device '{}'", sysname.to_owned());

                        self.driver_list.push(device.clone());
                        self.device_list.push(sysname.to_owned());
                    }

                    return Some(device);
                }
                Err(e) => {
                    warn!("Device not construed: {}", e);
                }
            }
        }

        None
    }

    pub(crate) async fn try_register_device_driver_first<T>(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<DeviceDescriptor<T>>
    where
        T: super::UserDevice + 'static,
        T::DeviceRuleset: super::IoDeviceProfile,
    {
        for applicant in self.host_elect_applicants::<T>() {
            trace!("Elected applicant: {}", applicant);

            match applicant.try_construe_device::<T>(timeout).await {
                Ok(device) => {
                    {
                        let devx = &device.lock().await;
                        let sysname = devx.sysname();

                        // TODO: Also want the device name.
                        trace!("Register driver with device '{}'", sysname.to_owned());

                        self.driver_list.push(device.clone());
                        self.device_list.push(sysname.to_owned());
                    }

                    return Some(device);
                }
                Err(e) => {
                    warn!("Device not construed: {}", e);
                }
            }
        }

        None
    }

    fn host_elect_applicants<T>(&self) -> Vec<super::applicant::Applicant>
    where
        T: super::UserDevice + 'static,
        T::DeviceRuleset: super::IoDeviceProfile,
    {
        let applicant_list: Vec<crate::device::applicant::Applicant> = HostInterface::new()
            .select_devices::<T::DeviceRuleset>()
            .filter(|a| !self.device_claimed(a.sysname()))
            .collect();

        trace!(
            "Device driver applicants after scan: {}",
            applicant_list.len()
        );

        applicant_list
    }
}
