mod driver;

pub use driver::gamepad::Gamepad;
pub use driver::gateway::*;
pub use driver::hcu::Hcu;
pub use driver::mecu::Mecu;
pub use driver::sink::Sink;
pub use driver::vecu::Vecu;

mod error;
pub use error::{DeviceError, ErrorKind, Result};

use crate::core::{input::Scancode, motion::Motion};

/// Device trait.
#[async_trait::async_trait]
pub trait Device: Send {
    // TODO: Maybe remove in future.
    /// Return the device name.
    fn name(&self) -> String;

    // TODO: Move into Udev
    /// Probe the device.
    ///
    /// Can be used to signal that the device is ready. If the probe returns
    /// with success then we can assume the device is connected. Any pre-use
    /// checks should be done now.
    ///
    /// Implementation is optional.
    async fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    /// Returns the current status of the device, or any complications.
    ///
    /// Implementation is optional but recommended.
    async fn status(&mut self) -> Result<()> {
        Ok(())
    }

    // TODO: add state change fn
}

/// Device which can exercise motion.
#[async_trait::async_trait]
pub trait MotionDevice: Device {
    /// Issue actuate command.
    async fn actuate(&mut self, motion: Motion); // TODO: Return result.
}

/// Device which can read input events.
pub trait InputDevice: Device {
    fn next(&mut self) -> Result<Scancode>;
}

#[async_trait::async_trait]
pub trait CoreDevice: Device {
    async fn next(&mut self) -> Result<()>;
}

/// Device which can read field metrics.
#[async_trait::async_trait]
pub trait MetricDevice: Device {
    /// Return the next metric value and the device address from which the
    /// measurement originated. The device address may be used by the operand
    /// to map to a known machine component.
    async fn next(&mut self);
}
