#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Temperature(i16),
    Acceleration(nalgebra::Vector3<f32>),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Temperature(scalar) => write!(f, "Temperature: {:>+3}", scalar),
            MetricValue::Acceleration(vector) => {
                write!(
                    f,
                    "Acceleration (mg): X: {:>+5} Y: {:>+5} Z: {:>+5}\t(ms^2) X: {:>+5.2} Y: {:>+5.2} Z: {:>+5.2}",
                    vector.x,
                    vector.y,
                    vector.z,
                    vector.x * 0.00980665,
                    vector.y * 0.00980665,
                    vector.z * 0.00980665
                )
            }
        }
    }
}
