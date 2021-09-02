mod compose;
mod error;
mod gamepad;
mod hydraulic;
mod inertial;

use crate::{
    common::position::Position,
    runtime::{Motion, Scancode},
};

pub use compose::Composer;
pub use gamepad::Gamepad;
pub use hydraulic::Hydraulic;
pub use inertial::Inertial;

pub use error::{DeviceError, ErrorKind, Result};

/// Device trait.
pub trait Device {
    /// Return the device name.
    fn name(&self) -> String;

    /// Probe the device.
    ///
    /// Can be used to signal that the device is ready.
    /// Implementation is optional.
    fn probe(&mut self) {} // TODO: Return result.
}

/// Device which can exercise motion.
pub trait MotionDevice: Device {
    /// Issue actuate command.
    fn actuate(&mut self, motion: Motion); // TODO: Return result.

    /// Halt all operation.
    ///
    /// Instruct all motion to stop. A device does not have to
    /// implement the halt method. This method should be called
    /// in rare occasions, for example in an emergency.
    fn halt(&mut self) {} // TODO: Return result.
}

/// Device which can read commands.
pub trait CommandDevice: Device {
    fn next(&mut self) -> Option<Scancode>;
}

#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Temperature(i16),
    Position(Position),
}

/// Device which can read field metrics.
pub trait MetricDevice: Device {
    fn next(&mut self) -> Option<MetricValue>;
}
