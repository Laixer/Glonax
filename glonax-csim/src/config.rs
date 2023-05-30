use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub struct SimConfig {
    /// CAN network interface.
    pub interface: Vec<String>,
    /// Randomize the start position.
    pub randomize_start: bool,
    /// Introcude jitter in the sensor data.
    pub jitter: bool,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for SimConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}
