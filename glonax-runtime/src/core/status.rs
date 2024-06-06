use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModuleState {
    /// The module is operating normally.
    Healthy = 0xF8,
    /// The module is operating normally, but some functionality is degraded.
    Degraded = 0xF9,
    /// The module is operating abnormally, but is still functional.
    Faulty = 0xFA,
    /// The module is in an emergency state and should be stopped immediately.
    Emergency = 0xFB,
}

impl std::fmt::Display for ModuleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleState::Healthy => write!(f, "Healthy"),
            ModuleState::Degraded => write!(f, "Degraded"),
            ModuleState::Faulty => write!(f, "Faulty"),
            ModuleState::Emergency => write!(f, "Emergency"),
        }
    }
}

impl TryFrom<u8> for ModuleState {
    type Error = (); // TODO: Error type

    fn try_from(buffer: u8) -> std::result::Result<Self, Self::Error> {
        match buffer {
            0xF8 => Ok(ModuleState::Healthy),
            0xF9 => Ok(ModuleState::Degraded),
            0xFA => Ok(ModuleState::Faulty),
            0xFB => Ok(ModuleState::Emergency),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModuleError {
    InvalidConfiguration,
    VersionMismatch,
    CommunicationTimeout,
    GenericCommunicationError,
    IOError,
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ModuleError::InvalidConfiguration => "invalid configuration",
                ModuleError::VersionMismatch => "version mismatch",
                ModuleError::CommunicationTimeout => "communication timeout",
                ModuleError::GenericCommunicationError => "generic communication error",
                ModuleError::IOError => "i/o error",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleStatus {
    /// Name of the module.
    pub name: String,
    /// State of the module.
    pub state: ModuleState,
    /// Module error if any.
    pub error: Option<ModuleError>,
}

impl ModuleStatus {
    /// Construct a new module status.
    pub fn new(name: String, state: ModuleState, error: ModuleError) -> Self {
        Self {
            name,
            state,
            error: Some(error),
        }
    }

    /// Construct a new healthy module status.
    pub fn healthy(name: String) -> Self {
        Self {
            name,
            state: ModuleState::Healthy,
            error: None,
        }
    }

    /// Construct a new faulty module status.
    pub fn faulty(name: String, error: ModuleError) -> Self {
        Self {
            name,
            state: ModuleState::Faulty,
            error: Some(error),
        }
    }
}

impl TryFrom<Vec<u8>> for ModuleStatus {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        let name_len = buf.get_u16() as usize;
        let name = buf.split_to(name_len).to_vec();
        let name = String::from_utf8_lossy(&name).to_string();

        let state = ModuleState::try_from(buf.get_u8())?;

        let error = match buf.get_u8() {
            0 => None,
            1 => match buf.get_u8() {
                0 => Some(ModuleError::InvalidConfiguration),
                1 => Some(ModuleError::VersionMismatch),
                2 => Some(ModuleError::CommunicationTimeout),
                3 => Some(ModuleError::GenericCommunicationError),
                4 => Some(ModuleError::IOError),
                _ => return Err(()),
            },
            _ => return Err(()),
        };

        Ok(ModuleStatus { name, state, error })
    }
}

impl crate::protocol::Packetize for ModuleStatus {
    const MESSAGE_TYPE: u8 = 0x16;

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(64);

        let name_bytes = self.name.as_bytes();
        buf.put_u16(name_bytes.len() as u16);
        buf.put(name_bytes);

        buf.put_u8(self.state as u8);

        if let Some(error) = &self.error {
            buf.put_u8(1);
            buf.put_u8(match error {
                ModuleError::InvalidConfiguration => 0,
                ModuleError::VersionMismatch => 1,
                ModuleError::CommunicationTimeout => 2,
                ModuleError::GenericCommunicationError => 3,
                ModuleError::IOError => 4,
            });
        } else {
            buf.put_u8(0);
        }

        buf.to_vec()
    }
}

impl std::fmt::Display for ModuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            match &self.error {
                Some(error) => format!("{}: {}", self.state, error),
                None => format!("{}", self.state),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::Packetize;

    use super::*;

    #[test]
    fn test_module_status_1() {
        let status = ModuleStatus {
            name: "Test".to_string(),
            state: ModuleState::Healthy,
            error: None,
        };

        let bytes = status.to_bytes();
        let status2 = ModuleStatus::try_from(bytes).unwrap();

        assert_eq!(status, status2);
    }

    #[test]
    fn test_module_status_2() {
        let status = ModuleStatus {
            name: "Test".to_string(),
            state: ModuleState::Healthy,
            error: Some(ModuleError::InvalidConfiguration),
        };

        let bytes = status.to_bytes();
        let status2 = ModuleStatus::try_from(bytes).unwrap();

        assert_eq!(status, status2);
    }
}
