use super::{Device, MotionDevice, NetDevice};

use glonax_core::motion::Motion;

const DEVICE_NAME: &str = "cau";
const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;
const DEVICE_NET_HCU_ADDR: u8 = 0x4a;

pub struct ControlAreaUnit {
    net: crate::net::ControlNet,
}

impl ControlAreaUnit {
    pub fn new(interface: &str) -> super::Result<Self> {
        let net = crate::net::ControlNet::open(interface, DEVICE_NET_LOCAL_ADDR);

        Ok(Self { net })
    }
}

#[async_trait::async_trait]
impl NetDevice for ControlAreaUnit {
    async fn from_interface(interface: &str) -> super::Result<Self> {
        Self::new(interface)
    }
}

impl Default for ControlAreaUnit {
    fn default() -> Self {
        Self::new("can0").unwrap()
    }
}

#[async_trait::async_trait]
impl Device for ControlAreaUnit {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl MotionDevice for ControlAreaUnit {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                trace!("Disable motion");

                self.net.set_motion_lock(DEVICE_NET_HCU_ADDR, true).await;
            }
            Motion::ResumeAll => {
                trace!("Enable motion");

                self.net.set_motion_lock(DEVICE_NET_HCU_ADDR, false).await;
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    let gate_bank = (actuator / 4) as usize;
                    let gate = actuator % 4;

                    trace!("Change actuator {} to value {}", actuator, value);

                    self.net
                        .gate_control(
                            DEVICE_NET_HCU_ADDR,
                            gate_bank,
                            [
                                if gate == 0 { Some(value) } else { None },
                                if gate == 1 { Some(value) } else { None },
                                if gate == 2 { Some(value) } else { None },
                                if gate == 3 { Some(value) } else { None },
                            ],
                        )
                        .await;
                }
            }
            _ => {}
        }
    }
}
