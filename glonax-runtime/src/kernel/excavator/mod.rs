use std::convert::TryFrom;

use glonax_core::{
    input::{ButtonState, Scancode},
    motion::Motion,
    Identity,
};

use crate::runtime::operand::{Operand, Parameter, Program};

mod arm_balance;
mod arm_fk;
mod arm_ik;
mod bucket;
mod drive;
mod noop;
mod turn;

/// Maximum empirical driving speed in meters per second.
const DRIVE_SPEED_MAX: f32 = 26.1 / 30.0;
/// Boom length in meters.
const BOOM_LENGTH: f32 = 6.0;
/// Arm length in meters.
const ARM_LENGTH: f32 = 2.97;
// TODO: Rename. This is not an height but an transformation.
/// Frame height in meters.
const FRAME_HEIGHT: f32 = 1.88;
/// Arm angle range.
#[allow(dead_code)]
const ARM_RANGE: std::ops::Range<f32> = -0.45..-2.47;

/// Frame dimensions in (L)x(W)x(H)
#[allow(dead_code)]
const FRAME_DIMENSIONS: (f32, f32, f32) = (3.88, 2.89, 1.91);
// TODO: track hight.
/// Track dimensions in (L)x(W)x(H)
#[allow(dead_code)]
const TRACK_DIMENSIONS: (f32, f32, f32) = (4.65, 0.9, 0.0);

#[allow(dead_code)]
const SERVICE_POSITION_A: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_B: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_C: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_D: (f32, f32) = (0.0, 0.0);

#[derive(Debug)]
enum Actuator {
    Boom = 2,
    Arm = 6,
    Bucket = 7,
    Slew = 3,
    LimpLeft = 5,
    LimpRight = 4,
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
        format!(
            "Hello, I'm an {} 🏗. Gimme som dirt! ⚒️",
            ansi_term::Color::Yellow.paint("excavator")
        )
    }
}

trait Level {
    fn ramp(self, lower: Self) -> Self;
}

impl Level for i16 {
    fn ramp(self, lower: Self) -> Self {
        if self < lower && self > -lower {
            0
        } else {
            self
        }
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
            Scancode::LeftStickX(value) => Ok(Motion::Change(vec![(
                Actuator::Slew.into(),
                value.ramp(3072),
            )])),
            Scancode::LeftStickY(value) => Ok(Motion::Change(vec![(
                Actuator::Arm.into(),
                value.ramp(3072),
            )])),
            Scancode::RightStickX(value) => Ok(Motion::Change(vec![(
                Actuator::Bucket.into(),
                value.ramp(4096),
            )])),
            Scancode::RightStickY(value) => Ok(Motion::Change(vec![(
                Actuator::Boom.into(),
                value.ramp(3072),
            )])),
            Scancode::LeftTrigger(value) => Ok(Motion::Change(vec![(
                Actuator::LimpLeft.into(),
                value.ramp(2048),
            )])),
            Scancode::RightTrigger(value) => Ok(Motion::Change(vec![(
                Actuator::LimpRight.into(),
                value.ramp(2048),
            )])),
            Scancode::Cancel(ButtonState::Pressed) => Ok(Motion::StopAll),
            Scancode::Cancel(ButtonState::Released) => Ok(Motion::ResumeAll),
            _ => {
                warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }

    /// Fetch program by identifier.
    ///
    /// The factory method returns a pointer to the excavator program.
    fn fetch_program(
        &self,
        order: i32,
        params: Parameter,
    ) -> Result<Box<dyn Program + Send + Sync>, ()> {
        match order {
            // Arm chain programs.
            600 => Ok(Box::new(arm_balance::ArmBalanceProgram::new())),
            601 => Ok(Box::new(arm_fk::ArmFkProgram::new())),
            602 => Ok(Box::new(arm_ik::ArmIkProgram::new())),

            // Movement programs.
            700 => Ok(Box::new(drive::DriveProgram::new(params))),
            701 => Ok(Box::new(turn::TurnProgram::new(params))),
            702 => Ok(Box::new(bucket::BucketProgram::new())),

            // Miscellaneous programs.
            900 => Ok(Box::new(noop::NoopProgram::new())),

            _ => Err(()),
        }
    }
}
