/// Configuration trait.
pub trait Configurable: Clone {
    /// Get the global configuration
    fn global(&self) -> &GlobalConfig;
}

pub struct Error {
    /// Error kind.
    kind: ErrorKind,
    /// Error message.
    message: String,
}

pub enum ErrorKind {
    /// Error while parsing configuration.
    ParseError(toml::de::Error),
    /// Error while loading configuration file.
    IoError(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::ParseError(e) => write!(f, "{}: Parse error: {:?}", self.message, e),
            ErrorKind::IoError(e) => write!(f, "{}: IO error: {:?}", self.message, e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::ParseError(e) => Some(e),
            ErrorKind::IoError(e) => Some(e),
        }
    }
}

/// Load configuration structure from TOML file.
pub fn from_file<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> std::result::Result<T, Error> {
    use std::io::Read;

    let mut contents = String::new();
    let mut file = std::fs::File::open(&path).map_err(|e| Error {
        kind: ErrorKind::IoError(e),
        message: format!(
            "Failed to open configuration file: {}",
            path.as_ref().display()
        ),
    })?;
    file.read_to_string(&mut contents).map_err(|e| Error {
        kind: ErrorKind::IoError(e),
        message: format!(
            "Failed to read configuration file: {}",
            path.as_ref().display()
        ),
    })?;

    toml::from_str(&contents).map_err(|e| Error {
        kind: ErrorKind::ParseError(e),
        message: format!(
            "Failed to parse configuration file: {}",
            path.as_ref().display()
        ),
    })
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
