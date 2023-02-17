use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct InputConfig {
    /// Input device.
    pub device: String,

    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for InputConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
