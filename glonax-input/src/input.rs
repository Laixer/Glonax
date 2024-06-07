use glonax::core::{Actuator, Motion, Object};

/// Level trait.
pub trait Level {
    /// Return the value of self above the lower threshold.
    /// Otherwise return a default value.
    fn ramp(self, lower: Self) -> Self;
}

/// Implement level trait for i16.
impl Level for i16 {
    /// Return the value of self above the lower threshold.
    /// Otherwise return a default value.
    fn ramp(self, lower: Self) -> Self {
        if self < lower && self > -lower {
            0
        } else {
            self
        }
    }
}

/// Button state.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ButtonState {
    /// Button pressed.
    Pressed,
    /// Button released.
    Released,
}

impl From<&i16> for ButtonState {
    fn from(value: &i16) -> Self {
        if value == &1 {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        }
    }
}

/// Input device scancode.
///
/// Scancodes are indirectly mapped to input pheripherials. Any
/// input device can emit these codes. Their effect is left to
/// device implementations.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Scancode {
    /// Slew axis.
    Slew(i16),
    /// Arm axis.
    Arm(i16),
    /// Attachment axis.
    Attachment(i16),
    /// Boom axis.
    Boom(i16),
    /// Left track axis.
    LeftTrack(i16),
    /// Right track axis.
    RightTrack(i16),
    /// Abort button.
    Abort(ButtonState),
    /// Confirm button.
    Confirm(ButtonState),
    /// Drive lock button.
    DriveLock(ButtonState),
    /// Up button.
    Up(ButtonState),
    /// Down button.
    Down(ButtonState),
    /// Left button.
    Left(ButtonState),
    /// Right button.
    Right(ButtonState),
}

pub(crate) struct InputState {
    /// Enable or disable drive lock.
    ///
    /// The drive lock locks both tracks together. Input on one track
    /// will be mirrored to the other track.
    pub(crate) drive_lock: bool,

    /// Enable or disable motion.
    ///
    /// This is a safety feature to prevent accidental motion. If motion
    /// is locked, the input device will not be able to move the actuators and
    /// no motion command will be sent to the vehicle.
    pub(crate) motion_lock: bool,

    /// Limit motion to lower values only.
    ///
    /// This prevents accidental damage by limiting the motion to lower
    /// values of the actuator.
    pub(crate) limit_motion: bool,

    /// The RPM (Revolutions Per Minute) of the engine.
    pub(crate) engine_rpm: i16,
}

impl InputState {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    pub(super) fn try_from(&mut self, input: Scancode) -> Option<Object> {
        match input {
            Scancode::Slew(value) => {
                if self.motion_lock {
                    return None;
                }

                Some(Object::Motion(Motion::new(Actuator::Slew, value)))
            }
            Scancode::Arm(value) => {
                if self.motion_lock {
                    return None;
                }

                Some(Object::Motion(Motion::new(Actuator::Arm, value)))
            }
            Scancode::Attachment(value) => {
                if self.motion_lock {
                    return None;
                }

                Some(Object::Motion(Motion::new(Actuator::Attachment, value)))
            }
            Scancode::Boom(value) => {
                if self.motion_lock {
                    return None;
                }

                Some(Object::Motion(Motion::new(Actuator::Boom, value)))
            }
            Scancode::LeftTrack(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Some(Object::Motion(Motion::StraightDrive(value)))
                } else {
                    Some(Object::Motion(Motion::new(Actuator::LimpLeft, value)))
                }
            }
            Scancode::RightTrack(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Some(Object::Motion(Motion::StraightDrive(value)))
                } else {
                    Some(Object::Motion(Motion::new(Actuator::LimpRight, value)))
                }
            }
            Scancode::Up(ButtonState::Pressed) => {
                if !self.motion_lock {
                    return None;
                }

                // TODO: Move somwhere else
                let rpm_new = (self.engine_rpm + 100).clamp(900, 2_100);
                self.engine_rpm = rpm_new;

                Some(Object::Engine(glonax::core::Engine::from_rpm(
                    self.engine_rpm as u16,
                )))
            }
            Scancode::Down(ButtonState::Pressed) => {
                if !self.motion_lock {
                    return None;
                }

                // TODO: Move somwhere else
                if self.engine_rpm <= 900 {
                    self.engine_rpm = 0;

                    return Some(Object::Engine(glonax::core::Engine::shutdown()));
                }

                let rpm_new = (self.engine_rpm - 100).clamp(900, 2_100);
                self.engine_rpm = rpm_new;

                Some(Object::Engine(glonax::core::Engine::from_rpm(
                    self.engine_rpm as u16,
                )))
            }
            Scancode::Abort(ButtonState::Pressed) => {
                self.motion_lock = true;
                Some(Object::Motion(Motion::StopAll))
            }
            Scancode::Abort(ButtonState::Released) => {
                self.motion_lock = false;
                Some(Object::Motion(Motion::ResumeAll))
            }
            Scancode::DriveLock(ButtonState::Pressed) => {
                self.drive_lock = true;
                None
            }
            Scancode::DriveLock(ButtonState::Released) => {
                self.drive_lock = false;
                Some(Object::Motion(Motion::StraightDrive(Motion::POWER_NEUTRAL)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp() {
        assert_eq!(120_i16.ramp(3_072), 0);
        assert_eq!(20_000_i16.ramp(3_072), 20_000);
        assert_eq!(-(10_i16.ramp(3_072)), 0);
        assert_eq!(-(5960_i16.ramp(3_072)), -5960);
    }
}
