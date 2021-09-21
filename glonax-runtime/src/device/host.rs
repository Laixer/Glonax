use super::IoDeviceProfile;

pub struct HostInterface {
    /// Device enumerator.
    enumerator: udev::Enumerator,
}

impl HostInterface {
    /// Construct new host interface.
    pub fn new() -> Self {
        Self {
            enumerator: udev::Enumerator::new().unwrap(),
        }
    }

    /// Host global device filter.
    ///
    /// The filter ignores incompatible device nodes from the host device list.
    /// None of the devices should match a global excluded device node.
    fn global_device_filter(device: &udev::Device) -> bool {
        // Uninitialized devices cannot be used.
        if !device.is_initialized() {
            return false;
        }

        // Ignore device which have been claimed by a driver.
        if device.driver().is_some() {
            return false;
        }

        // Devices will use the `devnode` for communication. Although not strictly
        // necessary, `sysnodes` may be inaccessible. `devnodes` are often setup
        // with the correct permissions and umask.
        if device.devnode().is_none() {
            return false;
        }

        true
    }

    /// Elect device candidates for the provided I/O device.
    ///
    /// Elected device nodes are matched based on the I/O device profile criteria.
    /// This method returns an iterator with the elected device candidates.
    pub fn elect<T: IoDeviceProfile + 'static>(
        &mut self,
    ) -> impl Iterator<Item = crate::device::claim::ResourceClaim> + '_ {
        let subsystem: &str = T::CLASS.into();

        trace!("Selecting subsystem '{}'", subsystem);

        self.enumerator.match_is_initialized().unwrap();
        self.enumerator.match_subsystem(subsystem).unwrap();

        for (key, value) in T::properties() {
            self.enumerator.match_property(key, value).unwrap();
        }

        self.enumerator
            .scan_devices()
            .unwrap()
            .filter(Self::global_device_filter)
            .filter(T::filter)
            .map(|d| crate::device::claim::ResourceClaim::new(d.syspath(), d.devnode().unwrap()))
    }
}
