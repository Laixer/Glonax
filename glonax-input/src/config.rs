use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct InputConfig {
    /// Remote network address.
    pub address: String,
    /// Input device.
    pub device: String,
    /// Input commands will translate to the full motion range.
    pub full_motion: bool,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for InputConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
