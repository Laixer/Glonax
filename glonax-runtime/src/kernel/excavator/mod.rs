use crate::{
    core::{
        input::{ButtonState, Scancode},
        motion::{Motion, ToMotion},
        Identity, Level,
    },
    runtime::operand::{Operand, Parameter, Program, ProgramFactory},
};

mod arm_balance;
mod arm_fk2;
mod drive;
mod halt;
mod noop;
mod sleep;
mod test;
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
    Boom = 0x6a0,
    Arm = 0x6c0,
}

impl From<BodyPart> for crate::core::metric::SignalSource {
    fn from(value: BodyPart) -> Self {
        value as crate::core::metric::SignalSource
    }
}

#[derive(Clone, Copy)]
pub struct Excavator {
    drive_lock: bool,
}

impl Identity for Excavator {
    /// Operand intro message.
    ///
    /// The introduction message makes it easier to spot the current running
    /// configuration. The message is printed with the information level.
    fn intro() -> String {
        format!(
            "Hello, I'm an {} 🏗. Gimme som dirt! ⚒️",
            ansi_term::Color::Yellow.paint("excavator")
        )
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
    pub(super) const POWER_MAX: i16 = i16::MAX;
    pub(super) const POWER_NEUTRAL: i16 = 0;
    pub(super) const POWER_MIN: i16 = i16::MIN;
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

impl Default for Excavator {
    fn default() -> Self {
        Self { drive_lock: false }
    }
}

impl Operand for Excavator {
    /// Construct operand from configuration.
    fn from_config<C: crate::config::Configurable>(_config: &C) -> Self {
        Self { drive_lock: false }
    }

    type MotionPlan = HydraulicMotion;

    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    fn try_from_input_device(&mut self, input: Scancode) -> Result<Self::MotionPlan, ()> {
        match input {
            Scancode::LeftStickX(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Slew,
                value.ramp(3072),
            )])),
            Scancode::LeftStickY(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Arm,
                value.ramp(3072),
            )])),
            Scancode::RightStickX(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Bucket,
                value.ramp(4096),
            )])),
            Scancode::RightStickY(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Boom,
                value.ramp(3072),
            )])),
            Scancode::LeftTrigger(value) => {
                if self.drive_lock {
                    Ok(HydraulicMotion::StraightDrive(value.ramp(2048)))
                } else {
                    Ok(HydraulicMotion::Change(vec![(
                        Actuator::LimpLeft,
                        value.ramp(2048),
                    )]))
                }
            }
            Scancode::RightTrigger(value) => {
                if self.drive_lock {
                    Ok(HydraulicMotion::StraightDrive(value.ramp(2048)))
                } else {
                    Ok(HydraulicMotion::Change(vec![(
                        Actuator::LimpRight,
                        value.ramp(2048),
                    )]))
                }
            }
            Scancode::Cancel(ButtonState::Pressed) => Ok(HydraulicMotion::StopAll),
            Scancode::Cancel(ButtonState::Released) => Ok(HydraulicMotion::ResumeAll),
            Scancode::Restrict(ButtonState::Pressed) => {
                self.drive_lock = true;
                Err(())
            }
            Scancode::Restrict(ButtonState::Released) => {
                self.drive_lock = false;
                Ok(HydraulicMotion::StraightDrive(
                    HydraulicMotion::POWER_NEUTRAL,
                ))
            }
            _ => {
                warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }
}

impl ProgramFactory for Excavator {
    type MotionPlan = HydraulicMotion;

    fn fetch_program(
        &self,
        id: i32,
        params: Parameter,
    ) -> Result<Box<dyn Program<MotionPlan = Self::MotionPlan> + Send + Sync>, ()> {
        match id {
            // Arm chain programs.
            600 => Ok(Box::new(arm_balance::ArmBalanceProgram::new())),
            603 => Ok(Box::new(arm_fk2::ArmFk2Program::new(params))),

            // Movement programs.
            700 => Ok(Box::new(drive::DriveProgram::new(params))),
            701 => Ok(Box::new(turn::TurnProgram::new(params))),

            // Miscellaneous programs.
            900 => Ok(Box::new(noop::NoopProgram::new())),
            901 => Ok(Box::new(sleep::SleepProgram::new(params))),
            902 => Ok(Box::new(halt::HaltProgram::new())),
            910 => Ok(Box::new(test::TestProgram::new())),

            _ => Err(()),
        }
    }
}
