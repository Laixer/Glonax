use crate::{
    input::{ButtonState, Level, Scancode},
    joystick::{Event, EventType},
};

pub trait InputDevice {
    /// Maps the given event to a scancode.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be mapped.
    ///
    /// # Returns
    ///
    /// An optional scancode if the event can be mapped, or `None` otherwise.
    fn map(&mut self, event: &Event) -> Option<Scancode>;
}

#[derive(Default)]
pub struct XboxController {
    reverse_left: bool,
    reverse_right: bool,
}

impl InputDevice for XboxController {
    /// Maps the given `Event` to a corresponding `Scancode`.
    ///
    /// # Arguments
    ///
    /// * `event` - The input event to be mapped.
    ///
    /// # Returns
    ///
    /// An optional `Scancode` representing the mapped input event, or `None` if the event does not match any mapping.
    fn map(&mut self, event: &Event) -> Option<Scancode> {
        match event {
            Event {
                ty: EventType::Axis(1),
                ..
            } => Some(Scancode::Arm(event.value)),
            Event {
                ty: EventType::Axis(0),
                ..
            } => Some(Scancode::Slew(event.value)),
            Event {
                ty: EventType::Axis(4),
                ..
            } => Some(Scancode::Boom(event.value)),
            Event {
                ty: EventType::Axis(3),
                ..
            } => Some(Scancode::Attachment(event.value)),
            Event {
                ty: EventType::Button(4),
                ..
            } => {
                self.reverse_left = event.value == 1;
                None
            }
            Event {
                ty: EventType::Button(5),
                ..
            } => {
                self.reverse_right = event.value == 1;
                None
            }
            Event {
                ty: EventType::Axis(2),
                ..
            } => Some(Scancode::LeftTrack(if self.reverse_left {
                -(((event.value as i32 - i16::MAX as i32) / 2).abs() as i16)
            } else {
                ((event.value as i32 - i16::MAX as i32) / 2).abs() as i16
            })),
            Event {
                ty: EventType::Axis(5),
                ..
            } => Some(Scancode::RightTrack(if self.reverse_right {
                -(((event.value as i32 - i16::MAX as i32) / 2).abs() as i16)
            } else {
                ((event.value as i32 - i16::MAX as i32) / 2).abs() as i16
            })),
            Event {
                ty: EventType::Axis(7),
                ..
            } => match event.value {
                value if value > 0 => Some(Scancode::Up(ButtonState::Pressed)),
                value if value < 0 => Some(Scancode::Down(ButtonState::Pressed)),
                _ => None,
            },
            Event {
                ty: EventType::Button(0),
                value,
                ..
            } => Some(Scancode::Confirm(ButtonState::from(value))),
            Event {
                ty: EventType::Button(1),
                value,
                ..
            } => Some(Scancode::Abort(ButtonState::from(value))),
            Event {
                ty: EventType::Button(2),
                value,
                ..
            } => Some(Scancode::DriveLock(ButtonState::from(value))),
            Event {
                ty: EventType::Button(3),
                value,
                ..
            } => Some(Scancode::LimitMotion(ButtonState::from(value))),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum LogitechJoystickMode {
    Left,
    Right,
    Solo,
}

pub struct LogitechJoystick {
    mode: LogitechJoystickMode,
}

impl LogitechJoystick {
    pub fn solo_mode() -> Self {
        Self {
            mode: LogitechJoystickMode::Solo,
        }
    }

    pub fn left_mode() -> Self {
        Self {
            mode: LogitechJoystickMode::Left,
        }
    }

    pub fn right_mode() -> Self {
        Self {
            mode: LogitechJoystickMode::Right,
        }
    }
}

impl InputDevice for LogitechJoystick {
    /// Maps the given `Event` to a corresponding `Scancode`.
    ///
    /// # Arguments
    ///
    /// * `event` - The input event to be mapped.
    ///
    /// # Returns
    ///
    /// An optional `Scancode` representing the mapped input event, or `None` if the event does not match any mapping.
    fn map(&mut self, event: &Event) -> Option<Scancode> {
        match event {
            Event {
                ty: EventType::Axis(1),
                ..
            } => Some(if self.mode == LogitechJoystickMode::Right {
                Scancode::Boom(if event.value.is_negative() {
                    event.value.ramp(3_500)
                } else {
                    (event.value / 2).ramp(1_750)
                })
            } else {
                Scancode::Arm((event.value / 2).ramp(1_500))
            }),
            Event {
                ty: EventType::Axis(0),
                ..
            } => Some(if self.mode == LogitechJoystickMode::Right {
                Scancode::Attachment(if event.value.is_negative() {
                    (event.value / 2).ramp(2_000)
                } else {
                    event.value.ramp(4_000)
                })
            } else {
                Scancode::Slew((event.value / 2).ramp(1_000))
            }),
            Event {
                ty: EventType::Button(1),
                value,
                ..
            } => Some(Scancode::Abort(ButtonState::from(value))),
            Event {
                ty: EventType::Button(6),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("Idle 1: {}", value);
                }
                None
            }
            Event {
                ty: EventType::Button(7),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("Idle 2: {}", value);
                }
                None
            }
            Event {
                ty: EventType::Button(8),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("Fine 1: {}", value);
                }
                None
            }
            Event {
                ty: EventType::Button(9),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("Fine 2: {}", value);
                }
                None
            }
            Event {
                ty: EventType::Button(10),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("General 1: {}", value);
                }
                None
            }
            Event {
                ty: EventType::Button(11),
                value,
                ..
            } => {
                if self.mode != LogitechJoystickMode::Right {
                    log::info!("Shutdown: {}", value);
                }
                None
            }
            _ => None,
        }
    }
}
