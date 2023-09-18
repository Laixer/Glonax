/// Configuration trait.
pub trait Configurable: Clone {
    /// Get the global configuration
    fn global(&self) -> &GlobalConfig;
}

// TODO: Move up ?
/// Load structure from TOML file.
pub fn from_toml<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<T> {
    use std::io::Read;

    let mut contents = String::new();
    std::fs::File::open(path)?.read_to_string(&mut contents)?;

    Ok(toml::from_str(&contents).expect("Failed to parse instance configuration"))
}

/// Glonax global configuration.
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    /// Name of the binary.
    pub bin_name: String,
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
            daemon: false,
        }
    }
}
