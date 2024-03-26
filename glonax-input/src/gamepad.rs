use std::path::{Path, PathBuf};

use glonax::driver;
use log::error;

use crate::{
    input::{ButtonState, Scancode},
    joystick::{Event, EventType, Joystick},
};

const DEVICE_NAME: &str = "gamepad";

// TODO: Maybe integrate the joystick driver into the gamepad driver.
pub(crate) struct Gamepad {
    driver: Joystick,
    node_path: PathBuf,
    reverse_left: bool,
    reverse_right: bool,
}

impl Gamepad {
    pub(crate) async fn new(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            driver: Joystick::open(path).await?,
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
                        ty: EventType::Axis(1),
                        ..
                    } => break Ok(Scancode::LeftStickY(event.value)),
                    Event {
                        ty: EventType::Axis(0),
                        ..
                    } => break Ok(Scancode::LeftStickX(event.value)),

                    Event {
                        ty: EventType::Axis(4),
                        ..
                    } => break Ok(Scancode::RightStickY(event.value)),
                    Event {
                        ty: EventType::Axis(3),
                        ..
                    } => break Ok(Scancode::RightStickX(event.value)),
                    Event {
                        ty: EventType::Button(4),
                        ..
                    } => {
                        self.reverse_left = event.value == 1;
                    }
                    Event {
                        ty: EventType::Button(5),
                        ..
                    } => {
                        self.reverse_right = event.value == 1;
                    }
                    Event {
                        ty: EventType::Axis(2),
                        ..
                    } => {
                        break Ok(Scancode::LeftTrigger(if self.reverse_left {
                            ((event.value as i32 - i16::MAX as i32) / 2) as i16
                        } else {
                            (((event.value as i32 - i16::MAX as i32) / 2) as i16).abs()
                        }))
                    }
                    Event {
                        ty: EventType::Axis(5),
                        ..
                    } => {
                        break Ok(Scancode::RightTrigger(if self.reverse_right {
                            ((event.value as i32 - i16::MAX as i32) / 2) as i16
                        } else {
                            (((event.value as i32 - i16::MAX as i32) / 2) as i16).abs()
                        }))
                    }
                    Event {
                        ty: EventType::Button(1),
                        value,
                        ..
                    } => break Ok(Scancode::Cancel(ButtonState::from(value))),
                    Event {
                        ty: EventType::Button(2),
                        value,
                        ..
                    } => break Ok(Scancode::DriveLock(ButtonState::from(value))),
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
