use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::util::OnOffExt;

const CONTROL_TYPE_HYDRAULIC_QUICK_DISCONNECT: u8 = 0x5;
const CONTROL_TYPE_HYDRAULIC_LOCK: u8 = 0x6;
const CONTROL_TYPE_HYDRAULIC_BOOST: u8 = 0x7;
const CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX: u8 = 0x8;
const CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX: u8 = 0x9;
const CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT: u8 = 0xA;
const CONTROL_TYPE_HYDRAULIC_RESET: u8 = 0xB;
const CONTROL_TYPE_MACHINE_SHUTDOWN: u8 = 0x1B;
const CONTROL_TYPE_MACHINE_ILLUMINATION: u8 = 0x1C;
const CONTROL_TYPE_MACHINE_LIGHTS: u8 = 0x2D;
const CONTROL_TYPE_MACHINE_HORN: u8 = 0x1E;
const CONTROL_TYPE_MACHINE_STROBE_LIGHT: u8 = 0x1F;
const CONTROL_TYPE_MACHINE_TRAVEL_ALARM: u8 = 0x20;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    /// Hydraulic quick disconnect.
    HydraulicQuickDisconnect(bool),
    /// Hydraulic lock.
    HydraulicLock(bool),
    /// Hydraulic boost.
    HydraulicBoost(bool),
    /// Hydraulic boom conflux.
    HydraulicBoomConflux(bool),
    /// Hydraulic arm conflux.
    HydraulicArmConflux(bool),
    /// Hydraulic boom float.
    HydraulicBoomFloat(bool),
    /// Hydraulic reset.
    HydraulicReset,
    /// Machine shutdown.
    MachineShutdown,
    /// Machine illumination.
    MachineIllumination(bool),
    /// Machine working lights.
    MachineLights(bool),
    /// Machine horn.
    MachineHorn(bool),
    /// Machine strobe light.
    MachineStrobeLight(bool),
    /// Machine travel alarm.
    MachineTravelAlarm(bool),
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::HydraulicQuickDisconnect(on) => {
                write!(f, "Hydraulic quick disconnect: {}", on.as_on_off_str())
            }
            Control::HydraulicLock(on) => write!(f, "Hydraulic lock: {}", on.as_on_off_str()),
            Control::HydraulicBoost(on) => write!(f, "Hydraulic boost: {}", on.as_on_off_str()),
            Control::HydraulicBoomConflux(on) => {
                write!(f, "Hydraulic boom conflux: {}", on.as_on_off_str())
            }
            Control::HydraulicArmConflux(on) => {
                write!(f, "Hydraulic arm conflux: {}", on.as_on_off_str())
            }
            Control::HydraulicBoomFloat(on) => {
                write!(f, "Hydraulic boom float: {}", on.as_on_off_str())
            }
            Control::HydraulicReset => write!(f, "Hydraulic reset"),
            Control::MachineShutdown => write!(f, "Robot shutdown"),
            Control::MachineIllumination(on) => {
                write!(f, "Machine illumination: {}", on.as_on_off_str())
            }
            Control::MachineLights(on) => write!(f, "Machine lights: {}", on.as_on_off_str()),
            Control::MachineHorn(on) => write!(f, "Machine horn: {}", on.as_on_off_str()),
            Control::MachineStrobeLight(on) => {
                write!(f, "Machine strobe light: {}", on.as_on_off_str())
            }
            Control::MachineTravelAlarm(on) => {
                write!(f, "Machine travel alarm: {}", on.as_on_off_str())
            }
        }
    }
}

impl TryFrom<Vec<u8>> for Control {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        let control_type = buf.get_u8();
        let on = buf.get_u8() == 1;

        match control_type {
            CONTROL_TYPE_HYDRAULIC_QUICK_DISCONNECT => Ok(Control::HydraulicQuickDisconnect(on)),
            CONTROL_TYPE_HYDRAULIC_LOCK => Ok(Control::HydraulicLock(on)),
            CONTROL_TYPE_HYDRAULIC_BOOST => Ok(Control::HydraulicBoost(on)),
            CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX => Ok(Control::HydraulicBoomConflux(on)),
            CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX => Ok(Control::HydraulicArmConflux(on)),
            CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT => Ok(Control::HydraulicBoomFloat(on)),
            CONTROL_TYPE_HYDRAULIC_RESET => Ok(Control::HydraulicReset),
            CONTROL_TYPE_MACHINE_SHUTDOWN => Ok(Control::MachineShutdown),
            CONTROL_TYPE_MACHINE_ILLUMINATION => Ok(Control::MachineIllumination(on)),
            CONTROL_TYPE_MACHINE_LIGHTS => Ok(Control::MachineLights(on)),
            CONTROL_TYPE_MACHINE_HORN => Ok(Control::MachineHorn(on)),
            CONTROL_TYPE_MACHINE_STROBE_LIGHT => Ok(Control::MachineStrobeLight(on)),
            CONTROL_TYPE_MACHINE_TRAVEL_ALARM => Ok(Control::MachineTravelAlarm(on)),
            _ => Err(()),
        }
    }
}

impl crate::protocol::Packetize for Control {
    const MESSAGE_TYPE: u8 = 0x45;
    const MESSAGE_SIZE: Option<usize> = Some(2);

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(2);

        match self {
            Control::HydraulicQuickDisconnect(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_QUICK_DISCONNECT);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicLock(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_LOCK);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicBoost(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOST);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicBoomConflux(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicArmConflux(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicBoomFloat(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT);
                buf.put_u8(u8::from(*on));
            }
            Control::HydraulicReset => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_RESET);
                buf.put_u8(1);
            }
            Control::MachineShutdown => {
                buf.put_u8(CONTROL_TYPE_MACHINE_SHUTDOWN);
                buf.put_u8(1);
            }
            Control::MachineIllumination(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_ILLUMINATION);
                buf.put_u8(u8::from(*on));
            }
            Control::MachineLights(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_LIGHTS);
                buf.put_u8(u8::from(*on));
            }
            Control::MachineHorn(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_HORN);
                buf.put_u8(u8::from(*on));
            }
            Control::MachineStrobeLight(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_STROBE_LIGHT);
                buf.put_u8(u8::from(*on));
            }
            Control::MachineTravelAlarm(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_TRAVEL_ALARM);
                buf.put_u8(u8::from(*on));
            }
        }

        buf.to_vec()
    }
}
