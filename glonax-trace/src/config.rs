use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct TraceConfig {
    /// CAN network interface.
    pub interface: Vec<String>,
    /// Trace items per file.
    pub items_per_file: u32,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for TraceConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
