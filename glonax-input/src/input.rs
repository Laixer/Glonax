use glonax::core::{Actuator, Motion};

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
}

impl InputState {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    pub(super) fn try_from(&mut self, input: Scancode) -> Option<Motion> {
        match input {
            Scancode::Slew(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(Actuator::Slew, value).into()
            }
            Scancode::Arm(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(Actuator::Arm, value).into()
            }
            Scancode::Attachment(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(Actuator::Attachment, value).into()
            }
            Scancode::Boom(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(Actuator::Boom, value).into()
            }
            Scancode::LeftTrack(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Motion::StraightDrive(value).into()
                } else {
                    Motion::new(Actuator::LimpLeft, value).into()
                }
            }
            Scancode::RightTrack(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Motion::StraightDrive(value).into()
                } else {
                    Motion::new(Actuator::LimpRight, value).into()
                }
            }
            Scancode::Abort(ButtonState::Pressed) => {
                self.motion_lock = true;
                Motion::StopAll.into()
            }
            Scancode::Abort(ButtonState::Released) => {
                self.motion_lock = false;
                Motion::ResumeAll.into()
            }
            Scancode::DriveLock(ButtonState::Pressed) => {
                self.drive_lock = true;
                None
            }
            Scancode::DriveLock(ButtonState::Released) => {
                self.drive_lock = false;
                Motion::StraightDrive(Motion::POWER_NEUTRAL).into()
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
