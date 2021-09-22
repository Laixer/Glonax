use std::path::Path;

mod claim;
mod serial_profile;

mod manager;
pub use manager::DeviceManager;

mod gamepad;
pub use gamepad::Gamepad;

mod hydraulic;
pub use hydraulic::Hydraulic;

mod inertial;
pub use inertial::Inertial;

mod error;
pub use error::{DeviceError, ErrorKind, Result};

pub mod host;

use glonax_core::{input::Scancode, metric::MetricValue, motion::Motion};

use self::host::HostInterface;

pub type DeviceDescriptor<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

/// Device subsystems.
pub enum Subsystem {
    /// Input device class.
    Input,
    /// TTY device class.
    TTY,
}

/// Convert device subsystem to string.
impl From<Subsystem> for &str {
    fn from(value: Subsystem) -> Self {
        match value {
            Subsystem::Input => "input",
            Subsystem::TTY => "tty",
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

    /// Hardware device profile.
    ///
    /// This profile will be used to match host device nodes.
    type DeviceProfile;

    /// Construct device from path resource.
    async fn from_path(path: &Path) -> Result<Self>;
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
pub(crate) async fn probe_io_device<D: IoDevice>(path: &Path) -> Result<DeviceDescriptor<D>> {
    // FUTURE: path.try_exists()
    // Every IO device must have an IO resource on disk. If that node does
    // not exist then exit right here. Doing this early on will ensure that
    // every IO device returns the same error if the IO resource was not found.
    //
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

    Ok(std::sync::Arc::new(tokio::sync::Mutex::new(io_device)))
}

/// Create, initialize and claim an IO device.
///
/// This function will return a shared handle to the device.
/// This is the recommended way to instantiate and claim IO devices.
pub(crate) async fn probe_claim_io_device<D: IoDevice>(
    claim: &mut claim::ResourceClaim,
) -> Result<DeviceDescriptor<D>> {
    let device = self::probe_io_device(claim.as_path()).await?;

    claim.claim();

    info!("Device '{}' is claimed", D::NAME.to_owned());

    Ok(device)
}

/// Discover devices of the device type.
///
/// Returns a list of all claimed devices.
pub(crate) async fn discover_devices<D>(manager: &mut DeviceManager) -> Vec<DeviceDescriptor<D>>
where
    D: IoDevice + 'static,
    D::DeviceProfile: IoDeviceProfile,
{
    let mut host_iface = HostInterface::new();
    let mut device_candidate = host_iface.elect::<D::DeviceProfile>();

    let mut claimed = Vec::new();

    loop {
        let mut device_claim = match device_candidate.next() {
            Some(device_claim) => device_claim,
            None => break,
        };

        trace!(
            "Elected claim: {}",
            device_claim.as_path().to_str().unwrap()
        );

        match probe_claim_io_device::<D>(&mut device_claim).await {
            Ok(device) => {
                if device_claim.is_claimed() {
                    claimed.push(device.clone());
                    manager.register_device(device.clone());
                }
            }
            Err(DeviceError {
                kind: ErrorKind::InvalidDeviceFunction,
                ..
            }) => continue,
            Err(e) => {
                warn!("{:?}", e);
                continue;
            }
        }
    }

    claimed
}
