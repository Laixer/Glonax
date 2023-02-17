pub trait Configurable: Clone {
    fn global(&self) -> &GlobalConfig;
}

/// Glonax global configuration.
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    /// Name of the binary.
    pub bin_name: String,

    /// MQTT broker hostname or ip address.
    pub mqtt_host: String,

    /// MQTT broker port.
    pub mqtt_port: u16,

    /// MQTT broker username.
    pub mqtt_username: Option<String>,

    /// MQTT broker username.
    pub mqtt_password: Option<String>,

    /// Whether motion is enabled.
    pub enable_motion: bool,

    /// Whether motion is slowed down.
    pub slow_motion: bool,

    /// Whether the application runs as daemon.
    pub daemon: bool,
}

impl Configurable for GlobalConfig {
    fn global(&self) -> &GlobalConfig {
        self
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            bin_name: String::new(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_username: None,
            mqtt_password: None,
            enable_motion: true,
            slow_motion: false,
            daemon: false,
        }
    }
}
