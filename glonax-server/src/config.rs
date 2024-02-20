use glonax::Configurable;

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
    /// Network address to listen on.
    pub listen: String,
    /// Maximum number of connections.
    pub max_connections: usize,
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
    pub id: u8,
    #[serde(rename = "type")]
    pub driver_type: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde_derive::Deserialize)]
pub enum OperationMode {
    /// Normal operation mode.
    Normal,
    /// Pilot operation mode.
    Pilot,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct Config {
    pub mode: OperationMode,
    /// Machine instance.
    pub instance: glonax::core::Instance,
    /// NMEA configuration.
    pub nmea: Option<NmeaConfig>,
    /// Host configuration.
    pub host: HostConfig,
    /// Simulation configuration.
    pub simulation: SimulationConfig,
    /// Server configuration.
    pub server: ServerConfig,
    /// CAN configuration.
    pub can: Vec<CanConfig>,
}

impl Configurable for Config {}
