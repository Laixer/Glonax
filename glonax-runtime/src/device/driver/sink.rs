use std::path::Path;

use glonax_core::motion::Motion;

use crate::{
    device::{self, Device, MotionDevice, UserDevice},
    Config,
};

const DEVICE_NAME: &str = "sink";

pub struct Sink {
    sysname: String,
}

#[async_trait::async_trait]
impl UserDevice for Sink {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceRuleset = device::profile::NullDeviceProfile;

    #[inline]
    fn sysname(&self) -> &str {
        self.sysname.as_str()
    }

    #[inline]
    async fn from_sysname(_name: &str, _config: &Config) -> device::Result<Self> {
        unimplemented!()
    }

    #[inline]
    async fn from_node_path(name: &str, _config: &Config, path: &Path) -> device::Result<Self> {
        Ok(Self::new(name, path))
    }
}

impl Sink {
    fn new(name: &str, _path: &Path) -> Self {
        Self {
            sysname: name.to_owned(),
        }
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
                trace!("Stop all pseudo actuators");
            }
            Motion::ResumeAll => {
                trace!("Resume all pseudo actuators");
            }
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    trace!("Stop pseudo actuator {} ", actuator);
                }
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    trace!("Change pseudo actuator {} to value {}", actuator, value);
                }
            }
        }
    }
}
