use bytes::{Buf, BufMut, Bytes, BytesMut};

const CONTROL_TYPE_ENGINE_REQUEST: u8 = 0x01;
const CONTROL_TYPE_ENGINE_SHUTDOWN: u8 = 0x02;
const CONTROL_TYPE_MACHINE_SHUTDOWN: u8 = 0x1B;

#[derive(Clone, Copy)]
pub enum Control {
    /// Engine RPM request.
    EngineRequest(u16),
    /// Engine shutdown.
    EngineShutdown,
    /// Robot shutdown.
    RobotShutdown,
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::EngineRequest(rpm) => write!(f, "Engine request: {}", rpm),
            Control::EngineShutdown => write!(f, "Engine shutdown"),
            Control::RobotShutdown => write!(f, "Robot shutdown"),
        }
    }
}

impl TryFrom<Vec<u8>> for Control {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        match buf.get_u8() {
            CONTROL_TYPE_ENGINE_REQUEST => Ok(Control::EngineRequest(buf.get_u16())),
            CONTROL_TYPE_ENGINE_SHUTDOWN => Ok(Control::EngineShutdown),
            CONTROL_TYPE_MACHINE_SHUTDOWN => Ok(Control::RobotShutdown),
            _ => Err(()),
        }
    }
}

impl crate::protocol::Packetize for Control {
    const MESSAGE_TYPE: u8 = 0x45;

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(2);

        match self {
            Control::EngineRequest(rpm) => {
                buf.put_u8(CONTROL_TYPE_ENGINE_REQUEST);
                buf.put_u16(*rpm);
            }
            Control::EngineShutdown => {
                buf.put_u8(CONTROL_TYPE_ENGINE_SHUTDOWN);
            }
            Control::RobotShutdown => {
                buf.put_u8(CONTROL_TYPE_MACHINE_SHUTDOWN);
            }
        }

        buf.to_vec()
    }
}
