use bytes::{Buf, BufMut, Bytes, BytesMut};

const CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT: u8 = 0x5;
const CONTROL_TYPE_HYDRAULIC_LOCK: u8 = 0x6;
const CONTROL_TYPE_HYDRAULIC_BOOST: u8 = 0x7;
const CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX: u8 = 0x8;
const CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX: u8 = 0x9;
const CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT: u8 = 0xA;
const CONTROL_TYPE_MACHINE_SHUTDOWN: u8 = 0x1B;
const CONTROL_TYPE_MACHINE_ILLUMINATION: u8 = 0x1C;
const CONTROL_TYPE_MACHINE_LIGHTS: u8 = 0x2D;
const CONTROL_TYPE_MACHINE_HORN: u8 = 0x1E;
const CONTROL_TYPE_MACHINE_STROBE_LIGHT: u8 = 0x1F;
const CONTROL_TYPE_MACHINE_TRAVEL_ALARM: u8 = 0x20;

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
        // TODO: Use a macro to avoid repeating this code.
        // TOOD: Move
        fn bool_to_on_off(b: bool) -> &'static str {
            if b {
                "on"
            } else {
                "off"
            }
        }

        match self {
            Control::HydraulicQuickDisconnect(on) => {
                write!(f, "Hydraulic quick disconnect: {}", bool_to_on_off(*on))
            }
            Control::HydraulicLock(on) => write!(f, "Hydraulic lock: {}", bool_to_on_off(*on)),
            Control::HydraulicBoost(on) => write!(f, "Hydraulic boost: {}", bool_to_on_off(*on)),
            Control::HydraulicBoomConflux(on) => {
                write!(f, "Hydraulic boom conflux: {}", bool_to_on_off(*on))
            }
            Control::HydraulicArmConflux(on) => {
                write!(f, "Hydraulic arm conflux: {}", bool_to_on_off(*on))
            }
            Control::HydraulicBoomFloat(on) => {
                write!(f, "Hydraulic boom float: {}", bool_to_on_off(*on))
            }
            Control::MachineShutdown => write!(f, "Robot shutdown"),
            Control::MachineIllumination(on) => {
                write!(f, "Machine illumination: {}", bool_to_on_off(*on))
            }
            Control::MachineLights(on) => write!(f, "Machine lights: {}", bool_to_on_off(*on)),
            Control::MachineHorn(on) => write!(f, "Machine horn: {}", bool_to_on_off(*on)),
            Control::MachineStrobeLight(on) => {
                write!(f, "Machine strobe light: {}", bool_to_on_off(*on))
            }
            Control::MachineTravelAlarm(on) => {
                write!(f, "Machine travel alarm: {}", bool_to_on_off(*on))
            }
        }
    }
}

impl TryFrom<Vec<u8>> for Control {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        match buf.get_u8() {
            CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT => {
                Ok(Control::HydraulicQuickDisconnect(buf.get_u8() != 0))
            }
            CONTROL_TYPE_HYDRAULIC_LOCK => Ok(Control::HydraulicLock(buf.get_u8() != 0)),
            CONTROL_TYPE_HYDRAULIC_BOOST => Ok(Control::HydraulicBoost(buf.get_u8() != 0)),
            CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX => {
                Ok(Control::HydraulicBoomConflux(buf.get_u8() != 0))
            }
            CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX => {
                Ok(Control::HydraulicArmConflux(buf.get_u8() != 0))
            }
            CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT => Ok(Control::HydraulicBoomFloat(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_SHUTDOWN => Ok(Control::MachineShutdown),
            CONTROL_TYPE_MACHINE_ILLUMINATION => {
                Ok(Control::MachineIllumination(buf.get_u8() != 0))
            }
            CONTROL_TYPE_MACHINE_LIGHTS => Ok(Control::MachineLights(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_HORN => Ok(Control::MachineHorn(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_STROBE_LIGHT => Ok(Control::MachineStrobeLight(buf.get_u8() != 0)),
            CONTROL_TYPE_MACHINE_TRAVEL_ALARM => Ok(Control::MachineTravelAlarm(buf.get_u8() != 0)),
            _ => Err(()),
        }
    }
}

impl crate::protocol::Packetize for Control {
    const MESSAGE_TYPE: u8 = 0x45;

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(2);

        match self {
            Control::HydraulicQuickDisconnect(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_QUICK_DISONNECT);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicLock(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_LOCK);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicBoost(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOST);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicBoomConflux(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOM_CONFLUX);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicArmConflux(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_ARM_CONFLUX);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::HydraulicBoomFloat(on) => {
                buf.put_u8(CONTROL_TYPE_HYDRAULIC_BOOM_FLOAT);
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
            Control::MachineStrobeLight(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_STROBE_LIGHT);
                buf.put_u8(if *on { 1 } else { 0 });
            }
            Control::MachineTravelAlarm(on) => {
                buf.put_u8(CONTROL_TYPE_MACHINE_TRAVEL_ALARM);
                buf.put_u8(if *on { 1 } else { 0 });
            }
        }

        buf.to_vec()
    }
}
