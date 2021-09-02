use crate::{ice::Session, runtime::Motion};

use super::{Device, DeviceError, IoDevice, MotionDevice};

use serial::{SerialPort, SystemPort};

const DEVICE_NAME: &str = "hydraulic";
const DEVICE_ADDR: u16 = 0x7;

pub struct Hydraulic {
    session: Session<SystemPort>,
}

impl IoDevice for Hydraulic {
    fn from_path(path: &String) -> std::result::Result<Self, DeviceError> {
        Hydraulic::new(path)
    }
}

impl Hydraulic {
    pub fn new(path: impl ToString) -> std::result::Result<Self, super::DeviceError> {
        let mut channel = serial::open(&path.to_string()).map_err(|e: serial::Error| {
            super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
        })?;

        channel
            .reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud115200)?;
                settings.set_parity(serial::Parity::ParityNone);
                settings.set_stop_bits(serial::StopBits::Stop1);
                settings.set_flow_control(serial::FlowControl::FlowNone);
                Ok(())
            })
            .map_err(|e: serial::Error| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?;

        Ok(Self {
            session: Session::new(channel, DEVICE_ADDR),
        })
    }
}

impl Device for Hydraulic {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    fn probe(&mut self) {
        self.halt();
    }
}

impl MotionDevice for Hydraulic {
    fn actuate(&mut self, motion: Motion) {
        match motion {
            crate::runtime::Motion::StopAll => self.halt(),
            crate::runtime::Motion::Stop(actuators) => {
                for actuator in actuators {
                    debug!("Stop actuator {} ", actuator);

                    // FUTURE: Handle error, translate to device error?
                    if let Err(err) = self.session.dispatch_valve_control(actuator as u8, 0) {
                        error!("Session error: {:?}", err);
                    }
                }
            }
            crate::runtime::Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    debug!("Change actuator {} to value {}", actuator, value);

                    // FUTURE: Handle error, translate to device error?
                    if let Err(err) = self.session.dispatch_valve_control(actuator as u8, value) {
                        error!("Session error: {:?}", err);
                    }
                }
            }
        }
    }

    fn halt(&mut self) {
        debug!("Stop all actuators");

        // FUTURE: Handle error, translate to device error?
        if let Err(err) = self.session.dispatch_valve_control(u8::MAX, 0) {
            error!("Session error: {:?}", err);
        }
    }
}

impl Drop for Hydraulic {
    /// On drop try to stop any enduring motion.
    ///
    /// This is a best effort and there are no guarantees
    /// this has any effect.
    fn drop(&mut self) {
        self.halt();
    }
}
