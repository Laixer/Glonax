use std::{env::current_dir, path::PathBuf};

/// Glonax configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether autopilot is enabled.
    pub enable_autopilot: bool,
    /// Whether input devices are enabled.
    pub enable_input: bool,
    /// Whether the foreground terminal can be cancelled.
    pub enable_term_shutdown: bool,
    /// Library worksapce.
    pub workspace: PathBuf,
    /// Number of events to queue.
    pub event_queue: usize,
    /// Number of programs to queue.
    pub program_queue: usize,
    /// Runtime workers.
    pub runtime_workers: usize,
    /// Runtime stack size.
    pub runtime_stack_size: usize,
    /// Runtime idle interval in seconds.
    pub runtime_idle_interval: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_autopilot: true,
            enable_input: true,
            enable_term_shutdown: true,
            workspace: current_dir().unwrap().join("data"),
            event_queue: 32,
            program_queue: 1024,
            runtime_workers: 8,
            runtime_stack_size: 8 * 1024 * 1024,
            runtime_idle_interval: 15,
        }
    }
}
