use super::{Device, IoDevice, MotionDevice};

use glonax_core::motion::Motion;
use glonax_ice::Session;
use serial::{SerialPort, SystemPort};

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
    session: Session<SystemPort>,
    cache: Cache<u32, i16>,
}

impl IoDevice for Hydraulic {
    fn from_path(path: &String) -> super::Result<Self> {
        Hydraulic::new(path)
    }
}

impl Hydraulic {
    pub fn new(path: impl ToString) -> super::Result<Self> {
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

        channel
            .set_timeout(std::time::Duration::from_millis(500))
            .map_err(|e: serial::Error| {
                super::DeviceError::from_serial(DEVICE_NAME.to_owned(), path.to_string(), e)
            })?;

        Ok(Self {
            session: Session::new(channel, DEVICE_ADDR),
            cache: Cache::new(),
        })
    }
}

impl Device for Hydraulic {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }

    fn probe(&mut self) -> super::Result<()> {
        // TODO: We shoud read the actuat packet.
        self.session
            .accept()
            .map_err(|e| super::DeviceError::from_session(DEVICE_NAME.to_owned(), e))?;

        self.halt();

        Ok(())
    }
}

impl MotionDevice for Hydraulic {
    fn actuate(&mut self, motion: Motion) {
        match motion {
            Motion::StopAll => self.halt(),
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    // Test the motion event against the cache. There is
                    // no point in sending the exact same motion value over and over again.
                    if !self.cache.hit(actuator, 0) {
                        debug!("Stop actuator {} ", actuator);

                        // FUTURE: Handle error, translate to device error?
                        if let Err(err) = self.session.dispatch_valve_control(actuator as u8, 0) {
                            error!("Session error: {:?}", err);
                        }
                        // TODO: HACK: XXX: Send exact same packet twice. This minimizes the chance one is never received.
                        if let Err(err) = self.session.dispatch_valve_control(actuator as u8, 0) {
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
                        debug!("Change actuator {} to value {}", actuator, value);

                        // FUTURE: Handle error, translate to device error?
                        if let Err(err) = self.session.dispatch_valve_control(actuator as u8, value)
                        {
                            error!("Session error: {:?}", err);
                        }
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
        // TODO: HACK: XXX: Send exact same packet twice. This minimizes the chance one is never received.
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
