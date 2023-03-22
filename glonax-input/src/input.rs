use glonax::core::{
    input::{ButtonState, Scancode},
    Level,
};

use crate::motion::{Actuator, HydraulicMotion};

pub(crate) struct InputState {
    /// Enable or disable drive lock.
    ///
    /// The drive locks allows two actuators to act at the same
    /// time with a single command.
    pub(crate) drive_lock: bool,
}

impl InputState {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    pub(crate) fn try_from_input_device(&mut self, input: Scancode) -> Result<HydraulicMotion, ()> {
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
                log::warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }
}
