use std::convert::TryInto;

use super::{Device, MetricDevice, MetricValue};

use glonax_ice::{PayloadType, Session, Vector3x16};
use serial::{SerialPort, SystemPort};

const DEVICE_NAME: &str = "imu";
const DEVICE_ADDR: u16 = 0x7;

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
            session: Session::new(channel, DEVICE_ADDR),
        })
    }
}

impl Device for Inertial {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    fn probe(&mut self) {
        // self.session.wait_for_init();
    }
}

impl MetricDevice for Inertial {
    fn next(&mut self) -> Option<MetricValue> {
        let frame = self.session.accept();
        match frame.packet().payload_type.try_into().unwrap() {
            PayloadType::MeasurementAcceleration => {
                let acc: Vector3x16 = frame.get(6).unwrap();
                let acc_x = acc.x;
                let acc_y = acc.y;
                let acc_z = acc.z;
                debug!(
                    "Acceleration: X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    acc_x, acc_y, acc_z
                );
                None
            }
            _ => None,
        }

        // Sugar::Acceleration(x, y, z) => {
        //  Some(MetricValue::Position(Position::from_raw(x, y, z)))
        // }
        // Sugar::Orientation(_x, _y, _z) => {
        //  // debug!("Arm Raw Orientation: X {} Y {} Z {}", x, y, z);
        //  None
        // }
        // Sugar::Direction(_, _, _) => None,
    }
}
