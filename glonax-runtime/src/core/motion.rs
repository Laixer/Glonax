use bytes::{Buf, BufMut, Bytes, BytesMut};

const PROTO_TYPE_STOP_ALL: u8 = 0x00;
const PROTO_TYPE_RESUME_ALL: u8 = 0x01;
const PROTO_TYPE_STRAIGHT_DRIVE: u8 = 0x05;
const PROTO_TYPE_CHANGE: u8 = 0x10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Actuator {
    /// Boom actuator.
    Boom = 0,
    /// Arm actuator.
    Arm = 4,
    /// Attachment actuator.
    Attachment = 5,
    /// Slew actuator.
    Slew = 1,
    /// Left limp actuator.
    LimpLeft = 3,
    /// Right limp actuator.
    LimpRight = 2,
}

type MotionValueType = i16;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ChangeSet {
    /// Actuator ID.
    pub actuator: Actuator,
    /// Actuator value.
    pub value: MotionValueType,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub enum Motion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Drive straight forward or backwards.
    StraightDrive(MotionValueType),
    /// Change motion on actuators.
    Change(Vec<ChangeSet>),
}

impl Motion {
    /// Maximum power setting.
    pub const POWER_MAX: MotionValueType = MotionValueType::MAX;
    /// Neutral power setting.
    pub const POWER_NEUTRAL: MotionValueType = 0;
    /// Minimum power setting.
    pub const POWER_MIN: MotionValueType = MotionValueType::MIN;

    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(32);

        match self {
            Motion::StopAll => {
                buf.put_u8(PROTO_TYPE_STOP_ALL);
            }
            Motion::ResumeAll => {
                buf.put_u8(PROTO_TYPE_RESUME_ALL);
            }
            Motion::StraightDrive(value) => {
                buf.put_u8(PROTO_TYPE_STRAIGHT_DRIVE);
                buf.put_i16(*value);
            }
            Motion::Change(changes) => {
                buf.put_u8(PROTO_TYPE_CHANGE);
                buf.put_u8(changes.len() as u8);
                for change in changes {
                    buf.put_u16(change.actuator as u16);
                    buf.put_i16(change.value);
                }
            }
        }

        buf.to_vec()
    }
}

impl std::fmt::Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Motion::StopAll => write!(f, "Stop all"),
            Motion::ResumeAll => write!(f, "Resume all"),
            Motion::StraightDrive(value) => write!(f, "Straight drive: {}", value),
            Motion::Change(changes) => {
                write!(
                    f,
                    "Change: {}",
                    changes
                        .iter()
                        .map(|changeset| format!(
                            "Actuator: {:?}; Value: {}, ",
                            changeset.actuator, changeset.value
                        ))
                        .collect::<String>()
                )
            }
        }
    }
}

impl TryFrom<&[u8]> for Motion {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(value);

        match buf.get_u8() {
            PROTO_TYPE_STOP_ALL => Ok(Motion::StopAll),
            PROTO_TYPE_RESUME_ALL => Ok(Motion::ResumeAll),
            PROTO_TYPE_STRAIGHT_DRIVE => Ok(Motion::StraightDrive(buf.get_i16())),
            PROTO_TYPE_CHANGE => {
                let count = buf.get_u8();
                let mut changes = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    changes.push(ChangeSet {
                        actuator: unsafe { std::mem::transmute(buf.get_u16() as u32) },
                        value: buf.get_i16(),
                    });
                }
                Ok(Motion::Change(changes))
            }
            _ => Err(()),
        }
    }
}
