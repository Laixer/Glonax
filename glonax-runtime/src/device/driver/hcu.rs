use std::sync::Arc;

use glonax_j1939::J1939Listener;

use crate::{
    core::motion::Motion,
    device::{Device, MotionDevice},
    net::ControlNet2,
};

const DEVICE_NAME: &str = "hcu";
const DEVICE_NET_HCU_ADDR: u8 = 0x4a;

pub struct Hcu(Arc<ControlNet2<J1939Listener>>);

unsafe impl Send for Hcu {}

impl Device for Hcu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Hcu {
    fn from_net(net: Arc<ControlNet2<J1939Listener>>) -> Self {
        Self(net)
    }

    async fn incoming(&mut self, _frame: &glonax_j1939::j1939::Frame) {
        //
    }
}

#[async_trait::async_trait]
impl MotionDevice for Hcu {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                trace!("Disable motion");

                self.0.set_motion_lock(DEVICE_NET_HCU_ADDR, true).await;
            }
            Motion::ResumeAll => {
                trace!("Enable motion");

                self.0.set_motion_lock(DEVICE_NET_HCU_ADDR, false).await;
            }
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    let gate_bank = (actuator / 4) as usize;
                    let gate = actuator % 4;

                    trace!("Stop actuator {}", actuator);

                    self.0
                        .gate_control(
                            DEVICE_NET_HCU_ADDR,
                            gate_bank,
                            [
                                if gate == 0 { Some(0) } else { None },
                                if gate == 1 { Some(0) } else { None },
                                if gate == 2 { Some(0) } else { None },
                                if gate == 3 { Some(0) } else { None },
                            ],
                        )
                        .await;
                }
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    let gate_bank = (actuator / 4) as usize;
                    let gate = actuator % 4;

                    trace!("Change actuator {} to value {}", actuator, value);

                    self.0
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
        }
    }
}
