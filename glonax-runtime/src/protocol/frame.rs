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

enum FrameMessage {
    _Error = 0x0,
    Echo = 0x1,
    Session = 0x10,
    Shutdown = 0x11,
    Request = 0x12,
}

pub struct Frame {
    buffer: BytesMut,
    pub message: u8,
    pub payload_length: usize,
}

impl Frame {
    pub fn new(message: u8, payload_length: usize) -> Self {
        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload_length);

        buffer.put(&PROTO_HEADER[..]);
        buffer.put_u8(PROTO_VERSION);
        buffer.put_u8(message);
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

        let payload_length = u16::from_be_bytes([buffer[5], buffer[6]]) as usize;
        if payload_length > MAX_PAYLOAD_SIZE {
            Err(FrameError::ExcessivePayloadLength(payload_length))?
        }

        // Check padding
        if buffer[7..10] != [0u8; 3] {
            Err(FrameError::InvalidPadding)?
        }

        Ok(Self::new(buffer[4], payload_length))
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

    // TODO: Convert mode to enum
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
    const MESSAGE_TYPE: u8 = FrameMessage::Session as u8;

    // TODO: Whenever we have a string, we should send its length first
    fn to_bytes(&self) -> Vec<u8> {
        let name_bytes = self.name.as_bytes();

        let mut buf = BytesMut::with_capacity(1 + name_bytes.len());

        buf.put_u8(self.flags);
        buf.put(name_bytes);

        buf.to_vec()
    }
}

pub struct Request {
    message: u8,
}

impl Request {
    pub fn new(message: u8) -> Self {
        Self { message }
    }

    #[inline]
    pub fn message(&self) -> u8 {
        self.message
    }
}

impl TryFrom<Vec<u8>> for Request {
    type Error = FrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::new(buffer[0]))
    }
}

impl super::Packetize for Request {
    const MESSAGE_TYPE: u8 = FrameMessage::Request as u8;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<u8>());

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.message]
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
    const MESSAGE_TYPE: u8 = FrameMessage::Shutdown as u8;
    const MESSAGE_SIZE: Option<usize> = Some(0);

    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

pub struct Echo {
    pub payload: i32,
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
    const MESSAGE_TYPE: u8 = FrameMessage::Echo as u8;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<i32>());

    fn to_bytes(&self) -> Vec<u8> {
        self.payload.to_be_bytes().to_vec()
    }
}
