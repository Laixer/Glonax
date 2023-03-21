use glonax_j1939::Frame;

use crate::net::{ActuatorService, J1939Network};

const DEVICE_NET_HCU_ADDR: u8 = 0x4a;

pub struct Hcu {
    service: ActuatorService,
}

impl Hcu {
    pub fn new(net: J1939Network) -> Self {
        Self {
            service: ActuatorService::new(net, DEVICE_NET_HCU_ADDR),
        }
    }
}

impl crate::net::Routable for Hcu {
    fn ingress(&mut self, frame: &Frame) -> bool {
        self.service.ingress(frame)
    }
}

impl Hcu {
    pub async fn actuate(&mut self, motion: crate::transport::Motion) {
        match motion.r#type() {
            crate::transport::motion::MotionType::None => panic!("NONE should not be used"),
            crate::transport::motion::MotionType::StopAll => {
                self.service.lock().await;
            }
            crate::transport::motion::MotionType::ResumeAll => {
                self.service.unlock().await;
            }
            crate::transport::motion::MotionType::Change => {
                self.service
                    .actuator_control(
                        motion
                            .changes
                            .into_iter()
                            .map(|changeset| (changeset.actuator as u8, changeset.value as i16))
                            .collect(),
                    )
                    .await;
            }
        }
    }
}
