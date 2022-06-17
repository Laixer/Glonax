use std::time::SystemTime;

// TODO: Maybe not?
use nalgebra as na;

pub type SignalSource = u32;
pub type SignalTuple = (SignalSource, Signal);

#[derive(Copy, Clone, Debug)]
pub struct Signal {
    /// Timestamp when this signal was received.
    pub timestamp: SystemTime,
    /// Signal value.
    pub value: MetricValue,
}

impl Signal {
    /// Construct new signal.
    pub fn new(value: MetricValue) -> Self {
        Self {
            timestamp: SystemTime::now(),
            value,
        }
    }
}

unsafe impl Sync for Signal {}
unsafe impl Send for Signal {}

#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    // TODO: replace i16 with nalgebra::Vector1<f32>
    Temperature(i16),
    Acceleration(na::Vector3<f32>),
    Stroke(na::Vector1<u16>),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Temperature(scalar) => write!(f, "Temperature: {:>+3}", scalar),
            MetricValue::Acceleration(vector) => {
                write!(
                    f,
                    "Acceleration (mg): X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    vector.x, vector.y, vector.z,
                )
            }
            MetricValue::Stroke(vector) => write!(f, "Stroke: {:>+5}", vector.x),
        }
    }
}
