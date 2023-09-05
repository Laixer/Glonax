use bytes::{Buf, BufMut, Bytes, BytesMut};

const PROTO_METRIC_VMS_UPTIME: u16 = 0x0;
const PROTO_METRIC_VMS_TIMESTAMP: u16 = 0x1;
const PROTO_METRIC_VMS_MEMORY_USAGE: u16 = 0x2;
const PROTO_METRIC_VMS_SWAP_USAGE: u16 = 0x3;
const PROTO_METRIC_VMS_CPU_LOAD: u16 = 0x4;

const PROTO_METRIC_GNSS_LATLONG: u16 = 0x14;
const PROTO_METRIC_GNSS_ALTITUDE: u16 = 0x15;
const PROTO_METRIC_GNSS_SPEED: u16 = 0x16;
const PROTO_METRIC_GNSS_HEADING: u16 = 0x17;
const PROTO_METRIC_GNSS_SATELLITES: u16 = 0x18;

const PROTO_METRIC_ENCODER_ABS_ANGLE: u16 = 0x50;
const PROTO_METRIC_ENCODER_RPM: u16 = 0x51;

const PROTO_METRIC_ENGINE_DRIVER_DEMAND: u16 = 0x60;
const PROTO_METRIC_ENGINE_ACTUAL_ENGINE: u16 = 0x61;
const PROTO_METRIC_ENGINE_RPM: u16 = 0x62;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Metric {
    /// VMS Uptime in seconds.
    VmsUptime(u64),
    /// VMS Timestamp in seconds.
    VmsTimestamp(chrono::DateTime<chrono::Utc>),
    /// VMS Memory total and used in bytes.
    VmsMemoryUsage((u64, u64)),
    /// VMS Swap total and used in bytes.
    VmsSwapUsage((u64, u64)),
    /// VMS CPU load.
    VmsCpuLoad((f64, f64, f64)),

    /// GNSS Latitude and Longitude.
    GnssLatLong((f32, f32)),
    /// GNSS Altitude in meters.
    GnssAltitude(f32),
    /// GNSS Speed in meters per second.
    GnssSpeed(f32),
    /// GNSS Heading in degrees.
    GnssHeading(f32),
    /// GNSS Satellites.
    GnssSatellites(u8),

    /// Encoder Absolute Angle in radians.
    EncoderAbsAngle((u8, f32)),
    /// Encoder RPM.
    EncoderRpm((u8, u16)),

    /// Engine Driver Demand in percent.
    EngineDriverDemand(u8),
    /// Engine Actual Engine in percent.
    EngineActualEngine(u8),
    /// Engine RPM.
    EngineRpm(u16),
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Signal {
    /// Signal metric.
    pub metric: Metric,
}

impl Signal {
    /// Create new signal.
    pub fn new(metric: Metric) -> Self {
        Self { metric }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.metric)
    }
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metric::VmsUptime(value) => write!(f, "VMS Uptime: {} seconds", value),
            Metric::VmsTimestamp(datetime) => {
                write!(f, "VMS Timestamp: {}", datetime)
            }
            Metric::VmsMemoryUsage((memory_used, memory_total)) => {
                write!(
                    f,
                    "VMS Memory usage: {:.2}GB / {:.2}GB",
                    *memory_used as f64 / 1024.0 / 1024.0 / 1024.0,
                    *memory_total as f64 / 1024.0 / 1024.0 / 1024.0
                )
            }
            Metric::VmsSwapUsage((swap_used, swap_total)) => {
                write!(
                    f,
                    "VMS Swap usage: {:.2}GB / {:.2}GB",
                    *swap_used as f64 / 1024.0 / 1024.0 / 1024.0,
                    *swap_total as f64 / 1024.0 / 1024.0 / 1024.0
                )
            }
            Metric::VmsCpuLoad((value_1, value_5, value_15)) => write!(
                f,
                "VMS CPU load: {:.1}%, {:.1}%, {:.1}%",
                value_1, value_5, value_15
            ),
            Metric::GnssLatLong((value_lat, value_long)) => {
                write!(f, "GNSS LatLong: ({:.5}, {:.5})", value_lat, value_long)
            }
            Metric::GnssAltitude(value) => write!(f, "GNSS Altitude: {:.1}m", value),
            Metric::GnssSpeed(value) => write!(f, "GNSS Speed: {:.1}m/s", value),
            Metric::GnssHeading(value) => write!(f, "GNSS Heading: {:.1}°", value),
            Metric::GnssSatellites(value) => write!(f, "GNSS Satellites: {}", value),
            Metric::EncoderAbsAngle((node, value)) => {
                write!(
                    f,
                    "Encoder 0x{:X?} Abs Angle: {:.2}rad {:.2}°",
                    node,
                    value,
                    (*value).to_degrees()
                )
            }
            Metric::EncoderRpm((node, value)) => write!(f, "Encoder 0x{:X?} RPM: {}", node, value),
            Metric::EngineDriverDemand(value) => write!(f, "Engine Driver Demand: {}%", value),
            Metric::EngineActualEngine(value) => write!(f, "Engine Actual Engine: {}%", value),
            Metric::EngineRpm(value) => write!(f, "Engine RPM: {}", value),
        }
    }
}

