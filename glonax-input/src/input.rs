use glonax::core::Level;

use crate::motion::{Actuator, HydraulicMotion};

/// Button state.
#[derive(PartialEq, Eq)]
pub enum ButtonState {
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
pub enum Scancode {
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
    pub(crate) fn try_from(&mut self, input: Scancode) -> Option<HydraulicMotion> {
        match input {
            Scancode::LeftStickX(value) => {
                if self.motion_lock {
                    None
                } else {
                    Some(HydraulicMotion::Change(vec![(
                        Actuator::Slew,
                        if self.limit_motion {
                            value.ramp(3_000).clamp(-15_000, 15_000)
                        } else {
                            value.ramp(3_000)
                        },
                    )]))
                }
            }
            Scancode::LeftStickY(value) => {
                if self.motion_lock {
                    None
                } else {
                    Some(HydraulicMotion::Change(vec![(
                        Actuator::Arm,
                        if self.limit_motion {
                            value.ramp(3_000).clamp(-17_000, 17_000)
                        } else {
                            value.ramp(3_000)
                        },
                    )]))
                }
            }
            Scancode::RightStickX(value) => {
                if self.motion_lock {
                    None
                } else {
                    Some(HydraulicMotion::Change(vec![(
                        Actuator::Bucket,
                        value.ramp(4096).clamp(-14_000, 32_000),
                    )]))
                }
            }
            Scancode::RightStickY(value) => {
                if self.motion_lock {
                    None
                } else {
                    Some(HydraulicMotion::Change(vec![(
                        Actuator::Boom,
                        if self.limit_motion {
                            value.ramp(3_000).clamp(i16::MIN, 17_000)
                        } else {
                            value.ramp(3_000)
                        },
                    )]))
                }
            }
            Scancode::LeftTrigger(value) => {
                if self.motion_lock {
                    None
                } else {
                    if self.drive_lock {
                        Some(HydraulicMotion::StraightDrive(value.ramp(2_000)))
                    } else {
                        Some(HydraulicMotion::Change(vec![(
                            Actuator::LimpLeft,
                            value.ramp(2_000),
                        )]))
                    }
                }
            }
            Scancode::RightTrigger(value) => {
                if self.motion_lock {
                    None
                } else {
                    if self.drive_lock {
                        Some(HydraulicMotion::StraightDrive(value.ramp(2_000)))
                    } else {
                        Some(HydraulicMotion::Change(vec![(
                            Actuator::LimpRight,
                            value.ramp(2_000),
                        )]))
                    }
                }
            }
            Scancode::Abort(ButtonState::Pressed) => {
                self.motion_lock = true;
                Some(HydraulicMotion::StopAll)
            }
            Scancode::Abort(ButtonState::Released) => {
                self.motion_lock = false;
                Some(HydraulicMotion::ResumeAll)
            }
            Scancode::DriveLock(ButtonState::Pressed) => {
                self.drive_lock = true;
                None
            }
            Scancode::DriveLock(ButtonState::Released) => {
                self.drive_lock = false;
                Some(HydraulicMotion::StraightDrive(
                    HydraulicMotion::POWER_NEUTRAL,
                ))
            }
        }
    }
}
