use serde_derive::Deserialize;

pub trait Configurable: Clone {
    /// Get the global configuration
    fn global(&self) -> &GlobalConfig;
}

// TODO: Move to a more appropriate place
// / Locate the file_name in the file system.
// /
// / NOTE: This function is UNIX specific.
// pub fn locate_file(file_name: &str) -> Option<String> {
//     let path = std::path::Path::new(file_name);
//     if path.exists() {
//         return Some(path.to_str().unwrap().to_string());
//     }

//     let path = std::path::Path::new("/etc").join(file_name);
//     if path.exists() {
//         return Some(path.to_str().unwrap().to_string());
//     }

//     let path = std::path::Path::new("/usr/local/share/glonax").join(file_name);
//     if path.exists() {
//         return Some(path.to_str().unwrap().to_string());
//     }

//     let path = std::path::Path::new("/var/lib/glonax").join(file_name);
//     if path.exists() {
//         return Some(path.to_str().unwrap().to_string());
//     }

//     None
// }

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    /// Telemetry host.
    pub host: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InstanceConfig {
    /// Instance unique identifier.
    pub instance: String,
    /// Instance name.
    pub name: Option<String>,
    /// Instance model.
    pub model: String,
    /// Telemetry configuration.
    pub telemetry: Option<TelemetryConfig>,
}

pub fn instance_config(path: impl AsRef<std::path::Path>) -> std::io::Result<InstanceConfig> {
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
