use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for ProxyConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
