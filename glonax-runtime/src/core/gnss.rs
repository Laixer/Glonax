use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GnssStatus {
    /// GNSS is disabled.
    Disabled = 0xFF,
    /// GNSS device not found.
    DeviceNotFound = 0x00,
    /// GNSS has a location fix.
    LocationFix = 0x01,
}

impl TryFrom<u8> for GnssStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFF => Ok(GnssStatus::Disabled),
            0x00 => Ok(GnssStatus::DeviceNotFound),
            0x01 => Ok(GnssStatus::LocationFix),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gnss {
    /// GNSS Latitude and Longitude.
    pub location: (f32, f32),
    /// GNSS Altitude in meters.
    pub altitude: f32,
    /// GNSS Speed in meters per second.
    pub speed: f32,
    /// GNSS Heading in degrees.
    pub heading: f32,
    /// GNSS Satellites.
    pub satellites: u8,
    /// GNSS Status.
    pub status: GnssStatus,
}

impl Default for Gnss {
    fn default() -> Self {
        Self {
            location: (0.0, 0.0),
            altitude: 0.0,
            speed: 0.0,
            heading: 0.0,
            satellites: 0,
            status: GnssStatus::Disabled,
        }
    }
}

impl std::fmt::Display for Gnss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str(&format!(
            "Location: ({:.5}, {:.5}); ",
            self.location.0, self.location.1
        ));
        s.push_str(&format!("Altitude: {:.1}m; ", self.altitude));
        s.push_str(&format!("Speed: {:.1}m/s; ", self.speed));
        s.push_str(&format!("Heading: {:.1}°; ", self.heading));
        s.push_str(&format!("Satellites: {}", self.satellites));

        write!(f, "{}", s)
    }
}

impl TryFrom<Vec<u8>> for Gnss {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        Ok(Self {
            location: (buf.get_f32(), buf.get_f32()),
            altitude: buf.get_f32(),
            speed: buf.get_f32(),
            heading: buf.get_f32(),
            satellites: buf.get_u8(),
            status: GnssStatus::try_from(buf.get_u8())?,
        })
    }
}

impl crate::protocol::Packetize for Gnss {
    const MESSAGE_TYPE: u8 = 0x42;
    const MESSAGE_SIZE: Option<usize> = Some((std::mem::size_of::<f32>() * 5) + 1 + 1);

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::MESSAGE_SIZE.unwrap());

        buf.put_f32(self.location.0);
        buf.put_f32(self.location.1);

        buf.put_f32(self.altitude);
        buf.put_f32(self.speed);
        buf.put_f32(self.heading);

        buf.put_u8(self.satellites);

        buf.put_u8(self.status as u8);

        buf.to_vec()
    }
}
