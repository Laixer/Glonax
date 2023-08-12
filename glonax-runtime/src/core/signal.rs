use bytes::{Buf, BufMut, Bytes, BytesMut};

const PROTO_METRIC_TEMPERATURE: u8 = 0x00;
const PROTO_METRIC_ANGLE: u8 = 0x01;
const PROTO_METRIC_SPEED: u8 = 0x02;
const PROTO_METRIC_ALTITUDE: u8 = 0x03;
const PROTO_METRIC_HEADING: u8 = 0x04;
const PROTO_METRIC_RPM: u8 = 0x05;
const PROTO_METRIC_ACCELERATION: u8 = 0x06;
const PROTO_METRIC_PERCENT: u8 = 0x07;
const PROTO_METRIC_COORDINATES: u8 = 0x08;
const PROTO_METRIC_TIMESTAMP: u8 = 0x09;
const PROTO_METRIC_POINT2D: u8 = 0x0A;
const PROTO_METRIC_POINT3D: u8 = 0x0B;
const PROTO_METRIC_COUNT: u8 = 0x0C;

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
    /// WGS 84 coordinates.
    Coordinates((f32, f32)),
    /// Timestamp in seconds.
    Timestamp(u64),
    //// Point in 2D space.
    Point2D((f32, f32)),
    /// Point in 3D space.
    Point3D((f32, f32, f32)),
    /// Number of unspecificed units.
    Count(u64),
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
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:02X?}:{:02X?} » {}",
            self.address, self.function, self.metric
        )
    }
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metric::Temperature(value) => write!(f, "{:.1}°C", value),
            Metric::Angle(value) => write!(
                f,
                "{:>6.2}rad {:>6.2}°",
                value,
                crate::core::rad_to_deg(*value)
            ),
            Metric::Speed(value) => write!(f, "{:.2}m/s", value),
            Metric::Altitude(value) => write!(f, "{:.1}m", value),
            Metric::Heading(value) => write!(f, "{:.1}°", value),
            Metric::Rpm(value) => write!(f, "{}rpm", value),
            Metric::Acceleration((value_x, value_y, value_z)) => {
                write!(f, "({:.2}, {:.2}, {:.2})", value_x, value_y, value_z)
            }
            Metric::Percent(value) => write!(f, "{:.1}%", value),
            Metric::Coordinates((value_lat, value_long)) => {
                write!(f, "({:.5}, {:.5})", value_lat, value_long)
            }
            Metric::Timestamp(value) => {
                use chrono::{DateTime, NaiveDateTime, Utc};

                if let Some(naive_datetime) = NaiveDateTime::from_timestamp_opt(*value as i64, 0) {
                    write!(f, "{}", DateTime::<Utc>::from_utc(naive_datetime, Utc))
                } else {
                    write!(f, "{:>+5}", value)
                }
            }
            Metric::Point2D((value_x, value_y)) => {
                write!(f, "({:.2}, {:.2})", value_x, value_y)
            }
            Metric::Point3D((value_x, value_y, value_z)) => {
                write!(f, "({:.2}, {:.2}, {:.2})", value_x, value_y, value_z)
            }
            Metric::Count(value) => write!(f, "{}", value),
        }
    }
}

impl Signal {
    // TODO: Move to transport
    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(32);

        buf.put_u32(self.address);
        buf.put_u32(self.function);

        match self.metric {
            Metric::Temperature(value) => {
                buf.put_u8(PROTO_METRIC_TEMPERATURE);
                buf.put_f32(value);
            }
            Metric::Angle(value) => {
                buf.put_u8(PROTO_METRIC_ANGLE);
                buf.put_f32(value);
            }
            Metric::Speed(value) => {
                buf.put_u8(PROTO_METRIC_SPEED);
                buf.put_f32(value);
            }
            Metric::Altitude(value) => {
                buf.put_u8(PROTO_METRIC_ALTITUDE);
                buf.put_f32(value);
            }
            Metric::Heading(value) => {
                buf.put_u8(PROTO_METRIC_HEADING);
                buf.put_f32(value);
            }
            Metric::Rpm(value) => {
                buf.put_u8(PROTO_METRIC_RPM);
                buf.put_i32(value);
            }
            Metric::Acceleration((value_x, value_y, value_z)) => {
                buf.put_u8(PROTO_METRIC_ACCELERATION);
                buf.put_f32(value_x);
                buf.put_f32(value_y);
                buf.put_f32(value_z);
            }
            Metric::Percent(value) => {
                buf.put_u8(PROTO_METRIC_PERCENT);
                buf.put_i32(value);
            }
            Metric::Coordinates((value_lat, value_long)) => {
                buf.put_u8(PROTO_METRIC_COORDINATES);
                buf.put_f32(value_lat);
                buf.put_f32(value_long);
            }
            Metric::Timestamp(value) => {
                buf.put_u8(PROTO_METRIC_TIMESTAMP);
                buf.put_u64(value);
            }
            Metric::Point2D((value_x, value_y)) => {
                buf.put_u8(PROTO_METRIC_POINT2D);
                buf.put_f32(value_x);
                buf.put_f32(value_y);
            }
            Metric::Point3D((value_x, value_y, value_z)) => {
                buf.put_u8(PROTO_METRIC_POINT3D);
                buf.put_f32(value_x);
                buf.put_f32(value_y);
                buf.put_f32(value_z);
            }
            Metric::Count(value) => {
                buf.put_u8(PROTO_METRIC_COUNT);
                buf.put_u64(value);
            }
        }

        buf.to_vec()
    }
}

// TODO: Move to transport
impl TryFrom<&[u8]> for Signal {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(value);

        let address = buf.get_u32();
        let function = buf.get_u32();

        let metric = match buf.get_u8() {
            PROTO_METRIC_TEMPERATURE => Metric::Temperature(buf.get_f32()),
            PROTO_METRIC_ANGLE => Metric::Angle(buf.get_f32()),
            PROTO_METRIC_SPEED => Metric::Speed(buf.get_f32()),
            PROTO_METRIC_ALTITUDE => Metric::Altitude(buf.get_f32()),
            PROTO_METRIC_HEADING => Metric::Heading(buf.get_f32()),
            PROTO_METRIC_RPM => Metric::Rpm(buf.get_i32()),
            PROTO_METRIC_ACCELERATION => {
                Metric::Acceleration((buf.get_f32(), buf.get_f32(), buf.get_f32()))
            }
            PROTO_METRIC_PERCENT => Metric::Percent(buf.get_i32()),
            PROTO_METRIC_COORDINATES => Metric::Coordinates((buf.get_f32(), buf.get_f32())),
            PROTO_METRIC_TIMESTAMP => Metric::Timestamp(buf.get_u64()),
            PROTO_METRIC_POINT2D => Metric::Point2D((buf.get_f32(), buf.get_f32())),
            PROTO_METRIC_POINT3D => Metric::Point3D((buf.get_f32(), buf.get_f32(), buf.get_f32())),
            PROTO_METRIC_COUNT => Metric::Count(buf.get_u64()),
            _ => return Err(()),
        };

        Ok(Self {
            address,
            function,
            metric,
        })
    }
}
