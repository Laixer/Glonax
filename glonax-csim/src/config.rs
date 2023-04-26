use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct SimConfig {
    /// CAN network interface.
    pub interface: Vec<String>,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for SimConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
