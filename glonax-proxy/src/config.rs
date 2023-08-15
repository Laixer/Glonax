use glonax::{Configurable, GlobalConfig, InstanceConfig};

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    /// Server network address.
    pub address: String,
    /// CAN network interface.
    pub interface: String,
    /// Refresh host service interval in milliseconds.
    pub host_interval: u64,
    /// Local instance configuration.
    pub instance: InstanceConfig,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for ProxyConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
