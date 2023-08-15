use glonax::{Configurable, GlobalConfig, InstanceConfig};

#[derive(Clone, Debug)]
pub(crate) struct AgentConfig {
    /// Remote network address.
    pub address: String,
    /// Local instance configuration.
    pub instance: InstanceConfig,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for AgentConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
