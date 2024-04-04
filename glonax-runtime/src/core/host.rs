use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStatus {
    /// Host is disabled.
    Disabled = 0xFF,
    /// Host memory is low.
    MemoryLow = 0x00,
    /// Host CPU is high.
    CPUHigh = 0x01,
    /// Engine is nominal.
    Nominal = 0x02,
}

impl TryFrom<u8> for HostStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFF => Ok(HostStatus::Disabled),
            0x00 => Ok(HostStatus::MemoryLow),
            0x01 => Ok(HostStatus::CPUHigh),
            0x02 => Ok(HostStatus::Nominal),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Host {
    /// VMS Memory total and used in bytes.
    pub memory: (u64, u64),
    /// VMS Swap total and used in bytes.
    pub swap: (u64, u64),
    /// VMS CPU load.
    pub cpu_load: (f64, f64, f64),
    /// VMS Uptime in seconds.
    pub uptime: u64,
    /// VMS Timestamp in seconds.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Host status.
    pub status: HostStatus,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            memory: (0, 0),
            swap: (0, 0),
            cpu_load: (0.0, 0.0, 0.0),
            uptime: 0,
            timestamp: chrono::Utc::now(),
            status: HostStatus::Disabled,
        }
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let seconds = self.uptime % 60;
        let minutes = (self.uptime / 60) % 60;
        let hours = (self.uptime / 60) / 60;

        s.push_str(&format!(
            "Uptime: {:02}:{:02}:{:02}; ",
            hours, minutes, seconds
        ));
        s.push_str(&format!(
            "Memory usage: {:.2}GB / {:.2}GB; ",
            self.memory.0 as f64 / 1024.0 / 1024.0 / 1024.0,
            self.memory.1 as f64 / 1024.0 / 1024.0 / 1024.0
        ));
        s.push_str(&format!(
            "Swap usage: {:.2}GB / {:.2}GB; ",
            self.swap.0 as f64 / 1024.0 / 1024.0 / 1024.0,
            self.swap.1 as f64 / 1024.0 / 1024.0 / 1024.0
        ));
        s.push_str(&format!(
            "CPU load: {:.1}%, {:.1}%, {:.1}%; ",
            self.cpu_load.0, self.cpu_load.1, self.cpu_load.2
        ));
        s.push_str(&format!("Timestamp: {}", self.timestamp));

        write!(f, "{}", s)
    }
}

impl TryFrom<Vec<u8>> for Host {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        use chrono::{TimeZone, Utc};

        let mut buf = Bytes::copy_from_slice(&value);

        Ok(Self {
            memory: (buf.get_u64(), buf.get_u64()),
            swap: (buf.get_u64(), buf.get_u64()),
            cpu_load: (buf.get_f64(), buf.get_f64(), buf.get_f64()),
            uptime: buf.get_u64(),
            timestamp: Utc.timestamp_opt(buf.get_i64(), 0).unwrap(),
            status: HostStatus::try_from(buf.get_u8())?,
        })
    }
}

impl crate::protocol::Packetize for Host {
    const MESSAGE_TYPE: u8 = 0x41;
    const MESSAGE_SIZE: Option<usize> = Some((std::mem::size_of::<u64>() * 9) + 1);

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::MESSAGE_SIZE.unwrap());

        buf.put_u64(self.memory.0);
        buf.put_u64(self.memory.1);

        buf.put_u64(self.swap.0);
        buf.put_u64(self.swap.1);

        buf.put_f64(self.cpu_load.0);
        buf.put_f64(self.cpu_load.1);
        buf.put_f64(self.cpu_load.2);

        buf.put_u64(self.uptime);

        buf.put_i64(self.timestamp.timestamp());

        buf.put_u8(self.status as u8);

        buf.to_vec()
    }
}
