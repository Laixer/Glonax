use bytes::{BufMut, BytesMut};

#[derive(Default)]
pub struct Engine {
    /// Engine Driver Demand in percent.
    pub driver_demand: u8,
    /// Engine Actual Engine in percent.
    pub actual_engine: u8,
    /// Engine RPM.
    pub rpm: u16,
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

        Ok(Self {
            driver_demand,
            actual_engine,
            rpm,
        })
    }
}

impl crate::protocol::Packetize for Engine {
    const MESSAGE_TYPE: u8 = 0x43;
    const MESSAGE_SIZE: Option<usize> = Some(4);

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::MESSAGE_SIZE.unwrap());

        buf.put_u8(self.driver_demand);
        buf.put_u8(self.actual_engine);
        buf.put_u16(self.rpm);

        buf.to_vec()
    }
}
