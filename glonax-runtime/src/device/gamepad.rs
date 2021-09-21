use std::path::Path;

use glonax_core::input::Scancode;
use glonax_gamepad::{Axis, Button, Event, EventType};

use super::{Device, InputDevice, IoDevice};

const DEVICE_NAME: &str = "gamepad";

pub struct Gamepad {
    driver: glonax_gamepad::Gamepad,
    reverse_left: bool,
    reverse_right: bool,
}

#[async_trait::async_trait]
impl IoDevice for Gamepad {
    const NAME: &'static str = DEVICE_NAME;

    async fn from_path(path: &std::path::Path) -> super::Result<Self> {
        Ok(Gamepad::new(path).await)
    }
}

impl Gamepad {
    async fn new(path: &Path) -> Self {
        Self {
            driver: glonax_gamepad::Gamepad::new(path).await.unwrap(),
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

#[async_trait::async_trait]
impl InputDevice for Gamepad {
    async fn next(&mut self) -> Option<Scancode> {
        if let Ok(event) = self.driver.next_event().await {
            match event {
                Event {
                    ty: EventType::Axis(Axis::LeftStickY),
                    ..
                } => Some(Scancode::LeftStickY(event.value_normal())),
                Event {
                    ty: EventType::Axis(Axis::LeftStickX),
                    ..
                } => Some(Scancode::LeftStickX(event.value_normal())),

                Event {
                    ty: EventType::Axis(Axis::RightStickY),
                    ..
                } => Some(Scancode::RightStickY(event.value_normal())),
                Event {
                    ty: EventType::Axis(Axis::RightStickX),
                    ..
                } => Some(Scancode::RightStickX(event.value_normal())),

                Event {
                    ty: EventType::Button(Button::LeftBumper),
                    ..
                } => {
                    self.reverse_left = if event.value == 1 { true } else { false };
                    None
                }
                Event {
                    ty: EventType::Button(Button::RightBumper),
                    ..
                } => {
                    self.reverse_right = if event.value == 1 { true } else { false };
                    None
                }

                Event {
                    ty: EventType::Axis(Axis::LeftTrigger),
                    ..
                } => Some(Scancode::LeftTrigger(if self.reverse_left {
                    -event.value_flatten_normal()
                } else {
                    event.value_flatten_normal()
                })),
                Event {
                    ty: EventType::Axis(Axis::RightTrigger),
                    ..
                } => Some(Scancode::RightTrigger(if self.reverse_right {
                    -event.value_flatten_normal()
                } else {
                    event.value_flatten_normal()
                })),

                Event {
                    ty: EventType::Button(Button::East),
                    ..
                } => Some(Scancode::Cancel),
                Event {
                    ty: EventType::Button(Button::South),
                    ..
                } => Some(Scancode::Activate),

                _ => None,
            }
        } else {
            None
        }
    }
}
