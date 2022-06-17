use std::path::Path;

mod applicant;
mod driver;
mod profile;

mod manager;
pub use manager::DeviceManager;

pub use driver::gamepad::Gamepad;
pub use driver::gateway::*;
pub use driver::hcu::Hcu;
pub use driver::hydraulic::Hydraulic;
pub use driver::inertial::Inertial;
pub use driver::mecu::Mecu;
pub use driver::sink::Sink;
pub use driver::vecu::Vecu;

mod error;
pub use error::{DeviceError, ErrorKind, Result};

pub mod host;

use crate::core::{input::Scancode, motion::Motion};

pub type DeviceDescriptor<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

/// Device subsystems.
pub enum Subsystem {
    /// Input device class.
    Input,
    /// TTY device class.
    TTY,
    /// Virtual memory class.
    Memory,
    /// Network device class.
    Net,
}

/// Convert device subsystem to string.
impl From<Subsystem> for &str {
    fn from(value: Subsystem) -> Self {
        match value {
            Subsystem::Input => "input",
            Subsystem::TTY => "tty",
            Subsystem::Memory => "mem",
            Subsystem::Net => "net",
        }
    }
}

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

/// udev hotplug device.
///
/// An I/O device takes a local resource such as a node or socket
/// as its communication medium.
#[async_trait::async_trait]
pub trait UserDevice: Device + Sized {
    /// Device name.
    const NAME: &'static str;

    /// Get the system name.
    fn sysname(&self) -> &str;

    /// Device match ruleset.
    ///
    /// This ruleset will be used to match host device candidates.
    type DeviceRuleset;

    /// Construct device from system name.
    async fn from_sysname(name: &str) -> Result<Self>;

    /// Construct device from node path.
    async fn from_node_path(_name: &str, _path: &Path) -> Result<Self> {
        unimplemented!()
    }
}

// TODO: Renam to UserDeviceRuleset
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
