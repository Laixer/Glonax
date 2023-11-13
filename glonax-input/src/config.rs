use glonax::{Configurable, GlobalConfig};

#[derive(Clone, Debug)]
pub(crate) struct InputConfig {
    /// Remote network address.
    pub address: std::net::SocketAddr,
    /// Input device.
    pub device: String,
    /// Configure failsafe mode.
    pub fail_safe: bool,
    /// Input commands will translate to the full motion range.
    pub full_motion: bool,
    /// Global configuration.
    pub global: GlobalConfig,
}

impl Configurable for InputConfig {
    fn global(&self) -> &GlobalConfig {
        &self.global
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            address: std::net::SocketAddr::from((
                std::net::Ipv4Addr::LOCALHOST,
                glonax::consts::DEFAULT_NETWORK_PORT,
            )),
            device: "/dev/input/js0".to_string(),
            fail_safe: false,
            full_motion: false,
            global: GlobalConfig::default(),
        }
    }
}
