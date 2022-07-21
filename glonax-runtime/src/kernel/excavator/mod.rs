use crate::{
    core::{
        input::{ButtonState, Scancode},
        motion::{Motion, ToMotion},
        Identity,
    },
    runtime::operand::{Operand, Parameter, Program},
};

mod arm_balance;
mod arm_fk;
mod arm_fk2;
mod arm_fk3;
mod arm_ik;
mod bucket;
mod drive;
mod noop;
mod turn;

// TODO: take all lengths in mm.

/// Maximum empirical driving speed in meters per second.
const DRIVE_SPEED_MAX: f32 = 26.1 / 30.0;
/// Boom length in meters.
const BOOM_LENGTH: f32 = 6.0;
/// Arm length in meters.
const ARM_LENGTH: f32 = 2.97;
// TODO: Rename. This is not an height but an transformation.
/// Frame height in meters.
const FRAME_HEIGHT: f32 = 1.885;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Actuator {
    Boom = 0,
    Arm = 4,
    Bucket = 5,
    Slew = 1,
    LimpLeft = 3,
    LimpRight = 2,
}

impl From<Actuator> for u32 {
    fn from(value: Actuator) -> Self {
        value as u32
    }
}

enum BodyPart {
    // Arm = 0x7c0,
    Boom = 0x6d0,
    Arm = 0x6c0,
}

impl From<BodyPart> for crate::core::metric::SignalSource {
    fn from(value: BodyPart) -> Self {
        value as crate::core::metric::SignalSource
    }
}

#[derive(Clone, Copy)]
pub struct Excavator {
    slow_motion: bool,
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

// TODO: Move somewhere.
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

pub enum HydraulicMotion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Drive straight forward or backwards.
    StraightDrive(i16),
    /// Stop motion on actuators.
    Stop(Vec<Actuator>),
    /// Slow motion on actuators.
    Slow(Vec<(Actuator, i16)>),
    /// Change motion on actuators.
    Change(Vec<(Actuator, i16)>),
}

impl HydraulicMotion {
    fn from_scancode(actuator: Actuator, value: i16, slow_motion: bool) -> Self {
        let value_normal = match actuator {
            Actuator::Boom => value.ramp(3072),
            Actuator::Arm => value.ramp(3072),
            Actuator::Bucket => value.ramp(4096),
            Actuator::Slew => value.ramp(3072),
            Actuator::LimpLeft => value.ramp(2048),
            Actuator::LimpRight => value.ramp(2048),
        };

        if slow_motion {
            HydraulicMotion::Slow(vec![(actuator, value_normal)])
        } else {
            HydraulicMotion::Change(vec![(actuator, value_normal)])
        }
    }
}

impl ToMotion for HydraulicMotion {
    fn to_motion(self) -> Motion {
        match self {
            HydraulicMotion::StopAll => Motion::StopAll,
            HydraulicMotion::ResumeAll => Motion::ResumeAll,
            HydraulicMotion::StraightDrive(value) => Motion::Change(vec![
                (Actuator::LimpLeft.into(), value),
                (Actuator::LimpRight.into(), value),
            ]),
            HydraulicMotion::Stop(v) => Motion::Stop(v.iter().map(|a| (*a).into()).collect()),
            HydraulicMotion::Slow(v) => {
                Motion::Change(v.iter().map(|(a, va)| ((*a).into(), (*va) / 4)).collect())
            }
            HydraulicMotion::Change(v) => {
                Motion::Change(v.iter().map(|(a, va)| ((*a).into(), *va)).collect())
            }
        }
    }
}

impl Operand for Excavator {
    /// Construct operand from configuration.
    fn from_config(_config: &crate::Config) -> Self {
        Self { slow_motion: false }
    }

    type MotionPlan = HydraulicMotion;

    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    fn try_from_input_device(&mut self, input: Scancode) -> Result<Self::MotionPlan, ()> {
        match input {
            Scancode::LeftStickX(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::Slew,
                value,
                self.slow_motion,
            )),
            Scancode::LeftStickY(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::Arm,
                value,
                self.slow_motion,
            )),
            Scancode::RightStickX(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::Bucket,
                value,
                self.slow_motion,
            )),
            Scancode::RightStickY(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::Boom,
                value,
                self.slow_motion,
            )),
            Scancode::LeftTrigger(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::LimpLeft,
                value,
                self.slow_motion,
            )),
            Scancode::RightTrigger(value) => Ok(HydraulicMotion::from_scancode(
                Actuator::LimpRight,
                value,
                self.slow_motion,
            )),
            Scancode::Cancel(ButtonState::Pressed) => Ok(HydraulicMotion::StopAll),
            Scancode::Cancel(ButtonState::Released) => Ok(HydraulicMotion::ResumeAll),
            Scancode::Restrict(ButtonState::Pressed) => {
                self.slow_motion = true;
                Err(())
            }
            Scancode::Restrict(ButtonState::Released) => {
                self.slow_motion = false;
                Err(())
            }
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
    ) -> Result<Box<dyn Program<MotionPlan = Self::MotionPlan> + Send + Sync>, ()> {
        match order {
            // Arm chain programs.
            600 => Ok(Box::new(arm_balance::ArmBalanceProgram::new())),
            601 => Ok(Box::new(arm_fk::ArmFkProgram::new())),
            602 => Ok(Box::new(arm_ik::ArmIkProgram::new())),
            603 => Ok(Box::new(arm_fk2::ArmFk2Program::new())),
            604 => Ok(Box::new(arm_fk3::ArmFk3Program::new())),

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
