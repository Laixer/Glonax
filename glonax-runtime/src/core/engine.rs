use bytes::{BufMut, BytesMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineStatus {
    /// Engine is disabled.
    Disabled = 0xFF,
    /// Controller Area Network is down.
    NetworkDown = 0x00,
    /// Engine message timeout.
    MessageTimeout = 0x01,
    /// Engine is nominal.
    Nominal = 0x02,
}

impl TryFrom<u8> for EngineStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFF => Ok(EngineStatus::Disabled),
            0x00 => Ok(EngineStatus::NetworkDown),
            0x01 => Ok(EngineStatus::MessageTimeout),
            0x02 => Ok(EngineStatus::Nominal),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engine {
    /// Engine Driver Demand in percent.
    pub driver_demand: u8,
    /// Engine Actual Engine in percent.
    pub actual_engine: u8,
    /// Engine RPM.
    pub rpm: u16,
    /// Engine status.
    pub status: EngineStatus,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            driver_demand: Default::default(),
            actual_engine: Default::default(),
            rpm: Default::default(),
            status: EngineStatus::Disabled,
        }
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str(&format!("Driver Demand: {}%; ", self.driver_demand));
        s.push_str(&format!("Actual Engine: {}%; ", self.actual_engine));
        s.push_str(&format!("RPM: {}; ", self.rpm));

        write!(f, "{}", s)
    }
}

impl TryFrom<Vec<u8>> for Engine {
    type Error = ();

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let driver_demand = buffer[0];
        let actual_engine = buffer[1];
        let rpm = u16::from_be_bytes([buffer[2], buffer[3]]);

        let status = EngineStatus::try_from(buffer[4])?;

        Ok(Self {
            driver_demand,
            actual_engine,
            rpm,
            status,
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

        buf.put_u8(self.status as u8);

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Packetize;

    #[test]
    fn test_engine_status() {
        assert_eq!(
            EngineStatus::try_from(0xFF).unwrap(),
            EngineStatus::Disabled
        );
        assert_eq!(
            EngineStatus::try_from(0x00).unwrap(),
            EngineStatus::NetworkDown
        );
        assert_eq!(
            EngineStatus::try_from(0x01).unwrap(),
            EngineStatus::MessageTimeout
        );
        assert_eq!(EngineStatus::try_from(0x02).unwrap(), EngineStatus::Nominal);
        assert!(EngineStatus::try_from(0x03).is_err());
    }

    #[test]
    fn test_engine() {
        let engine = Engine {
            driver_demand: 0x01,
            actual_engine: 0x02,
            rpm: 0x03,
            status: EngineStatus::Nominal,
        };

        let bytes = engine.to_bytes();

        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x02);
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes[3], 0x03);
        assert_eq!(bytes[4], 0x02);

        let engine = Engine::try_from(bytes).unwrap();

        assert_eq!(engine.driver_demand, 0x01);
        assert_eq!(engine.actual_engine, 0x02);
        assert_eq!(engine.rpm, 0x03);
        assert_eq!(engine.status, EngineStatus::Nominal);
    }
}
