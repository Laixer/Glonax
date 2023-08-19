use glonax::{core::Instance, Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct AgentConfig {
    /// Remote network address.
    pub address: String,
    /// Probe interval in seconds.
    pub interval: u64,
    /// Instance configuration.
    pub instance: Option<Instance>,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for AgentConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
