#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct SimulationConfig {
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
    /// Mode of operation.
    pub mode: OperationMode,
    /// Enable simulation mode.
    #[serde(default)]
    pub is_simulation: bool,
    /// Machine instance.
    pub machine: MachineConfig,
    /// NMEA configuration.
    pub gnss: Option<glonax::service::GnssConfig>,
    /// Host configuration.
    pub host: glonax::service::HostConfig,
    /// Simulation configuration.
    pub simulation: Option<SimulationConfig>,
    /// TCP Server configuration.
    pub tcp_server: Option<glonax::service::TcpServerConfig>,
    /// J1939 network configuration.
    pub j1939: Vec<J1939NetConfig>,
}
