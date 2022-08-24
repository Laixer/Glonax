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

#[derive(Clone, Debug)]
pub struct EcuConfig {
    /// ECU network bind address.
    pub address: String,

    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for EcuConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}

/// Glonax global configuration.
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    /// CAN network interface.
    pub interface: String,

    /// Whether tracing is enabled.
    pub enable_trace: bool,

    /// Whether motion is enabled.
    pub enable_motion: bool,

    /// Whether motion is slowed down.
    pub slow_motion: bool,

    /// Whether the application runs as daemon.
    pub daemon: bool,

    /// Runtime workers.
    pub runtime_workers: usize,
}

impl Configurable for GlobalConfig {
    fn global(&self) -> &GlobalConfig {
        self
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            interface: String::new(),
            enable_motion: true,
            slow_motion: false,
            enable_trace: false,
            daemon: false,
            runtime_workers: 4,
        }
    }
}
