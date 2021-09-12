use std::convert::TryInto;

use super::{Device, DeviceError, IoDevice, MetricDevice, MetricValue};

use glonax_ice::{PayloadType, Session, Vector3x16};
use serial::{SerialPort, SystemPort};

const DEVICE_NAME: &str = "imu";
const DEVICE_ADDR: u16 = 0x7;

pub struct Inertial {
    session: Session<SystemPort>,
}

impl IoDevice for Inertial {
    fn from_path(path: &String) -> std::result::Result<Self, DeviceError> {
        Inertial::new(path)
    }
}

impl Inertial {
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

impl Device for Inertial {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    fn probe(&mut self) {
        // TODO: We shoud read the actuat packet.
        // TODO: Remove the logline and report via Result
        if self.session.accept().is_ok() {
            info!("{} in online", self.name());
        }
    }
}

impl MetricDevice for Inertial {
    fn next(&mut self) -> Option<MetricValue> {
        let frame = self.session.accept().unwrap(); // TODO: handle err
        match frame.packet().payload_type.try_into().unwrap() {
            PayloadType::MeasurementAcceleration => {
                let acc: Vector3x16 = frame.get(6).unwrap();
                Some(MetricValue::Acceleration((acc.x, acc.y, acc.z).into()))
            }
            _ => None,
        }
    }
}
