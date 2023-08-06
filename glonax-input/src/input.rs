use glonax::core::{Actuator, Level, Motion};

/// Button state.
#[derive(PartialEq, Eq)]
pub(crate) enum ButtonState {
    /// Button pressed.
    Pressed,
    /// Button released.
    Released,
}

/// Input device scancode.
///
/// Scancodes are indirectly mapped to input pheripherials. Any
/// input device can emit these codes. Their effect is left to
/// device implementations.
#[derive(PartialEq, Eq)]
pub(crate) enum Scancode {
    /// Left stick X axis.
    LeftStickX(i16),
    /// Left stick Y axis.
    LeftStickY(i16),
    /// Right stick X axis.
    RightStickX(i16),
    /// Right stick Y axis.
    RightStickY(i16),
    /// Left trigger axis.
    LeftTrigger(i16),
    /// Right trigger axis.
    RightTrigger(i16),
    /// Abort button.
    Abort(ButtonState),
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
            Scancode::LeftStickX(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(
                    Actuator::Slew,
                    if self.limit_motion {
                        (value / 2).ramp(1_000)
                    } else {
                        value.ramp(2_000)
                    },
                )
                .into()
            }
            Scancode::LeftStickY(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(
                    Actuator::Arm,
                    if self.limit_motion {
                        (value / 2).ramp(1_500)
                    } else {
                        value.ramp(3_000)
                    },
                )
                .into()
            }
            Scancode::RightStickX(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(
                    Actuator::Attachment,
                    if self.limit_motion {
                        if value.is_negative() {
                            (value / 2).ramp(2_000)
                        } else {
                            value.ramp(4_000)
                        }
                    } else {
                        value.ramp(4_000)
                    },
                )
                .into()
            }
            Scancode::RightStickY(value) => {
                if self.motion_lock {
                    return None;
                }

                Motion::new(
                    Actuator::Boom,
                    if self.limit_motion {
                        if value.is_negative() {
                            value.ramp(3_500)
                        } else {
                            (value / 2).ramp(1_750)
                        }
                    } else {
                        value.ramp(3_500)
                    },
                )
                .into()
            }
            Scancode::LeftTrigger(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Motion::StraightDrive(value.ramp(2_000)).into()
                } else {
                    Motion::new(Actuator::LimpLeft, value.ramp(2_000)).into()
                }
            }
            Scancode::RightTrigger(value) => {
                if self.motion_lock {
                    return None;
                }

                if self.drive_lock {
                    Motion::StraightDrive(value.ramp(2_000)).into()
                } else {
                    Motion::new(Actuator::LimpRight, value.ramp(2_000)).into()
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
        }
    }
}
