use std::{env::current_dir, path::PathBuf};

/// Glonax configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether autopilot is enabled.
    pub enable_autopilot: bool,
    /// Whether command devices are enabled.
    pub enable_command: bool,
    /// Library worksapce.
    pub workspace: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_autopilot: true,
            enable_command: true,
            workspace: current_dir().unwrap(),
        }
    }
}
