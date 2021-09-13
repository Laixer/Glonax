use std::path::Path;

use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug)]
pub(crate) enum Button {
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
pub(crate) enum Axis {
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
pub(crate) enum EventType {
    /// Button pressed/released.
    Button(Button),
    /// Axis moved.
    Axis(Axis),
}

#[derive(Debug)]
pub(crate) struct Event {
    /// Event type.
    pub(crate) ty: EventType,
    /// Corresponding value.
    pub(crate) value: i16,
}

impl Event {
    /// Return the value as normal.
    pub(crate) fn value_normal(&self) -> f32 {
        self.value as f32 * (1.0 / i16::MAX as f32)
    }
}

pub(crate) struct Gamepad {
    file: File,
}

impl Gamepad {
    /// Construct new gamepad driver.
    pub(crate) async fn new(path: &Path) -> Result<Self, std::io::Error> {
        Ok(Self {
            file: File::open(path).await?,
        })
    }

    // TODO: Retrieve multiple events in one read.
    /// Return the next event from the gamepad.
    pub(crate) async fn next_event(&mut self) -> Result<Event, ()> {
        const JS_EVENT_TYPE_BUTTON: u8 = 0x1;
        const JS_EVENT_TYPE_AXIS: u8 = 0x2;

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

        let mut buf = [0; std::mem::size_of::<JsEvent>()];
        self.file.read(&mut buf).await.unwrap();

        let event: JsEvent = unsafe { std::ptr::read(buf.as_ptr() as *const JsEvent) };
        if event.ty == JS_EVENT_TYPE_BUTTON {
            Ok(Event {
                ty: EventType::Button(event.number.into()),
                value: event.value,
            })
        } else if event.ty == JS_EVENT_TYPE_AXIS {
            Ok(Event {
                ty: EventType::Axis(event.number.into()),
                value: event.value * -1,
            })
        } else {
            Err(())
        }
    }
}
