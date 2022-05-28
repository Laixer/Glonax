use std::path::{Path, PathBuf};

use glonax_core::motion::Motion;

use crate::device::{self, Device, IoDevice, MotionDevice};

const DEVICE_NAME: &str = "sink";

pub struct Sink {
    node_path: PathBuf,
}

#[async_trait::async_trait]
impl IoDevice for Sink {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceProfile = device::profile::NullDeviceProfile;

    #[inline]
    fn node_path(&self) -> &Path {
        self.node_path.as_path()
    }

    #[inline]
    async fn from_node_path(path: &std::path::Path) -> device::Result<Self> {
        Sink::new(path)
    }
}

impl Sink {
    fn new(path: &std::path::Path) -> device::Result<Self> {
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
            Motion::ResumeAll => {
                trace!("Resume all actuators");
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
