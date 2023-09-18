use glonax::{core::Instance, Configurable, GlobalConfig};

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
    /// Instance configuration.
    pub instance: Instance,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for ProxyConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
