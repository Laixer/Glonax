use glonax_core::motion::Motion;

use crate::device::{self, Device, IoDeviceProfile, MotionDevice, UserDevice};

const DEVICE_NAME: &str = "can";
const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;
const DEVICE_NET_HCU_ADDR: u8 = 0x4a;

pub struct CanDeviceProfile {}

impl IoDeviceProfile for CanDeviceProfile {
    const CLASS: device::Subsystem = device::Subsystem::Net;

    #[inline]
    fn filter(device: &udev::Device) -> bool {
        device.sysname().to_str().unwrap().starts_with("can")
    }
}

pub struct Can {
    interface: String,
    net: crate::net::ControlNet,
}

#[async_trait::async_trait]
impl UserDevice for Can {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceRuleset = CanDeviceProfile;

    #[inline]
    fn sysname(&self) -> &str {
        self.interface.as_str()
    }

    #[inline]
    async fn from_sysname(name: &str) -> device::Result<Self> {
        Ok(Self::new(name).await)
    }
}

impl Can {
    async fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_owned(),
            net: crate::net::ControlNet::open(interface, DEVICE_NET_LOCAL_ADDR),
        }
    }
}

unsafe impl Send for Can {}

impl Device for Can {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl MotionDevice for Can {
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
