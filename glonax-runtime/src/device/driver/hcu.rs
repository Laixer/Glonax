use std::sync::Arc;

use glonax_j1939::Frame;

use crate::{
    core::motion::Motion,
    device::{Device, MotionDevice},
    net::{ActuatorService, ControlNet},
};

const DEVICE_NAME: &str = "hcu";
const DEVICE_NET_HCU_ADDR: u8 = 0x4a;

pub struct Hcu {
    service: ActuatorService,
}

unsafe impl Send for Hcu {}

impl Device for Hcu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Hcu {
    fn from_net(net: Arc<ControlNet>) -> Self {
        Self {
            service: ActuatorService::new(net, DEVICE_NET_HCU_ADDR),
        }
    }

    async fn incoming(&mut self, _frame: &Frame) {
        // TODO: Need an external trigger.
        self.service.interval().await;
    }
}

#[async_trait::async_trait]
impl MotionDevice for Hcu {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                self.service.lock().await;
            }
            Motion::ResumeAll => {
                self.service.unlock().await;
            }
            Motion::Stop(actuators) => {
                self.service
                    .actuator_stop(actuators.into_iter().map(|k| k as u8).collect())
                    .await;
            }
            Motion::Change(actuators) => {
                self.service
                    .actuator_control(actuators.into_iter().map(|(k, v)| (k as u8, v)).collect())
                    .await;
            }
        }
    }
}
