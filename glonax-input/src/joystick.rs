use std::path::Path;

use tokio::io::{AsyncReadExt, BufReader};

/// Button pressed/released.
const JS_EVENT_TYPE_BUTTON: u8 = 0x1;
/// Joystick moved.
const JS_EVENT_TYPE_AXIS: u8 = 0x2;
/// Initial state of device.
const JS_EVENT_INIT: u8 = 0x80;

#[allow(dead_code)]
#[derive(Debug)]
pub enum EventType {
    /// Button pressed/released.
    ButtonInit(u8),
    /// Button pressed/released.
    Button(u8),
    /// Axis moved.
    Axis(u8),
    /// Axis moved.
    AxisInit(u8),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Event {
    /// Event timestamp in milliseconds.
    pub time: u32,
    /// Event type.
    pub ty: EventType,
    /// Corresponding value.
    pub value: i16,
}

impl From<&[u8]> for Event {
    fn from(buffer: &[u8]) -> Self {
        let event: JsEvent = unsafe { std::ptr::read(buffer.as_ptr() as *const JsEvent) };

        if event.ty == JS_EVENT_TYPE_BUTTON {
            Event {
                time: event.time,
                ty: EventType::Button(event.number),
                value: event.value,
            }
        } else if event.ty == JS_EVENT_TYPE_AXIS {
            Event {
                time: event.time,
                ty: EventType::Axis(event.number),
                value: -event.value,
            }
        } else if event.ty == JS_EVENT_INIT | JS_EVENT_TYPE_BUTTON {
            Event {
                time: event.time,
                ty: EventType::ButtonInit(event.number),
                value: event.value,
            }
        } else if event.ty == JS_EVENT_INIT | JS_EVENT_TYPE_AXIS {
            Event {
                time: event.time,
                ty: EventType::AxisInit(event.number),
                value: -event.value,
            }
        } else {
            unimplemented!();
        }
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

pub struct Joystick(BufReader<tokio::fs::File>);

impl Joystick {
    /// Construct new gamepad driver.
    pub async fn open(path: &Path) -> std::io::Result<Self> {
        Ok(Self(BufReader::with_capacity(
            16 * std::mem::size_of::<JsEvent>(),
            tokio::fs::File::open(path).await?,
        )))
    }

    /// Return the next event from the gamepad.
    pub async fn next_event(&mut self) -> std::io::Result<Event> {
        let mut buf = [0; std::mem::size_of::<JsEvent>()];

        self.0.read_exact(&mut buf).await?;

        Ok(Event::from(&buf[..]))
    }
}
