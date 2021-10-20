use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Glonax configuration.
#[derive(Clone, Deserialize)]
pub struct Config {
    /// Whether autopilot is enabled.
    #[serde(default)]
    pub enable_autopilot: bool,

    /// Whether input devices are enabled.
    #[serde(default)]
    pub enable_input: bool,

    /// Whether motion is enabled.
    #[serde(default = "Config::enable_motion")]
    pub enable_motion: bool,

    /// Library worksapce.
    #[serde(default = "Config::workspace")]
    pub workspace: PathBuf,

    /// Number of events to queue.
    #[serde(default = "Config::event_queue")]
    pub event_queue: usize,

    /// Number of programs to queue.
    #[serde(default = "Config::program_queue")]
    pub program_queue: usize,

    /// Runtime workers.
    #[serde(default = "Config::runtime_workers")]
    pub runtime_workers: usize,

    /// Runtime idle interval in seconds.
    #[serde(default = "Config::runtime_idle_interval")]
    pub runtime_idle_interval: usize,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration:
            \tAutopilot enabled: {}
            \tInput enabled: {}
            \tMotion enabled: {}
            \tWorkspace: {}
            \tEvent queue size: {}
            \tProgram queue size: {}
            \tRuntime workers: {}
            \tRuntime idle interval: {}",
            self.enable_autopilot,
            self.enable_input,
            self.enable_motion,
            self.workspace.to_str().unwrap(),
            self.event_queue,
            self.program_queue,
            self.runtime_workers,
            self.runtime_idle_interval
        )
    }
}

impl Config {
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
    fn enable_motion() -> bool {
        true
    }

    #[inline]
    fn workspace() -> PathBuf {
        std::env::current_dir().unwrap().join("data")
    }

    #[inline]
    fn event_queue() -> usize {
        32
    }

    #[inline]
    fn program_queue() -> usize {
        1024
    }

    #[inline]
    fn runtime_workers() -> usize {
        8
    }

    #[inline]
    fn runtime_idle_interval() -> usize {
        5
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}
