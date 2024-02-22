use tokio::io::{AsyncBufReadExt, BufReader, Lines};

use crate::runtime::{Service, SharedOperandState};

pub struct Gnss {
    line_reader: Lines<BufReader<glonax_serial::Uart>>,
}

impl<Cnf> Service<Cnf> for Gnss {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting GNSS service");

        let serial = glonax_serial::Uart::open(
            std::path::Path::new("/dev/ttyUSB0"), // TODO: "nmea_config.device
            glonax_serial::BaudRate::from_speed(9600), // TODO: "nmea_config.baud_rate
        )
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
