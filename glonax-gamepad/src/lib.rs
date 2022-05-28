use std::path::Path;

use tokio::{fs::File, io::AsyncReadExt};

/// Button pressed/released.
const JS_EVENT_TYPE_BUTTON: u8 = 0x1;
/// Joystick moved.
const JS_EVENT_TYPE_AXIS: u8 = 0x2;
/// Initial state of device.
const JS_EVENT_INIT: u8 = 0x80;

#[derive(Debug)]
pub enum Button {
    North,
    East,
    South,
    West,
    LeftBumper,
    RightBumper,
    LeftStick,
    RightStick,
    Select,
    Start,
    Guide,
    Other(u8),
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
        match value {
            v if v == 0 => Button::South,
            v if v == 1 => Button::East,
            v if v == 2 => Button::West,
            v if v == 3 => Button::North,
            v if v == 4 => Button::LeftBumper,
            v if v == 5 => Button::RightBumper,
            v if v == 6 => Button::Select,
            v if v == 7 => Button::Start,
            v if v == 8 => Button::Guide,
            v if v == 9 => Button::LeftStick,
            v if v == 10 => Button::RightStick,
            v => Button::Other(v),
        }
    }
}

#[derive(Debug)]
pub enum Axis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    DirectionalPadX,
    DirectionalPadY,
    LeftTrigger,
    RightTrigger,
    Other(u8),
}

impl From<u8> for Axis {
    fn from(value: u8) -> Self {
        match value {
            v if v == 0 => Axis::LeftStickX,
            v if v == 1 => Axis::LeftStickY,
            v if v == 2 => Axis::LeftTrigger,
            v if v == 3 => Axis::RightStickX,
            v if v == 4 => Axis::RightStickY,
            v if v == 5 => Axis::RightTrigger,
            v if v == 6 => Axis::DirectionalPadX,
            v if v == 7 => Axis::DirectionalPadY,
            v => Axis::Other(v),
        }
    }
}

#[derive(Debug)]
pub enum EventType {
    /// Button pressed/released.
    ButtonInit(Button),
    /// Button pressed/released.
    Button(Button),
    /// Axis moved.
    Axis(Axis),
    /// Axis moved.
    AxisInit(Axis),
}

#[derive(Debug)]
pub struct Event {
    /// Event type.
    pub ty: EventType,
    /// Corresponding value.
    pub value: i16,
}

impl Event {
    /// Return the value as normal.
    pub fn value_normal(&self) -> f32 {
        self.value as f32 * (1.0 / i16::MAX as f32)
    }

    pub fn value_flatten_normal(&self) -> f32 {
        let flat = (self.value as i32 - i16::MAX as i32).abs();
        flat as f32 * (1.0 / u16::MAX as f32)
    }
}

#[derive(Debug)]
#[repr(C)]
struct JsEvent {
    /// Event timestamp in milliseconds.
    time: u32,
    /// Value.
    value: i16,
    /// Event type.
    ty: u8,
    /// Axis/button number.
    number: u8,
}

pub struct Gamepad(tokio::io::BufReader<File>);

impl Gamepad {
    /// Construct new gamepad driver.
    pub async fn new(path: &Path) -> std::io::Result<Self> {
        Ok(Self(tokio::io::BufReader::with_capacity(
            4 * std::mem::size_of::<JsEvent>(),
            File::open(path).await?,
        )))
    }

    /// Return the next event from the gamepad.
    pub async fn next_event(&mut self) -> std::io::Result<Event> {
        let mut buf = [0; std::mem::size_of::<JsEvent>()];

        self.0.read_exact(&mut buf).await?;

        let event: JsEvent = unsafe { std::ptr::read(buf.as_ptr() as *const JsEvent) };

        if event.ty == JS_EVENT_TYPE_BUTTON {
            Ok(Event {
                ty: EventType::Button(event.number.into()),
                value: event.value,
            })
        } else if event.ty == JS_EVENT_TYPE_AXIS {
            Ok(Event {
                ty: EventType::Axis(event.number.into()),
                value: -event.value,
            })
        } else if event.ty == JS_EVENT_INIT | JS_EVENT_TYPE_BUTTON {
            Ok(Event {
                ty: EventType::ButtonInit(event.number.into()),
                value: event.value,
            })
        } else if event.ty == JS_EVENT_INIT | JS_EVENT_TYPE_AXIS {
            Ok(Event {
                ty: EventType::AxisInit(event.number.into()),
                value: -event.value,
            })
        } else {
            unimplemented!();
        }
    }
}
