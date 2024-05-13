#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct SimulationConfig {
    /// Enable simulation jitter.
    pub jitter: bool,
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
    /// Mode of operation.
    pub mode: OperationMode,
    /// Enable simulation mode.
    #[serde(default)]
    pub is_simulation: bool,
    /// Machine instance.
    pub machine: MachineConfig,
    /// NMEA configuration.
    pub gnss: Option<glonax::service::GnssConfig>,
    /// Simulation configuration.
    pub simulation: Option<SimulationConfig>,
    /// TCP Server configuration.
    pub tcp_server: Option<glonax::service::TcpServerConfig>,
    /// Unix socket configuration.
    pub unix_server: Option<glonax::service::UnixServerConfig>,
    /// J1939 network configuration.
    #[serde(default)]
    pub j1939: Vec<glonax::service::NetworkConfig>,
}
