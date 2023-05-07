pub trait Configurable: Clone {
    /// Get the global configuration
    fn global(&self) -> &GlobalConfig;
}

/// Glonax global configuration.
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    /// Name of the binary.
    pub bin_name: String,
    /// Whether the application runs as daemon.
    pub daemon: bool,
}

impl Configurable for GlobalConfig {
    fn global(&self) -> &GlobalConfig {
        self
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            bin_name: String::new(),
            daemon: false,
        }
    }
}
