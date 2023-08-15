use glonax::{Configurable, GlobalConfig, InstanceConfig};

#[derive(Clone, Debug)]
pub struct GnssConfig {
    /// Serial device.
    pub device: std::path::PathBuf,
    /// Serial baud rate.
    pub baud_rate: usize,
    /// Local instance configuration.
    pub instance: InstanceConfig,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for GnssConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
