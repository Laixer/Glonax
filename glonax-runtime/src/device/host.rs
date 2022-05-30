use super::{node::Applicant, IoDeviceProfile};

pub struct HostInterface(udev::Enumerator);

impl HostInterface {
    /// Construct new host interface.
    pub fn new() -> Self {
        Self(udev::Enumerator::new().unwrap())
    }

    /// Select device candidates for the provided I/O device.
    ///
    /// Elected device nodes are matched based on the I/O device profile criteria.
    /// This method returns an iterator with the elected device candidates.
    pub(crate) fn select_devices<T: IoDeviceProfile + 'static>(
        &mut self,
    ) -> impl Iterator<Item = Applicant> + '_ {
        let subsystem: &str = T::CLASS.into();

        trace!("Selecting subsystem '{}'", subsystem);

        self.0.match_is_initialized().unwrap();
        self.0.match_subsystem(subsystem).unwrap();

        for (key, value) in T::properties() {
            self.0.match_property(key, value).unwrap();
        }

        self.0
            .scan_devices()
            .unwrap()
            .filter(|device| device.is_initialized() && device.driver().is_none())
            .filter(T::filter)
            .filter(|device| match T::CLASS {
                crate::device::Subsystem::Net => device
                    .attribute_value("carrier")
                    .map_or(false, |a| a == "1"),
                crate::device::Subsystem::Input | crate::device::Subsystem::TTY => {
                    device.devnode().is_some()
                }
                _ => true,
            })
            .map(|device| Applicant::new(device.sysname().to_str().unwrap(), device.devnode()))
    }
}

#[allow(dead_code)]
mod todo {
    pub struct HostMonitor {
        inner: tokio::io::unix::AsyncFd<udev::MonitorSocket>,
    }

    impl HostMonitor {
        pub fn new(monitor: udev::MonitorSocket) -> std::io::Result<Self> {
            Ok(Self {
                inner: tokio::io::unix::AsyncFd::new(monitor)?,
            })
        }

        pub async fn listen(&mut self) -> std::io::Result<udev::Event> {
            loop {
                let mut guard = self.inner.readable_mut().await?;

                let event = guard.get_inner_mut().next();

                guard.clear_ready();

                if let Some(event) = event {
                    break Ok(event);
                }
            }
        }
    }
}
