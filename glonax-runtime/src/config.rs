use std::{env::current_dir, path::PathBuf};

/// Glonax configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether autopilot is enabled.
    pub enable_autopilot: bool,
    /// Whether command devices are enabled.
    pub enable_command: bool,
    /// Whether the foreground terminal can be cancelled.
    pub enable_term_shutdown: bool,
    /// Library worksapce.
    pub workspace: PathBuf,
    /// Motion device resource.
    pub motion_device: String,
    /// Metric device resources.
    pub metric_devices: Vec<String>,
    /// Number of programs to queue.
    pub program_queue: usize,
    /// Runtime workers.
    pub runtime_workers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_autopilot: true,
            enable_command: true,
            enable_term_shutdown: true,
            workspace: current_dir().unwrap().join("data"),
            motion_device: String::new(),
            metric_devices: vec![],
            program_queue: 1024,
            runtime_workers: 8,
        }
    }
}
