use glonax::{core::Instance, Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct AgentConfig {
    /// Probe interval in seconds.
    pub interval: u64,
    /// Instance configuration.
    pub instance: Instance,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for AgentConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
