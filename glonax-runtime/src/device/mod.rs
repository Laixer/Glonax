mod driver;
mod error;
mod gamepad;
mod gamepad2;
mod hydraulic;
mod inertial;

pub use gamepad::Gamepad;
use glonax_core::{input::Scancode, metric::MetricValue, motion::Motion};
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
    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    /// Run operation in idle time.
    ///
    /// The device can implement this method when it wants to
    /// run sporadic unscheduled events. There is no guarantee
    /// this method is ever called.
    fn idle_time(&mut self) {}
}

/// I/O device.
///
/// An I/O device takes a local resource such as a node or socket
/// as its communication medium.
pub trait IoDevice: Device + Sized {
    // TODO: type should be &std::path::Path
    /// Construct device from path resource.
    fn from_path(path: &String) -> Result<Self>;
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

/// Device which can read field metrics.
pub trait MetricDevice: Device {
    /// Return the next metric value and the device address from which the
    /// measurement originated. The device address may be used by the operand
    /// to map to a known machine component.
    fn next(&mut self) -> Option<(u16, MetricValue)>;
}
