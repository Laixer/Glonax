use std::path::{Path, PathBuf};

use super::{Device, IoDevice, MotionDevice};

use glonax_core::motion::Motion;
use glonax_ice::{eval::Evaluation, Session};
use glonax_serial::{BaudRate, FlowControl, Parity, StopBits, Uart};

const DEVICE_NAME: &str = "hydraulic";
const DEVICE_ADDR: u16 = 0x7;
// TODO: retrieve addr from session.
const REMOTE_DEVICE_ADDR: u16 = 0x7;

struct Debounce<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K, V> Debounce<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::cmp::PartialEq + Copy,
{
    /// Construct a new debouncer.
    fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    /// Push value on the key.
    ///
    /// If the current value is equal to the new value return true, otherwise
    /// return false.
    fn push(&mut self, key: K, value: V) -> bool {
        match self.map.insert(key, value) {
            Some(prev_value) => prev_value == value,
            None => false,
        }
    }
}

pub struct Hydraulic {
    session: Session<Uart>,
    node_path: PathBuf,
    debounce: Debounce<u32, i16>,
}

#[async_trait::async_trait]
impl IoDevice for Hydraulic {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceProfile = super::profile::SerialDeviceProfile;

    #[inline]
    fn node_path(&self) -> &Path {
        self.node_path.as_path()
    }

    #[inline]
    async fn from_node_path(path: &std::path::Path) -> super::Result<Self> {
        Hydraulic::new(path)
    }
}

impl Hydraulic {
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
            debounce: Debounce::new(),
        })
    }
}

#[async_trait::async_trait]
impl Device for Hydraulic {
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
impl MotionDevice for Hydraulic {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => {
                trace!("Stop all actuators");

                // FUTURE: Handle error, translate to device error?
                if let Err(err) = self.session.dispatch_valve_control(u8::MAX, 0).await {
                    error!("Session error: {:?}", err);
                }
                // TODO: HACK: XXX: Send exact same packet twice. This minimizes the chance one is never received.
                if let Err(err) = self.session.dispatch_valve_control(u8::MAX, 0).await {
                    error!("Session error: {:?}", err);
                }
            }
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    // Test the motion event against the debouncer. There is
                    // no point in sending the exact same motion value over and over again.
                    if !self.debounce.push(actuator, 0) {
                        trace!("Stop actuator {} ", actuator);

                        // FUTURE: Handle error, translate to device error?
                        if let Err(err) =
                            self.session.dispatch_valve_control(actuator as u8, 0).await
                        {
                            error!("Session error: {:?}", err);
                        }
                        // TODO: HACK: XXX: Send exact same packet twice. This minimizes the chance one is never received.
                        if let Err(err) =
                            self.session.dispatch_valve_control(actuator as u8, 0).await
                        {
                            error!("Session error: {:?}", err);
                        }
                    }
                }
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    // Test the motion event against the debouncer. There is
                    // no point in sending the exact same motion value over and over again.
                    if !self.debounce.push(actuator, value) {
                        trace!("Change actuator {} to value {}", actuator, value);

                        // FUTURE: Handle error, translate to device error?
                        if let Err(err) = self
                            .session
                            .dispatch_valve_control(actuator as u8, value)
                            .await
                        {
                            error!("Session error: {:?}", err);
                        }
                    }
                }
            }
        }

        // FUTURE: This must never happen after a value was dispatched. This action can take
        //         an undetermined amount of time.
        if let Err(err) = self.session.trigger_scheduler().await {
            error!("Session error: {:?}", err);
        };
    }

    async fn halt(&mut self) {
        self.actuate(Motion::StopAll).await;
    }
}
