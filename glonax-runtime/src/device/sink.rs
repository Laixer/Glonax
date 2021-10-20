use std::path::{Path, PathBuf};

use super::{Device, IoDevice, MotionDevice};

use glonax_core::motion::Motion;

const DEVICE_NAME: &str = "sink";

pub struct SinkDeviceProfile {}

impl super::IoDeviceProfile for SinkDeviceProfile {
    const CLASS: super::Subsystem = super::Subsystem::Memory;

    #[inline]
    fn filter(device: &udev::Device) -> bool {
        device.sysname().to_str().unwrap() == "null"
    }
}

pub struct Sink {
    node_path: PathBuf,
}

#[async_trait::async_trait]
impl IoDevice for Sink {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceProfile = SinkDeviceProfile;

    #[inline]
    fn node_path(&self) -> &Path {
        self.node_path.as_path()
    }

    #[inline]
    async fn from_node_path(path: &std::path::Path) -> super::Result<Self> {
        Sink::new(path)
    }
}

impl Sink {
    fn new(path: &std::path::Path) -> super::Result<Self> {
        Ok(Self {
            node_path: path.to_path_buf(),
        })
    }
}

impl Device for Sink {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl MotionDevice for Sink {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                trace!("Stop all actuators");
            }
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    trace!("Stop actuator {} ", actuator);
                }
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    trace!("Change actuator {} to value {}", actuator, value);
                }
            }
        }
    }
}
