use std::convert::TryFrom;

use glonax_core::{
    input::{ButtonState, Scancode},
    motion::{Motion, NormalControl},
    operand::{Operand, Program},
    Identity,
};

use self::{
    arm::ArmProgram, arm_balance::ArmBalanceProgram, drive::DriveProgram, turn::TurnProgram,
};

mod arm;
mod arm_balance;
mod drive;
mod turn;

#[derive(Debug)]
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

#[derive(Debug)]
enum Metric {
    Bucket = 0,
    Arm = 9,
    Boom = 2,
    Frame = 3,
}

impl TryFrom<u32> for Metric {
    type Error = (); // TODO

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            v if Metric::Bucket as u32 == v => Ok(Metric::Bucket),
            v if Metric::Arm as u32 == v => Ok(Metric::Arm),
            v if Metric::Boom as u32 == v => Ok(Metric::Boom),
            v if Metric::Frame as u32 == v => Ok(Metric::Frame),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Excavator;

impl Default for Excavator {
    fn default() -> Self {
        Self {}
    }
}

impl Identity for Excavator {
    /// The introduction message makes it easier to spot the current running
    /// configuration.
    fn intro() -> String {
        "Hello, I'm an excavator 🏗. Lets go diggin'!".to_owned()
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
            Scancode::Cancel(ButtonState::Pressed) => Ok(Motion::StopAll),
            Scancode::Cancel(ButtonState::Released) => Err(()),
            _ => {
                warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }

    /// Fetch program from identifier.
    ///
    /// The method returns a pointer to the excavator program.
    fn fetch_program(&self, order: i32) -> Box<dyn Program + Send + Sync> {
        match order {
            600 => Box::new(ArmBalanceProgram::new()),
            601 => Box::new(ArmProgram::new()),
            700 => Box::new(DriveProgram::new()),
            701 => Box::new(TurnProgram::new()),
            _ => Box::new(DriveProgram::new()),
        }
    }
}
