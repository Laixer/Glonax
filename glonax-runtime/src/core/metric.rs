#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Signal {
    /// Node address.
    pub address: u8,
    /// Node address.
    pub subaddress: u8,
    /// Signal value.
    pub value: MetricValue,
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Address: 0x{:X?}:{} {}",
            self.address, self.subaddress, self.value
        )
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum MetricValue {
    Temperature(f32),
    Acceleration((f32, f32, f32)),
    Angle(u32),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Temperature(scalar) => write!(f, "Temperature: {:>+3}", scalar),
            MetricValue::Acceleration((x, y, z)) => {
                write!(
                    f,
                    "Acceleration (mg): X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    x, y, z,
                )
            }
            MetricValue::Angle(value) => write!(f, "Angle: {:>+5}", value),
        }
    }
}
