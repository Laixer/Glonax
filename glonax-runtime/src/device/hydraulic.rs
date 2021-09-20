use super::{Device, IoDevice, MotionDevice};

use glonax_core::motion::Motion;
use glonax_ice::{eval::Evaluation, Session};
use glonax_serial::{BaudRate, FlowControl, Parity, StopBits, Uart};

const DEVICE_NAME: &str = "hydraulic";
const DEVICE_ADDR: u16 = 0x7;

struct Cache<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::cmp::PartialEq + Copy,
{
    fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    /// Check if value was found in cache.
    fn hit(&mut self, key: K, value: V) -> bool {
        match self.map.insert(key, value) {
            Some(prev_value) => prev_value == value,
            None => false,
        }
    }
}

pub struct Hydraulic {
    session: Session<Uart>,
    cache: Cache<u32, i16>,
}

impl IoDevice for Hydraulic {
    fn from_path(path: &String) -> super::Result<Self> {
        Hydraulic::new(path)
    }
}

impl Hydraulic {
    pub fn new(path: impl ToString) -> super::Result<Self> {
        let port = glonax_serial::builder(&std::path::Path::new(&path.to_string()))
            .map_err(|e| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?
            .set_baud_rate(BaudRate::Baud115200)
            .map_err(|e| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?
            .set_parity(Parity::ParityNone)
            .set_stop_bits(StopBits::Stop1)
            .set_flow_control(FlowControl::FlowNone)
            .build()
            .map_err(|e| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?;

        Ok(Self {
            session: Session::new(port, DEVICE_ADDR),
            cache: Cache::new(),
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

        eval.probe_test()
            .await
            .map_err(|e| super::DeviceError::from_session(DEVICE_NAME.to_owned(), e))?;

        self.halt().await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl MotionDevice for Hydraulic {
    async fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => self.halt().await,
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    // Test the motion event against the cache. There is
                    // no point in sending the exact same motion value over and over again.
                    if !self.cache.hit(actuator, 0) {
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
                    // Test the motion event against the cache. There is
                    // no point in sending the exact same motion value over and over again.
                    if !self.cache.hit(actuator, value) {
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
    }

    async fn halt(&mut self) {
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
}
