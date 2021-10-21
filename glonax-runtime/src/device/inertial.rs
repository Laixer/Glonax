use std::{
    convert::TryInto,
    path::{Path, PathBuf},
};

use super::{Device, IoDevice, MetricDevice, MetricValue};

use glonax_ice::{eval::Evaluation, PayloadType, Session, Vector3x16};
use glonax_serial::{BaudRate, FlowControl, Parity, StopBits, Uart};

const DEVICE_NAME: &str = "imu";
const DEVICE_ADDR: u16 = 0x7;
// TODO: retrieve addr from session.
const REMOTE_DEVICE_ADDR: u16 = 0x9;

pub struct Inertial {
    session: Session<Uart>,
    node_path: PathBuf,
}

#[async_trait::async_trait]
impl IoDevice for Inertial {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceProfile = super::profile::SerialDeviceProfile;

    #[inline]
    fn node_path(&self) -> &Path {
        self.node_path.as_path()
    }

    async fn from_node_path(path: &std::path::Path) -> super::Result<Self> {
        Inertial::new(path)
    }
}

impl Inertial {
    fn new(path: &std::path::Path) -> super::Result<Self> {
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
            node_path: path.to_path_buf(),
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

        let scan = eval
            .network_scan()
            .await
            .map_err(|e| super::DeviceError::from_session(DEVICE_NAME.to_owned(), e))?;

        trace!("Network scan result: {:?}", scan);

        if scan.address != REMOTE_DEVICE_ADDR {
            return Err(crate::device::DeviceError {
                device: DEVICE_NAME.to_owned(),
                kind: crate::device::ErrorKind::InvalidDeviceFunction,
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl MetricDevice for Inertial {
    async fn next(&mut self) -> Option<(u16, MetricValue)> {
        match self.session.accept().await {
            Ok(frame) => match frame.packet().payload_type.try_into().unwrap() {
                PayloadType::MeasurementAcceleration => {
                    let acc: Vector3x16 = frame.get(6).unwrap();
                    Some((
                        REMOTE_DEVICE_ADDR,
                        MetricValue::Acceleration(glonax_core::nalgebra::Vector3::new(
                            acc.x as f32,
                            acc.y as f32,
                            acc.z as f32,
                        )),
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
