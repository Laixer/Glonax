#[derive(Debug)]
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
                    "Acceleration: X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    vector.x, vector.y, vector.z
                )
            }
        }
    }
}
