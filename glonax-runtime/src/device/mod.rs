mod driver;

pub use driver::hcu::Hcu;
pub use driver::mecu::Mecu;
pub use driver::vecu::Vecu;

mod error;
pub use error::{DeviceError, ErrorKind, Result};

use crate::core::{input::Scancode, motion::Motion};

/// Device which can exercise motion.
#[async_trait::async_trait]
pub trait MotionDevice {
    /// Issue actuate command.
    async fn actuate(&mut self, motion: Motion); // TODO: Return result.
}

/// Device which can read input events.
#[async_trait::async_trait]
pub trait InputDevice {
    async fn next(&mut self) -> Result<Scancode>;
}

#[async_trait::async_trait]
pub trait CoreDevice {
    async fn next(&mut self) -> Result<()>;
}

/// Device which can read field metrics.
#[async_trait::async_trait]
pub trait MetricDevice {
    /// Return the next metric value and the device address from which the
    /// measurement originated. The device address may be used by the operand
    /// to map to a known machine component.
    async fn next(&mut self);
}
