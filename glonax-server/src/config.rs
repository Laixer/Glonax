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
pub struct HostConfig {
    // Host update interval.
    #[serde(default = "HostConfig::default_interval")]
    pub interval: u64,
}

impl HostConfig {
    fn default_interval() -> u64 {
        500
    }
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
pub struct J1939Name {
    /// Manufacturer code.
    pub manufacturer_code: u16,
    /// Function instance.
    pub function_instance: u8,
    /// ECU instance.
    pub ecu_instance: u8,
    /// Function.
    pub function: u8,
    /// Vehicle system.
    pub vehicle_system: u8,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct J1939NetConfig {
    /// CAN network interface.
    pub interface: String,
    /// Address.
    pub address: u8,
    /// Name.
    pub name: J1939Name,
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
    #[serde(alias = "normal")]
    Normal,
    /// Pilot restriction mode.
    #[serde(alias = "pilot-restrict")]
    PilotRestrict,
    /// Autonomous operation mode.
    #[serde(alias = "autonomous")]
    Autonomous,
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationMode::Normal => write!(f, "normal"),
            OperationMode::PilotRestrict => write!(f, "pilot-restrict"),
            OperationMode::Autonomous => write!(f, "autonomous"),
        }
    }
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct MachineConfig {
    /// Instance unique identifier.
    pub id: String,
    /// Machine model.
    pub model: String,
    /// Machine machine type.
    #[serde(rename = "type")]
    pub machine_type: glonax::core::MachineType,
    /// Serial number.
    pub serial: String,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct Config {
    pub mode: OperationMode,
    /// Machine instance.
    pub machine: MachineConfig,
    /// NMEA configuration.
    pub gnss: Option<glonax::service::GnssConfig>,
    /// Host configuration.
    pub host: HostConfig,
    /// Simulation configuration.
    pub simulation: SimulationConfig,
    /// Server configuration.
    pub server: ServerConfig,
    /// J1939 network configuration.
    pub j1939: Vec<J1939NetConfig>,
}

impl Configurable for Config {}
