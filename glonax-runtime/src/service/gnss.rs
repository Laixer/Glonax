use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};

use glonax_serial::{BaudRate, Uart};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};

use crate::runtime::{CommandSender, Service, ServiceContext, SharedOperandState, SignalSender};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct GnssConfig {
    /// Path to the serial device
    pub device: PathBuf,
    /// Baud rate of the serial device
    pub baud_rate: usize,
}

pub struct Gnss {
    line_reader: Lines<BufReader<Uart>>,
    path: PathBuf,
    driver: crate::driver::Nmea,
}

impl Service<GnssConfig> for Gnss {
    fn new(config: GnssConfig) -> Self
    where
        Self: Sized,
    {
        let serial = Uart::open(&config.device, BaudRate::from_speed(config.baud_rate))
            .map_err(|e| Error::new(ErrorKind::Other, e))
            .unwrap();

        Self {
            line_reader: BufReader::new(serial).lines(),
            path: config.device,
            driver: crate::driver::Nmea,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("gnss", self.path.display().to_string())
    }

    async fn wait_io(
        &mut self,
        _runtime_state: SharedOperandState,
        signal_tx: SignalSender,
        _command_tx: CommandSender,
    ) {
        if let Ok(Some(line)) = self.line_reader.next_line().await {
            if let Some(message) = self.driver.decode(line) {
                let mut gnss = crate::core::Gnss::default();

                if let Some((lat, long)) = message.coordinates {
                    gnss.location = (lat, long)
                }
                if let Some(altitude) = message.altitude {
                    gnss.altitude = altitude;
                }
                if let Some(speed) = message.speed {
                    const KNOT_TO_METER_PER_SECOND: f32 = 0.5144;

                    gnss.speed = speed * KNOT_TO_METER_PER_SECOND;
                }
                if let Some(heading) = message.heading {
                    gnss.heading = heading;
                }
                if let Some(satellites) = message.satellites {
                    gnss.satellites = satellites;
                }

                if let Err(e) = signal_tx.send(crate::core::Object::GNSS(gnss)) {
                    log::error!("Failed to send GNSS signal: {}", e);
                }

                // TODO: state will not exist in the future
                // let mut runtime_state = runtime_state.write().await;
                // runtime_state.state.gnss_signal_instant = Some(std::time::Instant::now());
                // runtime_state.state.gnss_signal = gnss;
            }
        }
    }
}