impl Signal {
    // TODO: Move to transport
    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(32);

        match self.metric {
            Metric::VmsUptime(value) => {
                buf.put_u16(PROTO_METRIC_VMS_UPTIME);
                buf.put_u64(value);
            }
            Metric::VmsTimestamp(value_datetime) => {
                buf.put_u16(PROTO_METRIC_VMS_TIMESTAMP);
                buf.put_i64(value_datetime.timestamp());
            }
            Metric::VmsMemoryUsage((value_used, value_total)) => {
                buf.put_u16(PROTO_METRIC_VMS_MEMORY_USAGE);
                buf.put_u64(value_used);
                buf.put_u64(value_total);
            }
            Metric::VmsSwapUsage((value_used, value_total)) => {
                buf.put_u16(PROTO_METRIC_VMS_SWAP_USAGE);
                buf.put_u64(value_used);
                buf.put_u64(value_total);
            }
            Metric::VmsCpuLoad((value_1, value_5, value_15)) => {
                buf.put_u16(PROTO_METRIC_VMS_CPU_LOAD);
                buf.put_f64(value_1);
                buf.put_f64(value_5);
                buf.put_f64(value_15);
            }
            Metric::GnssLatLong((value_lat, value_long)) => {
                buf.put_u16(PROTO_METRIC_GNSS_LATLONG);
                buf.put_f32(value_lat);
                buf.put_f32(value_long);
            }
            Metric::GnssAltitude(value) => {
                buf.put_u16(PROTO_METRIC_GNSS_ALTITUDE);
                buf.put_f32(value);
            }
            Metric::GnssSpeed(value) => {
                buf.put_u16(PROTO_METRIC_GNSS_SPEED);
                buf.put_f32(value);
            }
            Metric::GnssHeading(value) => {
                buf.put_u16(PROTO_METRIC_GNSS_HEADING);
                buf.put_f32(value);
            }
            Metric::GnssSatellites(value) => {
                buf.put_u16(PROTO_METRIC_GNSS_SATELLITES);
                buf.put_u8(value);
            }
            Metric::EncoderAbsAngle((node, value)) => {
                buf.put_u16(PROTO_METRIC_ENCODER_ABS_ANGLE);
                buf.put_u8(node);
                buf.put_f32(value);
            }
            Metric::EncoderRpm((node, value)) => {
                buf.put_u16(PROTO_METRIC_ENCODER_RPM);
                buf.put_u8(node);
                buf.put_u16(value);
            }
            Metric::EngineDriverDemand(value) => {
                buf.put_u16(PROTO_METRIC_ENGINE_DRIVER_DEMAND);
                buf.put_u8(value);
            }
            Metric::EngineActualEngine(value) => {
                buf.put_u16(PROTO_METRIC_ENGINE_ACTUAL_ENGINE);
                buf.put_u8(value);
            }
            Metric::EngineRpm(value) => {
                buf.put_u16(PROTO_METRIC_ENGINE_RPM);
                buf.put_u16(value);
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

        let metric = match buf.get_u16() {
            PROTO_METRIC_VMS_UPTIME => Metric::VmsUptime(buf.get_u64()),
            PROTO_METRIC_VMS_TIMESTAMP => {
                use chrono::prelude::*;

                let datetime = Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(buf.get_i64(), 0).unwrap(),
                );

                Metric::VmsTimestamp(datetime)
            }
            PROTO_METRIC_VMS_MEMORY_USAGE => Metric::VmsMemoryUsage((buf.get_u64(), buf.get_u64())),
            PROTO_METRIC_VMS_SWAP_USAGE => Metric::VmsSwapUsage((buf.get_u64(), buf.get_u64())),
            PROTO_METRIC_VMS_CPU_LOAD => {
                Metric::VmsCpuLoad((buf.get_f64(), buf.get_f64(), buf.get_f64()))
            }
            PROTO_METRIC_GNSS_LATLONG => Metric::GnssLatLong((buf.get_f32(), buf.get_f32())),
            PROTO_METRIC_GNSS_ALTITUDE => Metric::GnssAltitude(buf.get_f32()),
            PROTO_METRIC_GNSS_SPEED => Metric::GnssSpeed(buf.get_f32()),
            PROTO_METRIC_GNSS_HEADING => Metric::GnssHeading(buf.get_f32()),
            PROTO_METRIC_GNSS_SATELLITES => Metric::GnssSatellites(buf.get_u8()),
            PROTO_METRIC_ENCODER_ABS_ANGLE => {
                Metric::EncoderAbsAngle((buf.get_u8(), buf.get_f32()))
            }
            PROTO_METRIC_ENCODER_RPM => Metric::EncoderRpm((buf.get_u8(), buf.get_u16())),
            PROTO_METRIC_ENGINE_DRIVER_DEMAND => Metric::EngineDriverDemand(buf.get_u8()),
            PROTO_METRIC_ENGINE_ACTUAL_ENGINE => Metric::EngineActualEngine(buf.get_u8()),
            PROTO_METRIC_ENGINE_RPM => Metric::EngineRpm(buf.get_u16()),
            _ => return Err(()),
        };

        Ok(Self { metric })
    }
}
