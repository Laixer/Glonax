use std::path::{Path, PathBuf};

use glonax_gamepad::{Axis, Button, Event, EventType};

use crate::{
    core::input::{ButtonState, Scancode},
    device::{self, Device, InputDevice, IoDeviceProfile, UserDevice},
};

const DEVICE_NAME: &str = "gamepad";

pub struct JoystickDeviceProfile {}

impl IoDeviceProfile for JoystickDeviceProfile {
    const CLASS: device::Subsystem = device::Subsystem::Input;

    fn properties() -> std::collections::HashMap<&'static str, &'static str> {
        let mut props = std::collections::HashMap::<&str, &str>::new();
        props.insert("ID_INPUT_JOYSTICK", "1");
        props
    }

    #[inline]
    fn filter(device: &udev::Device) -> bool {
        device.sysname().to_str().unwrap().starts_with("js")
    }
}

pub struct Gamepad {
    driver: glonax_gamepad::Gamepad,
    sysname: String,
    node_path: PathBuf,
    reverse_left: bool,
    reverse_right: bool,
}

#[async_trait::async_trait]
impl UserDevice for Gamepad {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceRuleset = JoystickDeviceProfile;

    #[inline]
    fn sysname(&self) -> &str {
        self.sysname.as_str()
    }

    #[inline]
    async fn from_sysname(_name: &str) -> device::Result<Self> {
        unimplemented!()
    }

    #[inline]
    async fn from_node_path(name: &str, path: &Path) -> device::Result<Self> {
        Ok(Self::new(name, path))
    }
}

impl Gamepad {
    fn new(name: &str, path: &Path) -> Self {
        Self {
            driver: glonax_gamepad::Gamepad::new(path).unwrap(),
            sysname: name.to_owned(),
            node_path: path.to_path_buf(),
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

impl InputDevice for Gamepad {
    fn next(&mut self) -> device::Result<Scancode> {
        loop {
            match self.driver.next_event() {
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
                        self.reverse_left = if event.value == 1 { true } else { false };
                    }
                    Event {
                        ty: EventType::Button(Button::RightBumper),
                        ..
                    } => {
                        self.reverse_right = if event.value == 1 { true } else { false };
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
                        break Ok(Scancode::Cancel(if value == 1 {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        }))
                    }
                    Event {
                        ty: EventType::Button(Button::South),
                        value,
                    } => {
                        break Ok(Scancode::Activate(if value == 1 {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        }))
                    }
                    Event {
                        ty: EventType::Button(Button::West),
                        value,
                    } => {
                        break Ok(Scancode::Restrict(if value == 1 {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        }))
                    }
                    _ => {}
                },
                Err(_) => {
                    break Err(device::DeviceError::no_such_device(
                        self.name(),
                        &self.node_path,
                    ))
                }
            }
        }
    }
}
