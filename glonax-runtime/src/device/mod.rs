use std::path::Path;

mod driver;
mod node;
mod observer;
mod profile;

mod manager;
pub use manager::DeviceManager;

mod cau;
pub use cau::ControlAreaUnit;

pub use driver::gamepad::Gamepad;
pub use driver::hydraulic::Hydraulic;
pub use driver::inertial::Inertial;
pub use driver::sink::Sink;

mod error;
pub use error::{DeviceError, ErrorKind, Result};

pub mod host;

use glonax_core::{input::Scancode, metric::MetricValue, motion::Motion};

pub type DeviceDescriptor<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

/// Device subsystems.
pub enum Subsystem {
    /// Input device class.
    Input,
    /// TTY device class.
    TTY,
    /// Virtual memory class.
    Memory,
}

/// Convert device subsystem to string.
impl From<Subsystem> for &str {
    fn from(value: Subsystem) -> Self {
        match value {
            Subsystem::Input => "input",
            Subsystem::TTY => "tty",
            Subsystem::Memory => "mem",
        }
    }
}

/// Device trait.
#[async_trait::async_trait]
pub trait Device: Send {
    // TODO: Maybe remove in future.
    /// Return the device name.
    fn name(&self) -> String;

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
}

/// I/O device.
///
/// An I/O device takes a local resource such as a node or socket
/// as its communication medium.
#[async_trait::async_trait]
pub trait IoDevice: Device + Sized {
    /// Device name.
    const NAME: &'static str;

    /// Get the node path from I/O device.
    fn node_path(&self) -> &Path;

    /// Hardware device profile.
    ///
    /// This profile will be used to match host device nodes.
    type DeviceProfile;

    /// Construct device from node path.
    async fn from_node_path(path: &Path) -> Result<Self>;
}

/// I/O device profile.
pub trait IoDeviceProfile {
    const CLASS: Subsystem;

    fn properties() -> std::collections::HashMap<&'static str, &'static str> {
        std::collections::HashMap::<&str, &str>::new()
    }

    fn filter(_device: &udev::Device) -> bool {
        true
    }
}

/// Network device.
#[async_trait::async_trait]
pub trait NetDevice: Device + Sized {
    async fn from_interface(interface: &str) -> Result<Self>;
}

/// Device which can exercise motion.
#[async_trait::async_trait]
pub trait MotionDevice: Device {
    /// Issue actuate command.
    async fn actuate(&mut self, motion: Motion); // TODO: Return result.
}

/// Device which can read input events.
#[async_trait::async_trait]
pub trait InputDevice: Device {
    async fn next(&mut self) -> Result<Scancode>;
}

/// Device which can read field metrics.
#[async_trait::async_trait]
pub trait MetricDevice: Device {
    /// Return the next metric value and the device address from which the
    /// measurement originated. The device address may be used by the operand
    /// to map to a known machine component.
    async fn next(&mut self) -> Option<(u16, MetricValue)>;
}
