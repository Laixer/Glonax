use bytes::{Buf, BufMut, Bytes, BytesMut};

const CONTROL_TYPE_ENGINE_REQUEST: u8 = 0x01;
const CONTROL_TYPE_ENGINE_SHUTDOWN: u8 = 0x02;
const CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT: u8 = 0x5;
const CONTROL_TYPE_HYDRAULIC_LOCK: u8 = 0x6;
const CONTROL_TYPE_MACHINE_SHUTDOWN: u8 = 0x1B;
const CONTROL_TYPE_MACHINE_ILLUMINATION: u8 = 0x1C;
const CONTROL_TYPE_MACHINE_LIGHTS: u8 = 0x2D;
const CONTROL_TYPE_MACHINE_HORN: u8 = 0x1E;

#[derive(Clone, Copy)]
pub enum Control {
    /// Engine RPM request.
    EngineRequest(u16),
    /// Engine shutdown.
    EngineShutdown,
    /// Hydraulic quick disconnect.
    HydraulicQuickDisconnect(bool),
    /// Hydraulic lock.
    HydraulicLock(bool),
    /// Machine shutdown.
    MachineShutdown,
    /// Machine illumination.
    MachineIllumination(bool),
    /// Machine working lights.
    MachineLights(bool),
    /// Machine horn.
    MachineHorn(bool),
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::EngineRequest(rpm) => write!(f, "Engine request: {}", rpm),
            Control::EngineShutdown => write!(f, "Engine shutdown"),
            Control::HydraulicQuickDisconnect(on) => {
                write!(f, "Hydraulic quick disconnect: {}", on)
            }
            Control::HydraulicLock(on) => write!(f, "Hydraulic lock: {}", on),
            Control::MachineShutdown => write!(f, "Robot shutdown"),
            Control::MachineIllumination(on) => write!(f, "Machine illumination: {}", on),
            Control::MachineLights(on) => write!(f, "Machine lights: {}", on),
            Control::MachineHorn(on) => write!(f, "Machine horn: {}", on),
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
            CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT => {
                Ok(Control::HydraulicQuickDisconnect(buf.get_u8() != 0))
            }
            CONTROL_TYPE_HYDRAULIC_LOCK => Ok(Control::HydraulicLock(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_SHUTDOWN => Ok(Control::MachineShutdown),
            CONTROL_TYPE_MACHINE_ILLUMINATION => {
                Ok(Control::MachineIllumination(buf.get_u8() != 0))
            }
            CONTROL_TYPE_MACHINE_LIGHTS => Ok(Control::MachineLights(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_HORN => Ok(Control::MachineHorn(buf.get_u8() != 0)),
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
            Control::HydraulicQuickDisconnect(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicLock(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_LOCK);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::MachineShutdown => {
                buf.put_u8(CONTROL_TYPE_MACHINE_SHUTDOWN);
            }
            Control::MachineIllumination(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_ILLUMINATION);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::MachineLights(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_LIGHTS);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::MachineHorn(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_HORN);
                buf.put_u8(if *on { 1 } else { 0 });
            }
        }

        buf.to_vec()
    }
}
