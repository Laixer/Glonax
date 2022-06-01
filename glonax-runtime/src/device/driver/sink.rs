use glonax_core::motion::Motion;

use crate::device::{Device, MotionDevice};

const DEVICE_NAME: &str = "sink";

pub struct Sink {}

impl Sink {
    pub fn new() -> Self {
        Self {}
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
