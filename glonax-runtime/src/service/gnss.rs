use std::path::PathBuf;

use glonax_serial::{BaudRate, Uart};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};

use crate::runtime::{Service, SharedOperandState};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct GnssConfig {
    /// Path to the serial device
    pub device: PathBuf,
    /// Baud rate of the serial device
    pub baud_rate: usize,
}

pub struct Gnss {
    line_reader: Lines<BufReader<Uart>>,
}

impl Service<GnssConfig> for Gnss {
    fn new(config: GnssConfig) -> Self
    where
        Self: Sized,
    {
        log::debug!(
            "Starting GNSS service on {}:{}",
            config.device.display(),
            config.baud_rate
        );

        let serial = Uart::open(&config.device, BaudRate::from_speed(config.baud_rate))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .unwrap();

        Self {
            line_reader: BufReader::new(serial).lines(),
        }
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        let driver = crate::driver::Nmea;

        while let Ok(Some(line)) = self.line_reader.next_line().await {
            if let Some(message) = driver.decode(line) {
                let mut runtime_state = runtime_state.write().await;

                if let Some((lat, long)) = message.coordinates {
                    runtime_state.state.gnss.location = (lat, long)
                }
                if let Some(altitude) = message.altitude {
                    runtime_state.state.gnss.altitude = altitude;
                }
                if let Some(speed) = message.speed {
                    const KNOT_TO_METER_PER_SECOND: f32 = 0.5144;

                    runtime_state.state.gnss.speed = speed * KNOT_TO_METER_PER_SECOND;
                }
                if let Some(heading) = message.heading {
                    runtime_state.state.gnss.heading = heading;
                }
                if let Some(satellites) = message.satellites {
                    runtime_state.state.gnss.satellites = satellites;
                }
            }
        }
    }
}
