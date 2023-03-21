tonic::include_proto!("glonax");

pub trait ToMotion: Sync + Send {
    /// Convert into motion.
    fn to_motion(self) -> Motion;
}

impl ToMotion for Motion {
    fn to_motion(self) -> Self {
        self
    }
}

impl std::fmt::Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.r#type() {
            motion::MotionType::None => panic!("NONE should not be used"),
            motion::MotionType::StopAll => write!(f, "Stop all"),
            motion::MotionType::ResumeAll => write!(f, "Resume all"),
            motion::MotionType::Change => {
                write!(
                    f,
                    "Change: {}",
                    self.changes
                        .iter()
                        .map(|changeset| format!(
                            "Actuator: {}; Value: {}, ",
                            changeset.actuator, changeset.value
                        ))
                        .collect::<String>()
                )
            }
        }
    }
}

impl Signal {
    pub fn new(address: u32, metric: signal::Metric) -> Self {
        Self {
            address,
            metric: Some(metric),
        }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Address: 0x{:X?} {}",
            self.address,
            self.metric.as_ref().unwrap()
        )
    }
}

impl std::fmt::Display for signal::Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            signal::Metric::Temperature(scalar) => write!(f, "Temperature: {:>+3}", scalar),
            signal::Metric::Acceleration(value) => {
                write!(
                    f,
                    "Acceleration (mg): X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    value.x, value.y, value.z,
                )
            }
            signal::Metric::Angle(value) => write!(f, "Angle: {:>+5}", value),
            signal::Metric::Speed(value) => write!(f, "Speed: {:>+5}", value),
            signal::Metric::Rpm(value) => write!(f, "RPM: {:>+5}", value),
        }
    }
}
