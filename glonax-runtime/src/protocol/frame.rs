use bytes::{BufMut, BytesMut};

use super::{MAX_PAYLOAD_SIZE, MIN_BUFFER_SIZE, PROTO_HEADER, PROTO_VERSION};

pub enum FrameError {
    FrameTooSmall,
    InvalidHeader,
    VersionMismatch(u8),
    InvalidMessage(u8),
    ExcessivePayloadLength(usize),
    InvalidPadding,
    InvalidSessionFlags,
}

impl std::error::Error for FrameError {}

impl std::fmt::Debug for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FrameTooSmall => write!(f, "FrameTooSmall"),
            Self::InvalidHeader => write!(f, "InvalidHeader"),
            Self::VersionMismatch(got) => write!(f, "VersionMismatch({})", got),
            Self::InvalidMessage(message) => write!(f, "InvalidMessage({})", message),
            Self::ExcessivePayloadLength(len) => write!(f, "ExcessivePayloadLength({})", len),
            Self::InvalidPadding => write!(f, "InvalidPadding"),
            Self::InvalidSessionFlags => write!(f, "InvalidSessionFlags"),
        }
    }
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FrameTooSmall => write!(f, "frame too small"),
            Self::InvalidHeader => write!(f, "invalid header"),
            Self::VersionMismatch(got) => write!(f, "version mismatch: {}", got),
            Self::InvalidMessage(message) => write!(f, "invalid message type: {}", message),
            Self::ExcessivePayloadLength(len) => write!(f, "excessive payload length: {}", len),
            Self::InvalidPadding => write!(f, "invalid padding"),
            Self::InvalidSessionFlags => write!(f, "invalid session flags"),
        }
    }
}

// TODO: Split connection management and data messages
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FrameMessage {
    // Connection management messages
    Error = 0x0,
    Echo = 0x1,
    Session = 0x10,
    Shutdown = 0x11,
    Request = 0x12,
    // Data messages
    Instance = 0x15,
    Status = 0x16, // TODO: Integrate with Instance
    // Control messages
    Motion = 0x20,
    // Data messages
    Pose = 0x40,
    VMS = 0x41,
    GNSS = 0x42,
    Engine = 0x43,
}

impl FrameMessage {
    // TODO: TryFrom
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Self::Error),
            0x1 => Some(Self::Echo),
            0x10 => Some(Self::Session),
            0x11 => Some(Self::Shutdown),
            0x12 => Some(Self::Request),
            0x15 => Some(Self::Instance),
            0x16 => Some(Self::Status),
            0x20 => Some(Self::Motion),
            0x40 => Some(Self::Pose),
            0x41 => Some(Self::VMS),
            0x42 => Some(Self::GNSS),
            0x43 => Some(Self::Engine),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Error => 0x0,
            Self::Echo => 0x1,
            Self::Session => 0x10,
            Self::Shutdown => 0x11,
            Self::Request => 0x12,
            Self::Instance => 0x15,
            Self::Status => 0x16,
            Self::Motion => 0x20,
            Self::Pose => 0x40,
            Self::VMS => 0x41,
            Self::GNSS => 0x42,
            Self::Engine => 0x43,
        }
    }
}

pub struct Frame {
    buffer: BytesMut,
    pub message: FrameMessage,
    pub payload_length: usize,
}

impl Frame {
    pub fn new(message: FrameMessage, payload_length: usize) -> Self {
        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload_length);

        buffer.put(&PROTO_HEADER[..]);
        buffer.put_u8(PROTO_VERSION);
        buffer.put_u8(message.to_u8());
        buffer.put_u16(payload_length as u16);
        buffer.put(&[0u8; 3][..]);

        Self {
            buffer,
            message,
            payload_length,
        }
    }

    #[inline]
    pub fn put(&mut self, payload: &[u8]) {
        self.buffer.put(payload);
    }

    #[inline]
    pub fn payload_range(&self) -> std::ops::Range<usize> {
        MIN_BUFFER_SIZE..MIN_BUFFER_SIZE + self.payload_length
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = FrameError;

    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        if buffer.len() < MIN_BUFFER_SIZE {
            Err(FrameError::FrameTooSmall)?
        }

        // Check header
        if buffer[0..3] != PROTO_HEADER[..] {
            Err(FrameError::InvalidHeader)?
        }

        // Check protocol version
        let version = buffer[3];
        if version != PROTO_VERSION {
            Err(FrameError::VersionMismatch(version))?
        }

        // Check message type
        let message = FrameMessage::from_u8(buffer[4])
            .ok_or_else(|| FrameError::InvalidMessage(buffer[4]))?;

        let payload_length = u16::from_be_bytes([buffer[5], buffer[6]]) as usize;
        if payload_length > MAX_PAYLOAD_SIZE {
            Err(FrameError::ExcessivePayloadLength(payload_length))?
        }

        // Check padding
        if buffer[7..10] != [0u8; 3] {
            Err(FrameError::InvalidPadding)?
        }

        Ok(Self::new(message, payload_length))
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[..]
    }
}

