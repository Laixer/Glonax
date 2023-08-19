use glonax::{core::Instance, Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct DumpConfig {
    /// Remote network address.
    pub address: String,
    /// Instance configuration.
    pub instance: Option<Instance>,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for DumpConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
