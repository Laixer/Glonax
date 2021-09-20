use std::convert::TryInto;

use super::{Device, IoDevice, MetricDevice, MetricValue};

use glonax_ice::{eval::Evaluation, PayloadType, Session, Vector3x16};
use glonax_serial::{BaudRate, FlowControl, Parity, StopBits, Uart};

const DEVICE_NAME: &str = "imu";
const DEVICE_ADDR: u16 = 0x7;

pub struct Inertial {
    session: Session<Uart>,
}

#[async_trait::async_trait]
impl IoDevice for Inertial {
    async fn from_path(path: &std::path::Path) -> super::Result<Self> {
        Inertial::new(path)
    }
}

impl Inertial {
    pub fn new(path: &std::path::Path) -> super::Result<Self> {
        let port = glonax_serial::builder(path)
            .map_err(|e| super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?
            .set_baud_rate(BaudRate::Baud115200)
            .map_err(|e| super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?
            .set_parity(Parity::ParityNone)
            .set_stop_bits(StopBits::Stop1)
            .set_flow_control(FlowControl::FlowNone)
            .build()
            .map_err(|e| super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?;

        Ok(Self {
            session: Session::new(port, DEVICE_ADDR),
        })
    }
}

#[async_trait::async_trait]
impl Device for Inertial {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    async fn probe(&mut self) -> super::Result<()> {
        let mut eval = Evaluation::new(&mut self.session);

        eval.probe_test()
            .await
            .map_err(|e| super::DeviceError::from_session(DEVICE_NAME.to_owned(), e))?;

        Ok(())
    }
}

// TODO: retrieve addr from session.
const REMOTE_DEVICE_ADDR: u16 = 9;

#[async_trait::async_trait]
impl MetricDevice for Inertial {
    async fn next(&mut self) -> Option<(u16, MetricValue)> {
        match self.session.accept().await {
            Ok(frame) => match frame.packet().payload_type.try_into().unwrap() {
                PayloadType::MeasurementAcceleration => {
                    let acc: Vector3x16 = frame.get(6).unwrap();
                    Some((
                        REMOTE_DEVICE_ADDR,
                        MetricValue::Acceleration((acc.x, acc.y, acc.z).into()),
                    ))
                }
                _ => None,
            },
            Err(e) => {
                warn!("Session fault: {:?}", e);
                None
            }
        }
    }
}