pub struct Session {
    flags: u8,
    name: String,
}

impl Session {
    pub const MODE_CONTROL: u8 = 0b0000_0010;
    pub const MODE_FAILSAFE: u8 = 0b0001_0000;

    pub fn new(mode: u8, name: String) -> Self {
        Self {
            flags: mode,
            name: name.chars().take(64).collect::<String>(),
        }
    }

    #[inline]
    pub fn is_control(&self) -> bool {
        self.flags & Self::MODE_CONTROL != 0
    }

    #[inline]
    pub fn is_failsafe(&self) -> bool {
        self.flags & Self::MODE_FAILSAFE != 0
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<Vec<u8>> for Session {
    type Error = FrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let flags = buffer[0];

        let mask = 0b11100000;
        if (flags & mask) != 0 {
            Err(FrameError::InvalidSessionFlags)?
        }

        // TODO: Announce session name length

        Ok(Self::new(
            flags,
            String::from_utf8_lossy(&buffer[1..]).into_owned(),
        ))
    }
}

impl super::Packetize for Session {
    const MESSAGE: FrameMessage = FrameMessage::Session;

    // TODO: Whenever we have a string, we should send its length first
    fn to_bytes(&self) -> Vec<u8> {
        let name_bytes = self.name.as_bytes();

        let mut buf = BytesMut::with_capacity(1 + name_bytes.len());

        buf.put_u8(self.flags);
        buf.put(name_bytes);

        buf.to_vec()
    }
}

// pub enum Error {
//     InvalidRequest = 0x1,
// }

// impl TryFrom<Vec<u8>> for Error {
//     type Error = FrameError;

//     fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
//         match buffer[0] {
//             0x1 => Ok(Self::InvalidRequest),
//             _ => Err(FrameError::InvalidMessage(err)),
//         }
//     }
// }

// impl super::Packetize for Error {
//     const MESSAGE: FrameMessage = FrameMessage::Request;
//     const MESSAGE_SIZE: Option<usize> = Some(1);

//     fn to_bytes(&self) -> Vec<u8> {
//         vec![*self as u8]
//     }
// }

pub struct Request {
    message: FrameMessage,
}

impl Request {
    pub fn new(message: FrameMessage) -> Self {
        Self { message }
    }

    #[inline]
    pub fn message(&self) -> &FrameMessage {
        &self.message
    }
}

impl TryFrom<Vec<u8>> for Request {
    type Error = FrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        FrameMessage::from_u8(buffer[0])
            .ok_or_else(|| FrameError::InvalidMessage(buffer[0]))
            .map(Self::new)
    }
}

impl super::Packetize for Request {
    const MESSAGE: FrameMessage = FrameMessage::Request;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<u8>());

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.message.to_u8()]
    }
}

pub struct Shutdown;

impl Shutdown {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

impl TryFrom<Vec<u8>> for Shutdown {
    type Error = FrameError;

    fn try_from(_value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl super::Packetize for Shutdown {
    const MESSAGE: FrameMessage = FrameMessage::Shutdown;
    const MESSAGE_SIZE: Option<usize> = Some(0);

    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

pub struct Echo {
    payload: i32,
}

impl Echo {
    pub fn new(payload: i32) -> Self {
        Self { payload }
    }

    #[inline]
    pub fn payload(&self) -> i32 {
        self.payload
    }
}

impl TryFrom<Vec<u8>> for Echo {
    type Error = FrameError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: i32::from_be_bytes([value[0], value[1], value[2], value[3]]),
        })
    }
}

impl super::Packetize for Echo {
    const MESSAGE: FrameMessage = FrameMessage::Echo;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<i32>());

    fn to_bytes(&self) -> Vec<u8> {
        self.payload.to_be_bytes().to_vec()
    }
}
