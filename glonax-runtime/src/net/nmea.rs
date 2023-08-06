use crate::core::{Metric, Signal};

// TODO: Timestamp
pub struct NMEAMessage {
    /// WGS 84 coordinates.
    pub coordinates: Option<(f32, f32)>,
    /// Number of satellites.
    pub satellites: Option<u8>,
    /// Altitude.
    pub altitude: Option<f32>,
    /// Speed.
    pub speed: Option<f32>,
    /// Heading.
    pub heading: Option<f32>,
    /// Timestamp.
    pub timestamp: Option<f64>,
}

impl NMEAMessage {
    fn dms_to_degree(str: &str, quadrant: char) -> f32 {
        let offset = if str.find('.').unwrap() == 4 { 2 } else { 3 };

        let day = str
            .chars()
            .take(offset)
            .collect::<String>()
            .parse::<f32>()
            .unwrap();
        let min = str
            .replace(".", "")
            .chars()
            .skip(offset)
            .collect::<String>()
            .parse::<f32>()
            .unwrap();
        let min = (min / 60.0) / 100000.0;

        let degrees = day + min;

        if quadrant == 'S' || quadrant == 'W' {
            degrees * -1.0
        } else {
            degrees
        }
    }

    fn decode(line: String) -> Self {
        let mut this = Self {
            coordinates: None,
            satellites: None,
            altitude: None,
            speed: None,
            heading: None,
            timestamp: None,
        };

        if line.starts_with("$GNGGA") {
            let sentence: Vec<&str> = line.split(',').collect();

            // let hour = sentence[1]
            //     .chars()
            //     .take(2)
            //     .collect::<String>()
            //     .parse::<u64>()
            //     .unwrap();
            // let minute = sentence[1]
            //     .chars()
            //     .skip(2)
            //     .take(2)
            //     .collect::<String>()
            //     .parse::<u64>()
            //     .unwrap();
            // let second = sentence[1]
            //     .chars()
            //     .skip(4)
            //     .take(2)
            //     .collect::<String>()
            //     .parse::<u64>()
            //     .unwrap();
            // println!("Timestamp: {}:{}:{}", hour, minute, second);

            if sentence[6].len() > 0 {
                let fix_quality = sentence[6].parse::<u8>().unwrap();
                if fix_quality == 1 || fix_quality == 2 {
                    let lat_line = sentence[2];
                    let lat_quadrant = sentence[3].to_uppercase().chars().next().unwrap();

                    let long_line = sentence[4];
                    let long_quadrant = sentence[5].to_uppercase().chars().next().unwrap();

                    this.coordinates = Some((
                        Self::dms_to_degree(lat_line, lat_quadrant) as f32,
                        Self::dms_to_degree(long_line, long_quadrant),
                    ));
                }
            }

            if sentence[7].len() > 0 {
                this.satellites = Some(sentence[7].parse::<u8>().unwrap());
            }

            if sentence[9].len() > 0 && sentence[10].len() > 0 {
                let altitude = sentence[9].parse::<f32>().unwrap();
                let altitude_unit = sentence[10].to_uppercase().chars().next().unwrap();

                if altitude_unit == 'M' {
                    this.altitude = Some(altitude);
                }
            }
        } else if line.starts_with("$GNGLL") {
            let sentence: Vec<&str> = line.split(',').collect();

            if sentence[6].len() > 0 {
                let validity = sentence[6].to_uppercase().chars().next().unwrap();
                if validity == 'A' {
                    let lat_line = sentence[1];
                    let lat_quadrant = sentence[2].to_uppercase().chars().next().unwrap();

                    let long_line = sentence[3];
                    let long_quadrant = sentence[4].to_uppercase().chars().next().unwrap();

                    this.coordinates = Some((
                        Self::dms_to_degree(lat_line, lat_quadrant) as f32,
                        Self::dms_to_degree(long_line, long_quadrant),
                    ));
                }
            }
        } else if line.starts_with("$GNRMC") {
            let sentence: Vec<&str> = line.split(',').collect();

            if sentence[2].len() > 0 {
                let validity = sentence[2].to_uppercase().chars().next().unwrap();
                if validity == 'A' {
                    let lat_line = sentence[3];
                    let lat_quadrant = sentence[4].to_uppercase().chars().next().unwrap();

                    let long_line = sentence[5];
                    let long_quadrant = sentence[6].to_uppercase().chars().next().unwrap();

                    this.coordinates = Some((
                        Self::dms_to_degree(lat_line, lat_quadrant) as f32,
                        Self::dms_to_degree(long_line, long_quadrant),
                    ));
                }
            }

            if sentence[7].len() > 0 {
                this.speed = Some(sentence[7].parse::<f32>().unwrap());
            }

            if sentence[8].len() > 0 {
                this.heading = Some(sentence[8].parse::<f32>().unwrap());
            }
        }

        this
    }
}

impl std::fmt::Display for NMEAMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Coordinates: {}; satellites: {}; altitude: {}; speed: {}; heading: {}",
            self.coordinates.as_ref().map_or_else(
                || "-".to_owned(),
                |(lat, long)| format!("({:.5}, {:.5})", lat, long)
            ),
            self.satellites
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| format!("{}", f)),
            self.altitude
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| format!("{:.1}m", f)),
            self.speed
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| format!("{:.1}kts", f)),
            self.heading
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| format!("{:.1}°", f)),
        )
    }
}

impl crate::channel::SignalSource for NMEAMessage {
    fn fetch2(&self, writer: &mut impl crate::channel::SignalChannel) {
        if let Some((lat, long)) = self.coordinates {
            writer.push(Signal::new(1_u32, 0_u32, Metric::Coordinates((lat, long))))
        }
        if let Some(satellites) = self.satellites {
            writer.push(Signal::new(1_u32, 10_u32, Metric::Count(satellites as u64)))
        }
        if let Some(altitude) = self.altitude {
            writer.push(Signal::new(1_u32, 1_u32, Metric::Altitude(altitude)))
        }
        if let Some(speed) = self.speed {
            const KNOT_TO_METER_PER_SECOND: f32 = 0.5144;

            writer.push(Signal::new(
                1_u32,
                2_u32,
                Metric::Speed(speed * KNOT_TO_METER_PER_SECOND),
            ))
        }
        if let Some(heading) = self.heading {
            writer.push(Signal::new(1_u32, 3_u32, Metric::Heading(heading)))
        }
        // TODO: Timestamp
        // if let Some(timestamp) = self.timestamp {
        //     writer.push(Signal::new(
        //         1_u32,
        //         4_u32,
        //         Metric::Timestamp(timestamp as f64),
        //     ))
        // }
    }
}

pub struct NMEAService;

impl NMEAService {
    pub fn decode(&self, line: String) -> Option<NMEAMessage> {
        if line.starts_with("$GNGGA") {
            Some(NMEAMessage::decode(line))
        } else if line.starts_with("$GNGLL") {
            Some(NMEAMessage::decode(line))
        } else if line.starts_with("$GNRMC") {
            Some(NMEAMessage::decode(line))
        } else {
            None
        }
    }
}
