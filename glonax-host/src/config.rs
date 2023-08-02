use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct HostConfig {
    /// Refresh interval in milliseconds.
    pub interval: u64,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for HostConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
