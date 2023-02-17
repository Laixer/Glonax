use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct EcuConfig {
    /// CAN network interface.
    pub interface: String,

    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for EcuConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
