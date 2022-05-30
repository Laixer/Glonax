use super::{
    host::HostInterface, node::Applicant, DeviceDescriptor, DeviceManager, IoDeviceProfile,
    UserDevice,
};

pub struct Observer<'a> {
    pub(super) manager: &'a mut DeviceManager,
}

impl<'a> Observer<'a> {
    fn host_elect_applicants<T>(&self) -> Vec<Applicant>
    where
        T: super::UserDevice + 'static,
        T::DeviceRuleset: IoDeviceProfile,
    {
        let node_list: Vec<Applicant> = HostInterface::new()
            .select_devices::<T::DeviceRuleset>()
            .filter(|a| !self.manager.device_claimed(a.sysname()))
            .collect();

        trace!("Device driver applicants after scan: {}", node_list.len());

        node_list
    }

    // TODO: Merge into scan()
    /// Discover device instances of the device type.
    ///
    /// Return the first matching I/O device instance or none.
    pub async fn scan_first<T>(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<DeviceDescriptor<T>>
    where
        T: UserDevice + 'static,
        T::DeviceRuleset: IoDeviceProfile,
    {
        let mut applicants = self.host_elect_applicants::<T>();

        loop {
            if applicants.is_empty() {
                return None;
            }

            let applicant = applicants.remove(0);

            trace!("Elected applicant: {}", applicant);

            // TODO: Can be optimized.
            match applicant.try_construe_device::<T>(timeout).await {
                Ok(device) => {
                    self.manager
                        .register_device_driver(device.clone(), device.lock().await.sysname());

                    break Some(device);
                }
                Err(e) => {
                    warn!("Device not construed: {}", e);
                }
            }
        }
    }

    /// Discover device instances of the device type.
    ///
    /// Returns a list of construed I/O device instances.
    pub async fn scan<T>(&mut self, timeout: std::time::Duration) -> Vec<DeviceDescriptor<T>>
    where
        T: UserDevice + 'static,
        T::DeviceRuleset: IoDeviceProfile,
    {
        let mut construed_devices = Vec::new();

        for applicant in self.host_elect_applicants::<T>() {
            trace!("Elected applicant: {}", applicant);

            match applicant.try_construe_device::<T>(timeout).await {
                Ok(device) => {
                    self.manager
                        .register_device_driver(device.clone(), device.lock().await.sysname());
                    construed_devices.push(device);
                }
                Err(e) => {
                    warn!("Device not construed: {}", e);
                }
            }
        }

        construed_devices
    }
}
