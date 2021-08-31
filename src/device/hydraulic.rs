use crate::ice::Session;

use super::{Device, MotionDevice};

use serial::{SerialPort, SystemPort};

const DEVICE_NAME: &str = "hydraulic";
const DEVICE_ADDR: u16 = 0x7;

pub struct Hydraulic {
    session: Session<SystemPort>,
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
    fn actuate(&mut self, actuator: u32, value: i16) {
        if let Err(err) = self.session.dispatch_valve_control(actuator as u8, value) {
            error!("Session error: {:?}", err);
        }
    }

    fn halt(&mut self) {
        if let Err(err) = self.session.dispatch_valve_control(u8::MAX, 0) {
            error!("Session error: {:?}", err);
        }
    }
}

impl Drop for Hydraulic {
    fn drop(&mut self) {
        self.halt();
    }
}
