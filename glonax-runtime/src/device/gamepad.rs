use gilrs::{Axis, Button, Event, EventType, Gilrs};
use glonax_core::input::Scancode;

use super::{CommandDevice, Device};

const DEVICE_NAME: &str = "gamepad";

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
        DEVICE_NAME.to_owned()
    }
}

impl CommandDevice for Gamepad {
    fn next(&mut self) -> Option<Scancode> {
        if let Some(event) = self.inner.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::LeftStickY, value, ..),
                    ..
                } => Some(Scancode::LeftStickY(value)),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::LeftStickX, value, ..),
                    ..
                } => Some(Scancode::LeftStickX(value)),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::RightStickY, value, ..),
                    ..
                } => Some(Scancode::RightStickY(value)),
                Event {
                    id: _,
                    event: EventType::AxisChanged(Axis::RightStickX, value, ..),
                    ..
                } => Some(Scancode::RightStickX(value)),
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
                } => Some(Scancode::LeftTrigger(if self.reverse_left {
                    -value
                } else {
                    value
                })),
                Event {
                    id: _,
                    event: EventType::ButtonChanged(Button::RightTrigger2, value, ..),
                    ..
                } => Some(Scancode::RightTrigger(if self.reverse_right {
                    -value
                } else {
                    value
                })),
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::East, ..),
                    ..
                } => Some(Scancode::Cancel),
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::South, ..),
                    ..
                } => Some(Scancode::Activate),
                _ => None,
            }
        } else {
            None
        }
    }
}
