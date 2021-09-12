use crate::position::Vector3;

pub struct Acceleration(Vector3<i16>);

impl Acceleration {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self(Vector3 { x, y, z })
    }
}

impl From<(i16, i16, i16)> for Acceleration {
    fn from(value: (i16, i16, i16)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl std::fmt::Debug for Acceleration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Acceleration: X: {:>+5} Y: {:>+5} Z: {:>+5}",
            self.0.x, self.0.y, self.0.z
        )
    }
}

pub enum MetricValue {
    Temperature(i16),
    Acceleration(Acceleration),
}
