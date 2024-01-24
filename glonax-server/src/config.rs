use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug, Default)]
pub struct ProxyConfig {
    /// Server network address.
    pub address: String,
    /// CAN network interface.
    pub interface: Vec<String>,
    /// Refresh host service interval in milliseconds.
    pub host_interval: u64,
    /// Serial device for NMEA data.
    pub nmea_device: Option<std::path::PathBuf>,
    /// Serial baud rate for NMEA data.
    pub nmea_baud_rate: usize,
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
