use std::path::{Path, PathBuf};

use glonax::driver;
use glonax_gamepad::{Axis, Button, Event, EventType};
use log::error;

use crate::input::{ButtonState, Scancode};

const DEVICE_NAME: &str = "gamepad";

pub(crate) struct Gamepad {
    driver: glonax_gamepad::AsyncGamepad,
    node_path: PathBuf,
    reverse_left: bool,
    reverse_right: bool,
}

impl Gamepad {
    pub(crate) async fn new(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            driver: glonax_gamepad::AsyncGamepad::new(path).await?,
            node_path: path.to_path_buf(),
            reverse_left: false,
            reverse_right: false,
        })
    }
}

impl Gamepad {
    pub(super) async fn next(&mut self) -> driver::Result<Scancode> {
        loop {
            match self.driver.next_event().await {
                Ok(event) => match event {
                    Event {
                        ty: EventType::Axis(Axis::LeftStickY),
                        ..
                    } => break Ok(Scancode::LeftStickY(event.value)),
                    Event {
                        ty: EventType::Axis(Axis::LeftStickX),
                        ..
                    } => break Ok(Scancode::LeftStickX(event.value)),

                    Event {
                        ty: EventType::Axis(Axis::RightStickY),
                        ..
                    } => break Ok(Scancode::RightStickY(event.value)),
                    Event {
                        ty: EventType::Axis(Axis::RightStickX),
                        ..
                    } => break Ok(Scancode::RightStickX(event.value)),
                    Event {
                        ty: EventType::Button(Button::LeftBumper),
                        ..
                    } => {
                        self.reverse_left = event.value == 1;
                    }
                    Event {
                        ty: EventType::Button(Button::RightBumper),
                        ..
                    } => {
                        self.reverse_right = event.value == 1;
                    }
                    Event {
                        ty: EventType::Axis(Axis::LeftTrigger),
                        ..
                    } => {
                        break Ok(Scancode::LeftTrigger(if self.reverse_left {
                            ((event.value as i32 - i16::MAX as i32) / 2) as i16
                        } else {
                            (((event.value as i32 - i16::MAX as i32) / 2) as i16).abs()
                        }))
                    }
                    Event {
                        ty: EventType::Axis(Axis::RightTrigger),
                        ..
                    } => {
                        break Ok(Scancode::RightTrigger(if self.reverse_right {
                            ((event.value as i32 - i16::MAX as i32) / 2) as i16
                        } else {
                            (((event.value as i32 - i16::MAX as i32) / 2) as i16).abs()
                        }))
                    }
                    Event {
                        ty: EventType::Button(Button::East),
                        value,
                    } => {
                        break Ok(Scancode::Abort(if value == 1 {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        }))
                    }
                    Event {
                        ty: EventType::Button(Button::West),
                        value,
                    } => {
                        break Ok(Scancode::DriveLock(if value == 1 {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        }))
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("{}", e);
                    break Err(driver::DeviceError::no_such_device(
                        DEVICE_NAME.to_string(),
                        &self.node_path,
                    ));
                }
            }
        }
    }
}
