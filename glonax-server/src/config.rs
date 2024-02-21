use glonax::Configurable;

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
    /// Network address to listen on.
    #[serde(default = "ServerConfig::default_listen")]
    pub listen: String,
    /// Maximum number of connections.
    #[serde(default = "ServerConfig::default_max_connections")]
    pub max_connections: usize,
}

impl ServerConfig {
    fn default_listen() -> String {
        "127.0.0.1:30051".to_owned()
    }

    fn default_max_connections() -> usize {
        10
    }
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct NmeaConfig {
    /// Serial device for NMEA data.
    pub device: std::path::PathBuf,
    /// Serial baud rate for NMEA data.
    pub baud_rate: usize,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct HostConfig {
    // Host update interval.
    pub interval: u64,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct SimulationConfig {
    /// Enable simulation mode.
    #[serde(default)]
    pub enabled: bool,
    /// Enable simulation jitter.
    pub jitter: bool,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct CanConfig {
    /// CAN network interface.
    pub interface: String,
    /// Address.
    pub address: u8,
    /// Driver configuration.
    pub driver: Vec<CanDriverConfig>,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct CanDriverConfig {
    /// Driver identifier.
    pub id: u8,
    /// Driver type.
    #[serde(rename = "type")]
    pub driver_type: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde_derive::Deserialize)]
pub enum OperationMode {
    /// Normal operation mode.
    Normal,
    /// Pilot restriction mode.
    PilotRestrict,
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationMode::Normal => write!(f, "normal"),
            OperationMode::PilotRestrict => write!(f, "pilot-restrict"),
        }
    }
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct InstanceConfig {
    /// Instance unique identifier.
    pub id: String,
    /// Machine model.
    pub model: String,
    /// Machine machine type.
    pub ty: glonax::core::MachineType,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct Config {
    pub mode: OperationMode,
    /// Machine instance.
    pub instance: InstanceConfig,
    /// NMEA configuration.
    pub nmea: Option<NmeaConfig>,
    /// Host configuration.
    pub host: HostConfig,
    /// Simulation configuration.
    pub simulation: SimulationConfig,
    /// Server configuration.
    pub server: ServerConfig,
    /// J1939 configuration.
    pub j1939: Vec<CanConfig>,
}

impl Configurable for Config {}
