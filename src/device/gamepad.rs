use gilrs::{Axis, Button, Event, EventType, Gilrs};

use super::{CommandDevice, CommandEvent, Device};

pub struct Gamepad {
    inner: Gilrs,
    reverse_left: bool,
    reverse_right: bool,
}

impl Gamepad {
    pub fn new() -> Self {
        Self {
            inner: Gilrs::new().unwrap(),
            reverse_left: false,
            reverse_right: false,
        }
    }
}

unsafe impl Send for Gamepad {}

impl Device for Gamepad {
    fn name(&self) -> String {
        "gamepad".to_owned()
    }
}

impl CommandDevice for Gamepad {
    fn next(&mut self) -> Option<CommandEvent> {
        if let Some(event) = self.inner.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::LeftStickY, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion { code: 0, value }),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::LeftStickX, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion { code: 1, value }),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::RightStickY, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion { code: 2, value }),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::RightStickX, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion { code: 3, value }),
                Event {
                    id: _,
                    event: EventType::ButtonChanged(Button::LeftTrigger, value, ..),
                    ..
                } => {
                    self.reverse_left = if value == 1.0 { true } else { false };
                    None
                }
                Event {
                    id: _,
                    event: EventType::ButtonChanged(Button::RightTrigger, value, ..),
                    ..
                } => {
                    self.reverse_right = if value == 1.0 { true } else { false };
                    None
                }
                Event {
                    id: _,
                    event: EventType::ButtonChanged(Button::LeftTrigger2, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion {
                    code: 5,
                    value: if self.reverse_left { -value } else { value },
                }),
                Event {
                    id: _,
                    event: EventType::ButtonChanged(Button::RightTrigger2, value, ..),
                    ..
                } => Some(CommandEvent::DirectMotion {
                    code: 6,
                    value: if self.reverse_right { -value } else { value },
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}
