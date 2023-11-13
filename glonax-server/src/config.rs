use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    /// Server network address.
    pub address: String,
    /// CAN network interface.
    pub interface: String,
    /// CAN network interface2.
    pub interface2: Option<String>,
    /// Refresh host service interval in milliseconds.
    pub host_interval: u64,
    /// Serial device.
    pub gnss_device: Option<std::path::PathBuf>,
    /// Serial baud rate.
    pub gnss_baud_rate: usize,
    /// Send a probe to remote host.
    pub probe: bool,
    /// Probe interval in seconds.
    pub probe_interval: u64,
    /// Enable simulation mode.
    pub simulation: bool,
    /// Enable simulation jitter.
    pub simulation_jitter: bool,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for ProxyConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
