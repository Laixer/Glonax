use std::path::Path;

use glonax_core::input::Scancode;

use super::{
    driver::gamepad::{Axis, Button, Event, EventType},
    CommandDevice, Device,
};

const DEVICE_NAME: &str = "gamepad";

#[allow(dead_code)]
pub struct Gamepad {
    driver: super::driver::gamepad::Gamepad,
    reverse_left: bool,
    reverse_right: bool,
}

#[allow(dead_code)]
impl Gamepad {
    pub async fn new(path: &Path) -> Self {
        Self {
            driver: super::driver::gamepad::Gamepad::new(path).await.unwrap(),
            reverse_left: false,
            reverse_right: false,
        }
    }

    async fn next_async(&mut self) -> Option<Scancode> {
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

unsafe impl Send for Gamepad {}

impl Device for Gamepad {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

impl CommandDevice for Gamepad {
    fn next(&mut self) -> Option<Scancode> {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(self.next_async())
    }
}
