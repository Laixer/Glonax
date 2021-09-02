use crate::runtime::{Motion, NormalControl, Operand, Scancode};

pub mod arm_balance;
pub mod drive;

enum Actuator {
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

#[derive(Clone, Copy)]
pub struct Excavator;

impl Default for Excavator {
    fn default() -> Self {
        Self {}
    }
}

impl Operand for Excavator {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    fn try_from_input_device(&self, input: Scancode) -> std::result::Result<Motion, ()> {
        match input {
            Scancode::LeftStickX(value) => {
                Ok(NormalControl::new(Actuator::Slew.into(), value).into())
            }
            Scancode::LeftStickY(value) => {
                Ok(NormalControl::new(Actuator::Arm.into(), value).into())
            }
            Scancode::RightStickX(value) => {
                Ok(NormalControl::new(Actuator::Bucket.into(), value).into())
            }
            Scancode::RightStickY(value) => {
                Ok(NormalControl::new(Actuator::Boom.into(), value).into())
            }
            Scancode::LeftTrigger(value) => {
                Ok(NormalControl::new(Actuator::LimpLeft.into(), value).into())
            }
            Scancode::RightTrigger(value) => {
                Ok(NormalControl::new(Actuator::LimpRight.into(), value).into())
            }
            Scancode::Cancel => Ok(Motion::StopAll),
            _ => {
                warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }
}
