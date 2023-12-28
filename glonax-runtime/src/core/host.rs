use bytes::{Buf, BufMut, Bytes, BytesMut};

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
}

impl Default for Host {
    fn default() -> Self {
        Self {
            memory: (0, 0),
            swap: (0, 0),
            cpu_load: (0.0, 0.0, 0.0),
            uptime: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str(&format!("Uptime: {} seconds; ", self.uptime));
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
        })
    }
}

impl crate::protocol::Packetize for Host {
    const MESSAGE: crate::protocol::frame::FrameMessage =
        crate::protocol::frame::FrameMessage::VMS;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<u64>() * 9);

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

        buf.to_vec()
    }
}
