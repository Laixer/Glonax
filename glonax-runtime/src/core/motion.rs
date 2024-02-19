use bytes::{Buf, BufMut, Bytes, BytesMut};

const MOTION_TYPE_STOP_ALL: u8 = 0x00;
const MOTION_TYPE_RESUME_ALL: u8 = 0x01;
const MOTION_TYPE_RESET_ALL: u8 = 0x02;
const MOTION_TYPE_STRAIGHT_DRIVE: u8 = 0x05;
const MOTION_TYPE_CHANGE: u8 = 0x10;

// FUTURE: Move to glonax-server or an excatavator module
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl TryFrom<u16> for Actuator {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Actuator::Boom),
            4 => Ok(Actuator::Arm),
            5 => Ok(Actuator::Attachment),
            1 => Ok(Actuator::Slew),
            3 => Ok(Actuator::LimpLeft),
            2 => Ok(Actuator::LimpRight),
            _ => Err(()),
        }
    }
}

type MotionValueType = i16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeSet {
    /// Actuator ID.
    pub actuator: Actuator,
    /// Actuator value.
    pub value: MotionValueType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Reset the motion state machine.
    ResetAll,
    /// Drive straight forward or backwards.
    StraightDrive(MotionValueType),
    /// Change motion on actuators.
    Change(Vec<ChangeSet>),
}

impl Default for Motion {
    fn default() -> Self {
        Self::StopAll
    }
}

impl Motion {
    /// Maximum power setting.
    pub const POWER_MAX: MotionValueType = MotionValueType::MAX;
    /// Neutral power setting.
    pub const POWER_NEUTRAL: MotionValueType = 0;
    /// Minimum power setting.
    pub const POWER_MIN: MotionValueType = MotionValueType::MIN;

    /// Create a new motion command.
    pub fn new<T: Into<MotionValueType>>(actuator: Actuator, value: T) -> Self {
        Self::Change(vec![ChangeSet {
            actuator,
            value: value.into(),
        }])
    }
}

impl FromIterator<(Actuator, MotionValueType)> for Motion {
    fn from_iter<T: IntoIterator<Item = (Actuator, MotionValueType)>>(iter: T) -> Self {
        Self::Change(
            iter.into_iter()
                .map(|(actuator, value)| ChangeSet { actuator, value })
                .collect(),
        )
    }
}

impl std::fmt::Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Motion::StopAll => write!(f, "Stop all"),
            Motion::ResumeAll => write!(f, "Resume all"),
            Motion::ResetAll => write!(f, "Reset all"),
            Motion::StraightDrive(value) => write!(f, "Straight drive: {}", value),
            Motion::Change(changes) => {
                let mut s = String::new();

                for changeset in changes {
                    s.push_str(&format!(
                        "Actuator: {:?}; Value: {}, ",
                        changeset.actuator,
                        match changeset.value {
                            Self::POWER_MIN => "Power minimum".to_string(),
                            Self::POWER_MAX => "Power maximum".to_string(),
                            _ => changeset.value.to_string(),
                        }
                    ));
                }

                write!(f, "Change: {}", s)
            }
        }
    }
}

impl TryFrom<Vec<u8>> for Motion {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        match buf.get_u8() {
            MOTION_TYPE_STOP_ALL => Ok(Motion::StopAll),
            MOTION_TYPE_RESUME_ALL => Ok(Motion::ResumeAll),
            MOTION_TYPE_RESET_ALL => Ok(Motion::ResetAll),
            MOTION_TYPE_STRAIGHT_DRIVE => Ok(Motion::StraightDrive(buf.get_i16())),
            MOTION_TYPE_CHANGE => {
                let count = buf.get_u8();
                let mut changes = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    changes.push(ChangeSet {
                        actuator: buf.get_u16().try_into().unwrap(),
                        value: buf.get_i16(),
                    });
                }
                Ok(Motion::Change(changes))
            }
            _ => Err(()),
        }
    }
}

impl crate::protocol::Packetize for Motion {
    const MESSAGE_TYPE: u8 = 0x20;

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(32);

        match self {
            Motion::StopAll => {
                buf.put_u8(MOTION_TYPE_STOP_ALL);
            }
            Motion::ResumeAll => {
                buf.put_u8(MOTION_TYPE_RESUME_ALL);
            }
            Motion::ResetAll => {
                buf.put_u8(MOTION_TYPE_RESET_ALL);
            }
            Motion::StraightDrive(value) => {
                buf.put_u8(MOTION_TYPE_STRAIGHT_DRIVE);
                buf.put_i16(*value);
            }
            Motion::Change(changes) => {
                buf.put_u8(MOTION_TYPE_CHANGE);
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

#[cfg(test)]
mod tests {
    use crate::protocol::Packetize;

    use super::*;

    #[test]
    fn test_motion() {
        let motion = Motion::new(Actuator::Boom, Motion::POWER_MAX);
        let bytes = motion.to_bytes();
        let motion2 = Motion::try_from(bytes).unwrap();

        assert_eq!(motion, motion2);
    }
}
