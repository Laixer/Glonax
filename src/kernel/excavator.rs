use crate::runtime::{Motion, NormalControl, Scancode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Actuator {
    Boom = 2,
    Arm = 1,
    Bucket = 0,
    Slew = 3,
    LimpLeft = 4,
    LimpRight = 5,
}

impl From<Actuator> for u32 {
    fn from(value: Actuator) -> Self {
        value as u32
    }
}

pub trait Operand {
    fn try_from_input_device(&self, input: Scancode) -> std::result::Result<Motion, ()>;
}

#[derive(Clone, Copy)]
pub struct Excavator {
    //
}

impl Excavator {}

impl Operand for Excavator {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    fn try_from_input_device(&self, input: Scancode) -> std::result::Result<Motion, ()> {
        match input {
            Scancode::LeftStickX(value) => Ok(NormalControl {
                actuator: Actuator::Slew.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::LeftStickY(value) => Ok(NormalControl {
                actuator: Actuator::Arm.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::RightStickX(value) => Ok(NormalControl {
                actuator: Actuator::Bucket.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::RightStickY(value) => Ok(NormalControl {
                actuator: Actuator::Boom.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::LeftTrigger(value) => Ok(NormalControl {
                actuator: Actuator::LimpLeft.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::RightTrigger(value) => Ok(NormalControl {
                actuator: Actuator::LimpRight.into(),
                value,
                ..Default::default()
            }
            .to_motion()),
            Scancode::Cancel => Ok(Motion::StopAll),
            _ => {
                warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }
}
