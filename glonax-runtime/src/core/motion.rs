use bytes::{Buf, BufMut, Bytes, BytesMut};

const PROTO_TYPE_STOP_ALL: u8 = 0x00;
const PROTO_TYPE_RESUME_ALL: u8 = 0x01;
const PROTO_TYPE_CHANGE: u8 = 0x02;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ChangeSet {
    /// Actuator ID.
    pub actuator: u32,
    /// Actuator value.
    pub value: i32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub enum Motion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Change motion on actuators.
    Change(Vec<ChangeSet>),
}

impl Motion {
    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(32);

        // buf.put(&PROTO_HEADER[..]);
        // buf.put_u8(PROTO_VERSION);
        // buf.put_u8(PROTO_MESSAGE);

        match self {
            Motion::StopAll => {
                buf.put_u8(PROTO_TYPE_STOP_ALL);
            }
            Motion::ResumeAll => {
                buf.put_u8(PROTO_TYPE_RESUME_ALL);
            }
            Motion::Change(changes) => {
                buf.put_u8(PROTO_TYPE_CHANGE);
                buf.put_u8(changes.len() as u8);
                for change in changes {
                    buf.put_u32(change.actuator);
                    buf.put_i32(change.value);
                }
            }
        }

        buf.to_vec()
    }
}

impl std::fmt::Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Motion::StopAll => write!(f, "Stop all"),
            Motion::ResumeAll => write!(f, "Resume all"),
            Motion::Change(changes) => {
                write!(
                    f,
                    "Change: {}",
                    changes
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

impl TryFrom<&[u8]> for Motion {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(value);

        match buf.get_u8() {
            PROTO_TYPE_STOP_ALL => Ok(Motion::StopAll),
            PROTO_TYPE_RESUME_ALL => Ok(Motion::ResumeAll),
            PROTO_TYPE_CHANGE => {
                let count = buf.get_u8();
                let mut changes = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    changes.push(ChangeSet {
                        actuator: buf.get_u32(),
                        value: buf.get_i32(),
                    });
                }
                Ok(Motion::Change(changes))
            }
            _ => Err(()),
        }
    }
}
