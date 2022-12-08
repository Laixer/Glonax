use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

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

impl Device for Hcu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

impl Hcu {
    pub fn new(net: Arc<ControlNet>) -> Self {
        Self {
            service: ActuatorService::new(net, DEVICE_NET_HCU_ADDR),
        }
    }
}

#[async_trait::async_trait]
impl crate::net::Routable for Hcu {
    fn node(&self) -> u8 {
        DEVICE_NET_HCU_ADDR
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if self.service.ingress(pgn, frame) {
            true
        } else {
            false
        }
    }
}

#[async_trait::async_trait]
impl MotionDevice for Hcu {
    async fn actuate(&self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                self.service.lock().await;
            }
            Motion::ResumeAll => {
                self.service.unlock().await;
            }
            Motion::Stop(actuators) => {
                self.service
                    .actuator_control(actuators.into_iter().map(|k| (k as u8, 0)).collect())
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
