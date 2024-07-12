#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct Config {
    /// Unix socket listener configuration.
    #[serde(default)]
    pub unix_listener: glonax::service::UnixServerConfig,
}
