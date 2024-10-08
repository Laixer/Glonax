use bytes::{BufMut, BytesMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineState {
    /// Engine is shut down, ready to start.
    NoRequest = 0x00,
    /// Engine is starting up.
    Starting = 0x01,
    /// Engine is shutting down.
    Stopping = 0x02,
    /// Engine is running.
    Request = 0x10,
}

impl TryFrom<u8> for EngineState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(EngineState::NoRequest),
            0x01 => Ok(EngineState::Starting),
            0x02 => Ok(EngineState::Stopping),
            0x10 => Ok(EngineState::Request),
            _ => Err(()),
        }
    }
}

/// Represents the state of an engine.
///
/// This struct holds information about the state of an engine, including the driver demand, actual engine,
/// revolutions per minute (RPM), and state.
///
/// # Fields
///
/// * `driver_demand` - The engine driver demand in percent.
/// * `actual_engine` - The engine actual engine in percent.
/// * `rpm` - The engine RPM.
/// * `state` - The engine state.
///
/// # Examples
///
/// ```rust
/// use glonax::core::Engine;
///
/// let engine = Engine::from_rpm(1000);
/// assert_eq!(engine.rpm, 1000);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engine {
    /// Engine Driver Demand in percent.
    pub driver_demand: u8,
    /// Engine Actual Engine in percent.
    pub actual_engine: u8,
    /// Engine RPM.
    pub rpm: u16,
    /// Engine state.
    pub state: EngineState,
}

impl Engine {
    /// Create a new engine with the given RPM.
    ///
    /// # Arguments
    ///
    /// * `rpm` - The revolutions per minute of the engine.
    ///
    /// # Returns
    ///
    /// A new `Engine` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glonax::core::Engine;
    ///
    /// let engine = Engine::from_rpm(1000);
    /// assert_eq!(engine.rpm, 1000);
    /// ```
    pub fn from_rpm(rpm: u16) -> Self {
        Self {
            rpm,
            state: EngineState::Request,
            ..Default::default()
        }
    }

    /// Create a new engine with the given state.
    ///
    /// # Returns
    ///
    /// A new `Engine` instance with the state set to `EngineState::NoRequest`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glonax::core::{Engine, EngineState};
    ///
    /// let engine = Engine::shutdown();
    /// assert_eq!(engine.state, EngineState::NoRequest);
    /// ```
    pub fn shutdown() -> Self {
        Self {
            state: EngineState::NoRequest,
            ..Default::default()
        }
    }

    /// Check if the engine is running.
    ///
    /// # Returns
    ///
    /// `true` if the engine is running, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glonax::core::Engine;
    ///
    /// let engine = Engine::from_rpm(1000);
    /// assert!(engine.is_running());
    /// ```
    #[inline]
    pub fn is_running(&self) -> bool {
        self.state == EngineState::Request && (self.actual_engine > 0 || self.rpm > 0)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            driver_demand: Default::default(),
            actual_engine: Default::default(),
            rpm: Default::default(),
            state: EngineState::NoRequest,
        }
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Driver Demand: {}% Actual Engine: {}% RPM: {} State: {:?}",
            self.driver_demand, self.actual_engine, self.rpm, self.state
        )
    }
}

impl TryFrom<Vec<u8>> for Engine {
    type Error = ();

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let driver_demand = buffer[0];
        let actual_engine = buffer[1];
        let rpm = u16::from_be_bytes([buffer[2], buffer[3]]);

        let state = EngineState::try_from(buffer[4])?;

        Ok(Self {
            driver_demand,
            actual_engine,
            rpm,
            state,
        })
    }
}

impl crate::protocol::Packetize for Engine {
    const MESSAGE_TYPE: u8 = 0x43;
    const MESSAGE_SIZE: Option<usize> = Some(5);

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::MESSAGE_SIZE.unwrap());

        buf.put_u8(self.driver_demand);
        buf.put_u8(self.actual_engine);
        buf.put_u16(self.rpm);
        buf.put_u8(self.state as u8);

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Packetize;

    #[test]
    fn test_engine_state() {
        assert_eq!(EngineState::try_from(0x00).unwrap(), EngineState::NoRequest);
        assert_eq!(EngineState::try_from(0x01).unwrap(), EngineState::Starting);
        assert_eq!(EngineState::try_from(0x02).unwrap(), EngineState::Stopping);
        assert_eq!(EngineState::try_from(0x10).unwrap(), EngineState::Request);
        assert!(EngineState::try_from(0x03).is_err());
    }

    #[test]
    fn test_engine() {
        let engine = Engine {
            driver_demand: 0x01,
            actual_engine: 0x02,
            rpm: 0x03,
            state: EngineState::Request,
        };

        let bytes = engine.to_bytes();

        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x02);
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes[3], 0x03);
        assert_eq!(bytes[4], 0x10);

        let engine = Engine::try_from(bytes).unwrap();

        assert_eq!(engine.driver_demand, 0x01);
        assert_eq!(engine.actual_engine, 0x02);
        assert_eq!(engine.rpm, 0x03);
        assert_eq!(engine.state, EngineState::Request);
    }
}
