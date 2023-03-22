pub trait Configurable: Clone {
    /// Get the global configuration
    fn global(&self) -> &GlobalConfig;
}

/// Glonax global configuration.
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    /// Name of the binary.
    pub bin_name: String,
    /// Whether motion is enabled.
    pub enable_motion: bool,
    /// Whether motion is slowed down.
    pub slow_motion: bool,
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
            enable_motion: true,
            slow_motion: false,
            daemon: false,
        }
    }
}
