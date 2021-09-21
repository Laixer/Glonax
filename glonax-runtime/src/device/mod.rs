mod error;
mod gamepad;
mod hydraulic;
mod inertial;

use std::path::Path;

pub use gamepad::Gamepad;
use glonax_core::{input::Scancode, metric::MetricValue, motion::Motion};
pub use hydraulic::Hydraulic;
pub use inertial::Inertial;

pub use error::{DeviceError, ErrorKind, Result};

pub type DeviceDescriptor<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

/// Device trait.
#[async_trait::async_trait]
pub trait Device: Send {
    // TODO: Maybe remove in future.
    /// Return the device name.
    fn name(&self) -> String;

    /// Probe the device.
    ///
    /// Can be used to signal that the device is ready.
    /// Implementation is optional.
    async fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    /// Run operation in idle time.
    ///
    /// The device can implement this method when it wants to
    /// run sporadic unscheduled events. There is no guarantee
    /// this method is ever called.
    async fn idle_time(&mut self) {}
}

/// I/O device.
///
/// An I/O device takes a local resource such as a node or socket
/// as its communication medium.
#[async_trait::async_trait]
pub trait IoDevice: Device + Sized {
    /// Device name.
    const NAME: &'static str;

    /// Construct device from path resource.
    async fn from_path(path: &Path) -> Result<Self>;
}

/// Device which can exercise motion.
#[async_trait::async_trait]
pub trait MotionDevice: Device {
    /// Issue actuate command.
    async fn actuate(&mut self, motion: Motion); // TODO: Return result.

    /// Halt all operation.
    ///
    /// Instruct all motion to stop. A device does not have to
    /// implement the halt method. This method should be called
    /// in rare occasions, for example in an emergency.
    async fn halt(&mut self) {} // TODO: Return result.
}

/// Device which can read input events.
#[async_trait::async_trait]
pub trait InputDevice: Device {
    async fn next(&mut self) -> Option<Scancode>;
}

/// Device which can read field metrics.
#[async_trait::async_trait]
pub trait MetricDevice: Device {
    /// Return the next metric value and the device address from which the
    /// measurement originated. The device address may be used by the operand
    /// to map to a known machine component.
    async fn next(&mut self) -> Option<(u16, MetricValue)>;
}

/// Create and initialize an IO device.
///
/// This function will return a shared handle to the device.
/// This is the recommended way to instantiate IO devices.
pub(crate) async fn probe_io_device<D: IoDevice + Send>(
    path: &Path,
) -> Result<DeviceDescriptor<D>> {
    // FUTURE: path.try_exists()
    // Every IO device must have an IO resource on disk. If that node does
    // not exist then exit right here. Doing this early on will ensure that
    // every IO device returns the same error if the IO resource was not found.
    // NOTE: We only check that the IO resource exist, but not if it is accessible.
    if !path.exists() {
        return Err(DeviceError::no_such_device(D::NAME.to_owned(), path));
    }

    let mut io_device = D::from_path(path).await?;

    debug!(
        "Probe I/O device '{}' from node {}",
        D::NAME.to_owned(),
        path.to_str().unwrap()
    );

    io_device.probe().await?;

    info!("Device '{}' is online", D::NAME.to_owned());

    Ok(std::sync::Arc::new(tokio::sync::Mutex::new(io_device)))
}
