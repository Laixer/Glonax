#[derive(Clone, Copy)]
pub enum Control {
    /// Engine start.
    EngineStart = 3,
    /// Engine stop.
    EngineStop = 7,
    /// Robot shutdown.
    RobotShutdown = 27,
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::EngineStart => write!(f, "Engine start"),
            Control::EngineStop => write!(f, "Engine stop"),
            Control::RobotShutdown => write!(f, "Robot shutdown"),
        }
    }
}

impl TryFrom<Vec<u8>> for Control {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value[0] {
            3 => Ok(Control::EngineStart),
            7 => Ok(Control::EngineStop),
            27 => Ok(Control::RobotShutdown),
            _ => Err(()),
        }
    }
}

impl crate::protocol::Packetize for Control {
    const MESSAGE_TYPE: u8 = 0x45;
    const MESSAGE_SIZE: Option<usize> = Some(1);

    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}
