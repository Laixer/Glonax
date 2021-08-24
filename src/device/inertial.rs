use crate::{
    common::position::Position,
    gloproto::{Session, Sugar},
};

use super::{Device, MetricDevice, MetricValue};

use serial::{SerialPort, SystemPort};

const DEVICE_NAME: &str = "imu";

pub struct Inertial {
    session: Session<SystemPort>,
}

impl Inertial {
    pub fn new(path: impl ToString) -> std::result::Result<Self, super::DeviceError> {
        let mut channel = serial::open(&path.to_string()).map_err(|e: serial::Error| {
            super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
        })?;

        channel
            .reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud9600)?;
                settings.set_parity(serial::Parity::ParityNone);
                settings.set_stop_bits(serial::StopBits::Stop1);
                settings.set_flow_control(serial::FlowControl::FlowNone);
                Ok(())
            })
            .map_err(|e: serial::Error| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?;

        Ok(Self {
            session: Session::new(channel),
        })
    }
}

impl Device for Inertial {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    fn probe(&mut self) {
        self.session.wait_for_init();
    }
}

impl MetricDevice for Inertial {
    // TODO: Code can be improved.
    fn next(&mut self) -> Option<MetricValue> {
        if let Some(packet) = self.session.next() {
            match packet {
                Sugar::Temperature(temp) => Some(MetricValue::Temperature(temp)),
                Sugar::Acceleration(x, y, z) => {
                    Some(MetricValue::Position(Position::from_raw(x, y, z)))
                }
                Sugar::Orientation(_x, _y, _z) => {
                    // debug!("Arm Raw Orientation: X {} Y {} Z {}", x, y, z);
                    None
                }
                Sugar::Direction(_, _, _) => None,
                _ => None,
            }
        } else {
            None
        }
    }
}
