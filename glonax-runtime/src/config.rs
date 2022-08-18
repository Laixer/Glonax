use std::path::Path;

use serde::Deserialize;

pub trait Configurable: Clone {
    fn global(&self) -> &GlobalConfig;
}

#[derive(Clone, Debug)]
pub struct ProgramConfig {
    /// Whether autopilot is enabled.
    pub enable_autopilot: bool,

    /// Number of programs to queue.
    pub program_queue: usize,

    /// Number of programs to queue.
    pub program_id: Option<i32>,

    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for ProgramConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}

impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            enable_autopilot: true,
            program_queue: 1024,
            program_id: None,
            global: Default::default(),
        }
    }
}

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

/// Glonax global configuration.
#[derive(Clone, Deserialize, Debug)]
pub struct GlobalConfig {
    /// CAN network interface.
    #[serde(default = "GlobalConfig::default_interface")]
    pub interface: String,

    /// Whether tracing is enabled.
    #[serde(default)]
    pub enable_trace: bool,

    /// Whether motion is enabled.
    #[serde(default = "GlobalConfig::enable_motion")]
    pub enable_motion: bool,

    /// Whether motion is slowed down.
    #[serde(default)]
    pub slow_motion: bool,

    /// Whether the application runs as daemon.
    #[serde(default)]
    pub daemon: bool,

    /// Runtime workers.
    #[serde(default = "GlobalConfig::runtime_workers")]
    pub runtime_workers: usize,
}

impl Configurable for GlobalConfig {
    fn global(&self) -> &GlobalConfig {
        self
    }
}

impl GlobalConfig {
    /// Read configuration from first existing file in the list.
    ///
    /// Values not configured in the configuration file will be set to their
    /// default value. If none of the provided files exist then the default
    /// configuration will be used.
    pub fn try_from_file<T: AsRef<Path>>(config_location_list: Vec<T>) -> std::io::Result<Self> {
        config_location_list
            .iter()
            .filter(|p| p.as_ref().exists())
            .nth(0)
            .map_or_else(|| Ok(Self::default()), |f| Self::from_file(f))
    }

    /// Read configuration from file.
    ///
    /// Values not configured in the configuration file will be set to their
    /// default value.
    pub fn from_file<T: AsRef<Path>>(path: T) -> std::io::Result<Self> {
        use std::io::Read;

        let mut config_file = std::fs::File::open(path)?;

        let mut buffer = Vec::new();

        config_file.read_to_end(&mut buffer)?;

        Ok(toml::from_slice(&buffer)?)
    }

    #[inline]
    fn default_interface() -> String {
        String::new()
    }

    #[inline]
    fn enable_motion() -> bool {
        true
    }

    #[inline]
    fn runtime_workers() -> usize {
        4
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}
