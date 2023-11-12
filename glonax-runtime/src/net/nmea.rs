use crate::runtime::SharedOperandState;

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
            .replace('.', "")
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

            if sentence.is_empty() {
                return this;
            }

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

            if !sentence[6].is_empty() {
                let fix_quality = sentence[6].parse::<u8>().unwrap();
                if fix_quality == 1 || fix_quality == 2 {
                    let lat_line = sentence[2];
                    let lat_quadrant = sentence[3].to_uppercase().chars().next().unwrap();

                    let long_line = sentence[4];
                    let long_quadrant = sentence[5].to_uppercase().chars().next().unwrap();

                    this.coordinates = Some((
                        Self::dms_to_degree(lat_line, lat_quadrant),
                        Self::dms_to_degree(long_line, long_quadrant),
                    ));
                }
            }

            if !sentence[7].is_empty() {
                this.satellites = Some(sentence[7].parse::<u8>().unwrap());
            }

            if !sentence[9].is_empty() && !sentence[10].is_empty() {
                let altitude = sentence[9].parse::<f32>().unwrap();
                let altitude_unit = sentence[10].to_uppercase().chars().next().unwrap();

                if altitude_unit == 'M' {
                    this.altitude = Some(altitude);
                }
            }
        } else if line.starts_with("$GNGLL") {
            let sentence: Vec<&str> = line.split(',').collect();

            if sentence.is_empty() {
                return this;
            }

            // let last = sentence.last().unwrap();
            // if !last.starts_with("N*") {
            //     return this;
            // }

            if sentence[6].is_empty() {
                return this;
            }

            let validity = sentence[6].to_uppercase().chars().next().unwrap();
            if validity == 'A' {
                let lat_line = sentence[1];
                let lat_quadrant = sentence[2].to_uppercase().chars().next().unwrap();

                let long_line = sentence[3];
                let long_quadrant = sentence[4].to_uppercase().chars().next().unwrap();

                this.coordinates = Some((
                    Self::dms_to_degree(lat_line, lat_quadrant),
                    Self::dms_to_degree(long_line, long_quadrant),
                ));
            }
        } else if line.starts_with("$GNRMC") {
            let sentence: Vec<&str> = line.split(',').collect();

            if sentence.is_empty() {
                return this;
            }

            // let last = sentence.last().unwrap();
            // if !last.starts_with("N*") {
            //     return this;
            // }

            if sentence[2].is_empty() {
                return this;
            }

            let validity = sentence[2].to_uppercase().chars().next().unwrap();
            if validity == 'A' {
                let lat_line = sentence[3];
                let lat_quadrant = sentence[4].to_uppercase().chars().next().unwrap();

                let long_line = sentence[5];
                let long_quadrant = sentence[6].to_uppercase().chars().next().unwrap();

                this.coordinates = Some((
                    Self::dms_to_degree(lat_line, lat_quadrant),
                    Self::dms_to_degree(long_line, long_quadrant),
                ));
            }

            if !sentence[7].is_empty() {
                this.speed = Some(sentence[7].parse::<f32>().unwrap());
            }

            if !sentence[8].is_empty() {
                this.heading = Some(sentence[8].parse::<f32>().unwrap());
            }
        }

        this
    }

    pub async fn fill(&self, local_runtime_state: SharedOperandState) {
        let mut runtime_state = local_runtime_state.write().await;

        if let Some((lat, long)) = self.coordinates {
            runtime_state.state.gnss.location = (lat, long)
        }
        if let Some(altitude) = self.altitude {
            runtime_state.state.gnss.altitude = altitude;
        }
        if let Some(speed) = self.speed {
            const KNOT_TO_METER_PER_SECOND: f32 = 0.5144;

            runtime_state.state.gnss.speed = speed * KNOT_TO_METER_PER_SECOND;
        }
        if let Some(heading) = self.heading {
            runtime_state.state.gnss.heading = heading;
        }
        if let Some(satellites) = self.satellites {
            runtime_state.state.gnss.satellites = satellites;
        }
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

#[derive(Default)]
pub struct NMEAService;

impl NMEAService {
    pub fn decode(&self, line: String) -> Option<NMEAMessage> {
        if line.starts_with("$GNGGA") || line.starts_with("$GNGLL") || line.starts_with("$GNRMC") {
            Some(NMEAMessage::decode(line))
        } else {
            None
        }
    }
}
