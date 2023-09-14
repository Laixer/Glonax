use bytes::{BufMut, BytesMut};
use serde_derive::Deserialize;

const PROTO_STATUS_HEALTHY: u8 = 0xF8;
const PROTO_STATUS_DEGRADED_HIGH_USAGE_MEMORY: u8 = 0x10;
const PROTO_STATUS_DEGRADED_HIGH_USAGE_SWAP: u8 = 0x11;
const PROTO_STATUS_DEGRADED_HIGH_USAGE_CPU: u8 = 0x12;
const PROTO_STATUS_DEGRADED_TIMEOUT_GNSS: u8 = 0x21;
const PROTO_STATUS_DEGRADED_TIMEOUT_IMU: u8 = 0x22;
const PROTO_STATUS_DEGRADED_TIMEOUT_ENCODER: u8 = 0x23;
const PROTO_STATUS_DEGRADED_TIMEOUT_ENGINE: u8 = 0x24;
const PROTO_STATUS_FAULTY_LINK_DOWN_CAN1: u8 = 0x33;
const PROTO_STATUS_FAULTY_LINK_DOWN_CAN2: u8 = 0x34;

#[derive(Clone, Debug, Deserialize)]
pub enum Status {
    Healthy,
    DegradedHighUsageMemory,
    DegradedHighUsageSwap,
    DegradedHighUsageCPU,
    DegradedTimeoutGNSS,
    DegradedTimeoutIMU,
    DegradedTimeoutEncoder,
    DegradedTimeoutEngine,
    FaultyLinkDownCAN1,
    FaultyLinkDownCAN2,
}

impl Status {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(1);

        match self {
            Status::Healthy => buf.put_u8(PROTO_STATUS_HEALTHY),
            Status::DegradedHighUsageMemory => buf.put_u8(PROTO_STATUS_DEGRADED_HIGH_USAGE_MEMORY),
            Status::DegradedHighUsageSwap => buf.put_u8(PROTO_STATUS_DEGRADED_HIGH_USAGE_SWAP),
            Status::DegradedHighUsageCPU => buf.put_u8(PROTO_STATUS_DEGRADED_HIGH_USAGE_CPU),
            Status::DegradedTimeoutGNSS => buf.put_u8(PROTO_STATUS_DEGRADED_TIMEOUT_GNSS),
            Status::DegradedTimeoutIMU => buf.put_u8(PROTO_STATUS_DEGRADED_TIMEOUT_IMU),
            Status::DegradedTimeoutEncoder => buf.put_u8(PROTO_STATUS_DEGRADED_TIMEOUT_ENCODER),
            Status::DegradedTimeoutEngine => buf.put_u8(PROTO_STATUS_DEGRADED_TIMEOUT_ENGINE),
            Status::FaultyLinkDownCAN1 => buf.put_u8(PROTO_STATUS_FAULTY_LINK_DOWN_CAN1),
            Status::FaultyLinkDownCAN2 => buf.put_u8(PROTO_STATUS_FAULTY_LINK_DOWN_CAN2),
        }

        buf.to_vec()
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Healthy => write!(f, "Healthy"),
            Status::DegradedHighUsageMemory => write!(f, "Degraded (High Usage Memory)"),
            Status::DegradedHighUsageSwap => write!(f, "Degraded (High Usage Swap)"),
            Status::DegradedHighUsageCPU => write!(f, "Degraded (High Usage CPU)"),
            Status::DegradedTimeoutGNSS => write!(f, "Degraded (Timeout GNSS)"),
            Status::DegradedTimeoutIMU => write!(f, "Degraded (Timeout IMU)"),
            Status::DegradedTimeoutEncoder => write!(f, "Degraded (Timeout Encoder)"),
            Status::DegradedTimeoutEngine => write!(f, "Degraded (Timeout Engine)"),
            Status::FaultyLinkDownCAN1 => write!(f, "Faulty (Link Down CAN1)"),
            Status::FaultyLinkDownCAN2 => write!(f, "Faulty (Link Down CAN2)"),
        }
    }
}

impl TryFrom<&[u8]> for Status {
    type Error = ();

    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        if buffer.len() < 1 {
            log::warn!("Invalid buffer size");
            return Err(());
        }

        match buffer[0] {
            PROTO_STATUS_HEALTHY => Ok(Status::Healthy),
            PROTO_STATUS_DEGRADED_HIGH_USAGE_MEMORY => Ok(Status::DegradedHighUsageMemory),
            PROTO_STATUS_DEGRADED_HIGH_USAGE_SWAP => Ok(Status::DegradedHighUsageSwap),
            PROTO_STATUS_DEGRADED_HIGH_USAGE_CPU => Ok(Status::DegradedHighUsageCPU),
            PROTO_STATUS_DEGRADED_TIMEOUT_GNSS => Ok(Status::DegradedTimeoutGNSS),
            PROTO_STATUS_DEGRADED_TIMEOUT_IMU => Ok(Status::DegradedTimeoutIMU),
            PROTO_STATUS_DEGRADED_TIMEOUT_ENCODER => Ok(Status::DegradedTimeoutEncoder),
            PROTO_STATUS_DEGRADED_TIMEOUT_ENGINE => Ok(Status::DegradedTimeoutEngine),
            PROTO_STATUS_FAULTY_LINK_DOWN_CAN1 => Ok(Status::FaultyLinkDownCAN1),
            PROTO_STATUS_FAULTY_LINK_DOWN_CAN2 => Ok(Status::FaultyLinkDownCAN2),
            _ => Err(()),
        }
    }
}
