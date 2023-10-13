use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct AgentConfig {
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for AgentConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
