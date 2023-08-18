use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct DumpConfig {
    /// Remote network address.
    pub address: String,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for DumpConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
