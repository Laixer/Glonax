use std::{
    convert::TryInto,
    path::{Path, PathBuf},
};

use glonax_ice::{eval::Evaluation, PayloadType, Session, Vector3x16};
use glonax_serial::{BaudRate, FlowControl, Parity, StopBits, Uart};

use crate::device::{self, Device, MetricDevice, MetricValue, UserDevice};

const DEVICE_NAME: &str = "imu";
const DEVICE_ADDR: u16 = 0x7;
// TODO: retrieve addr from session.
const REMOTE_DEVICE_ADDR: u16 = 0x9;

pub struct Inertial {
    session: Session<Uart>,
    sysname: String,
    node_path: PathBuf,
}

#[async_trait::async_trait]
impl UserDevice for Inertial {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceRuleset = device::profile::SerialDeviceProfile;

    #[inline]
    fn sysname(&self) -> &str {
        self.sysname.as_str()
    }

    #[inline]
    async fn from_sysname(_name: &str) -> device::Result<Self> {
        unimplemented!()
    }

    #[inline]
    async fn from_node_path(name: &str, path: &Path) -> device::Result<Self> {
        Self::new(name, path)
    }
}

impl Inertial {
    fn new(name: &str, path: &std::path::Path) -> device::Result<Self> {
        let port = glonax_serial::builder(path)
            .map_err(|e| device::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?
            .set_baud_rate(BaudRate::Baud115200)
            .map_err(|e| device::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?
            .set_parity(Parity::ParityNone)
            .set_stop_bits(StopBits::Stop1)
            .set_flow_control(FlowControl::FlowNone)
            .build()
            .map_err(|e| device::DeviceError::from_serial(DEVICE_NAME.to_owned(), path, e))?;

        Ok(Self {
            session: Session::new(port, DEVICE_ADDR),
            sysname: name.to_owned(),
            node_path: path.to_path_buf(),
        })
    }
}

#[async_trait::async_trait]
impl Device for Inertial {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    async fn probe(&mut self) -> device::Result<()> {
        let mut eval = Evaluation::new(&mut self.session);

        let scan = eval
            .network_scan()
            .await
            .map_err(|e| device::DeviceError::from_session(DEVICE_NAME.to_owned(), e))?;

        trace!("Network scan result: {:?}", scan);

        if scan.address != REMOTE_DEVICE_ADDR {
            return Err(device::DeviceError {
                device: DEVICE_NAME.to_owned(),
                kind: device::ErrorKind::InvalidDeviceFunction,
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
