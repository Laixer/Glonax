use serde_derive::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Status {
    /// The robot is operating normally.
    Healthy = 0xF8,
    /// The robot is operating normally, but some functionality is degraded.
    Degraded = 0xF9,
    /// The robot is not operating normally, but is still functional. However, the robot should
    /// be stopped as soon as possible.
    Faulty = 0xFA,
    /// The robot is in an emergency state and should be stopped immediately.
    Emergency = 0xFB,
}

impl Status {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Healthy => write!(f, "Healthy"),
            Status::Degraded => write!(f, "Degraded"),
            Status::Faulty => write!(f, "Faulty"),
            Status::Emergency => write!(f, "Emergency"),
        }
    }
}

impl TryFrom<&[u8]> for Status {
    type Error = ();

    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        match buffer[0] {
            0xF8 => Ok(Status::Healthy),
            0xF9 => Ok(Status::Degraded),
            0xFA => Ok(Status::Faulty),
            0xFB => Ok(Status::Emergency),
            _ => Err(()),
        }
    }
}

impl TryFrom<Vec<u8>> for Status {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Status::try_from(&value[..])
    }
}

impl crate::transport::Packetize for Status {
    const MESSAGE: crate::transport::frame::FrameMessage =
        crate::transport::frame::FrameMessage::Status;
    const MESSAGE_SIZE: Option<usize> = Some(1);

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
