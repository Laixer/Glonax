#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Metric {
    /// Temperature in degrees celcius.
    Temperature(f32),
    /// Angle in radians.
    Angle(f32),
    /// Speed in meters per second.
    Speed(f32),
    /// Altitude in meters.
    Altitude(f32),
    /// Heading in degrees.
    Heading(f32),
    /// Revolutions per minute.
    Rpm(i32),
    /// Acceleration in mg.
    Acceleration((f32, f32, f32)),
    /// Percentage.
    Percent(i32),
    /// WS84 coordinates.
    Coordinates((f32, f32)),
    /// Timestamp in seconds.
    Timestamp(f64),
    //// Point in 2D space.
    Point2D((f32, f32)),
    /// Point in 3D space.
    Point3D((f32, f32, f32)),
    /// Number of unspecificed units.
    Count(u32),
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Signal {
    /// Device address.
    pub address: u32,
    /// Device driver function.
    pub function: u32,
    /// Signal metric.
    pub metric: Metric,
}

impl Signal {
    /// Create new signal.
    pub fn new<I: Into<u32>>(address: I, function: I, metric: Metric) -> Self {
        Self {
            address: address.into(),
            function: function.into(),
            metric,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, std::mem::size_of::<Self>())
        }
    }

    pub fn from(data: &[u8]) -> Self {
        let (head, body, _tail) = unsafe { data.align_to::<Self>() };
        assert_eq!(head.len(), 0);

        body[0]
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:02X?}:{:02X?} - {}",
            self.address, self.function, self.metric
        )
    }
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metric::Temperature(value) => write!(f, "Temperature: {:>3}°C", value),
            Metric::Angle(value) => write!(
                f,
                "Angle: {:>6.2}rad {:>6.2}°",
                value,
                crate::core::rad_to_deg(*value)
            ),
            Metric::Speed(value) => write!(f, "Speed: {:.2}m/s", value),
            Metric::Altitude(value) => write!(f, "Altitude: {:.1}m", value),
            Metric::Heading(value) => write!(f, "Heading: {:.1}°", value),
            Metric::Rpm(value) => write!(f, "RPM: {}", value),
            Metric::Acceleration((value_x, value_y, value_z)) => {
                write!(
                    f,
                    "Acceleration (mg): X: {:>+5} Y: {:>+5} Z: {:>+5}",
                    value_x, value_y, value_z,
                )
            }
            Metric::Percent(value) => write!(f, "Per: {:.1}%", value),
            Metric::Coordinates((value_lat, value_long)) => {
                write!(
                    f,
                    "Coordinates: (Lat: {:.5}, Long: {:.5})",
                    value_lat, value_long
                )
            }
            Metric::Timestamp(value) => write!(f, "Timestamp: {:>+5}", value),
            Metric::Point2D((value_x, value_y)) => {
                write!(f, "Point2D: ({:.2}, {:.2})", value_x, value_y)
            }
            Metric::Point3D((value_x, value_y, value_z)) => {
                write!(
                    f,
                    "Point3D: ({:.2}, {:.2}, {:.2})",
                    value_x, value_y, value_z
                )
            }
            Metric::Count(value) => write!(f, "Count: {}", value),
        }
    }
}
