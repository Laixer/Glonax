use bytes::{BufMut, BytesMut};

use super::{MAX_PAYLOAD_SIZE, PROTO_BUFFER_SIZE, PROTO_HEADER, PROTO_VERSION};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    FrameTooSmall,
    InvalidHeader,
    VersionMismatch(u8),
    InvalidMessage(u8),
    PayloadEmpty,
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
            Self::PayloadEmpty => write!(f, "PayloadEmpty"),
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
            Self::PayloadEmpty => write!(f, "payload empty"),
            Self::ExcessivePayloadLength(len) => write!(f, "excessive payload length: {}", len),
            Self::InvalidPadding => write!(f, "invalid padding"),
            Self::InvalidSessionFlags => write!(f, "invalid session flags"),
        }
    }
}

enum FrameMessage {
    Error = 0x0,
    Echo = 0x1,
    Session = 0x10,
    _Shutdown = 0x11,
    Request = 0x12,
}

#[derive(Debug)]
pub struct Frame {
    buffer: BytesMut,
    pub message: u8,
    pub payload_length: usize,
}

impl Frame {
    pub fn new(message: u8, payload_length: usize) -> Self {
        let mut buffer = BytesMut::with_capacity(PROTO_BUFFER_SIZE + payload_length);

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
        PROTO_BUFFER_SIZE..PROTO_BUFFER_SIZE + self.payload_length
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = FrameError;

    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        if buffer.len() != PROTO_BUFFER_SIZE {
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
        if payload_length == 0 {
            Err(FrameError::PayloadEmpty)?
        }

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

#[derive(Debug)]
pub struct Session {
    flags: u8,
    name: String,
}

impl Session {
    pub const MODE_STREAM: u8 = 0b0000_0001;
    pub const MODE_CONTROL: u8 = 0b0000_0010;
    pub const MODE_COMMAND: u8 = 0b0000_0100;
    pub const MODE_FAILSAFE: u8 = 0b0001_0000;

    // TODO: Convert mode to enum
    pub fn new(mode: u8, name: String) -> Self {
        Self {
            flags: mode,
            name: name.chars().take(64).collect::<String>(),
        }
    }

    #[inline]
    pub fn is_stream(&self) -> bool {
        self.flags & Self::MODE_STREAM != 0
    }

    #[inline]
    pub fn is_control(&self) -> bool {
        self.flags & Self::MODE_CONTROL != 0
    }

    #[inline]
    pub fn is_command(&self) -> bool {
        self.flags & Self::MODE_COMMAND != 0
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
        if buffer.is_empty() {
            Err(FrameError::FrameTooSmall)?
        }

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

#[derive(Debug)]
pub enum SessionError {
    UnknownRequest = 0x0,
    UnknownMessage = 0x1,
    UnauthorizedControl = 0x2,
    UnauthorizedCommand = 0x3,
}

impl SessionError {}

impl TryFrom<Vec<u8>> for SessionError {
    type Error = FrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        if buffer.is_empty() {
            Err(FrameError::FrameTooSmall)?
        }

        match buffer[0] {
            0x0 => Ok(Self::UnknownRequest),
            0x1 => Ok(Self::UnknownMessage),
            0x2 => Ok(Self::UnauthorizedControl),
            0x3 => Ok(Self::UnauthorizedCommand),
            _ => Err(FrameError::InvalidMessage(buffer[0])),
        }
    }
}

impl super::Packetize for SessionError {
    const MESSAGE_TYPE: u8 = FrameMessage::Error as u8;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<u8>());

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::UnknownRequest => vec![0x0],
            Self::UnknownMessage => vec![0x1],
            Self::UnauthorizedControl => vec![0x2],
            Self::UnauthorizedCommand => vec![0x3],
        }
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
        if buffer.is_empty() {
            Err(FrameError::FrameTooSmall)?
        }

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

pub struct Echo {
    pub payload: i32,
}

impl TryFrom<Vec<u8>> for Echo {
    type Error = FrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        if buffer.len() != std::mem::size_of::<i32>() {
            Err(FrameError::FrameTooSmall)?
        }

        Ok(Self {
            payload: i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame() {
        let frame = Frame::new(FrameMessage::Echo as u8, 4);
        let bytes = frame.as_ref();

        let frame = Frame::try_from(bytes).unwrap();

        assert_eq!(frame.message, FrameMessage::Echo as u8);
        assert_eq!(frame.payload_length, 4);
    }

    #[test]
    fn test_frame_frame_too_small() {
        let frame = Frame::try_from(&[0x1, 0x2, 0x3][..]);

        assert_eq!(frame.unwrap_err(), FrameError::FrameTooSmall);
    }

    #[test]
    fn test_frame_invalid_header() {
        let frame = Frame::try_from(&[0u8; PROTO_BUFFER_SIZE][..]);

        assert_eq!(frame.unwrap_err(), FrameError::InvalidHeader);
    }

    #[test]
    fn test_frame_version_mismatch() {
        let mut frame = Frame::new(FrameMessage::Echo as u8, 4);
        frame.buffer[3] = 0xff;

        let frame = Frame::try_from(frame.as_ref());

        assert_eq!(frame.unwrap_err(), FrameError::VersionMismatch(0xff));
    }

    #[test]
    fn test_frame_payload_empty() {
        let mut frame = Frame::new(FrameMessage::Echo as u8, 4);
        frame.buffer[5] = 0;
        frame.buffer[6] = 0;

        let frame = Frame::try_from(frame.as_ref());

        assert_eq!(frame.unwrap_err(), FrameError::PayloadEmpty);
    }

    #[test]
    fn test_frame_excessive_payload_length() {
        let frame = Frame::new(FrameMessage::Echo as u8, MAX_PAYLOAD_SIZE + 1);

        let frame = Frame::try_from(frame.as_ref());

        assert_eq!(
            frame.unwrap_err(),
            FrameError::ExcessivePayloadLength(MAX_PAYLOAD_SIZE + 1)
        );
    }

    #[test]
    fn test_frame_invalid_padding() {
        let mut frame = Frame::new(FrameMessage::Echo as u8, 4);
        frame.buffer[7] = 0xff;

        let frame = Frame::try_from(frame.as_ref());

        assert_eq!(frame.unwrap_err(), FrameError::InvalidPadding);
    }

    #[test]
    fn test_session() {
        use crate::protocol::Packetize;

        let session = Session::new(Session::MODE_STREAM, "test".to_string());
        let bytes = session.to_bytes();

        let session = Session::try_from(bytes).unwrap();

        assert!(session.is_stream());
        assert!(!session.is_control());
        assert!(!session.is_command());
        assert!(!session.is_failsafe());
        assert_eq!(session.name(), "test");
    }

    #[test]
    fn test_session_invalid_session_flags() {
        let session = Session::try_from(vec![0b11100000]);

        assert_eq!(session.unwrap_err(), FrameError::InvalidSessionFlags);
    }

    #[test]
    fn test_session_frame_too_small() {
        let session = Session::try_from(Vec::new());

        assert_eq!(session.unwrap_err(), FrameError::FrameTooSmall);
    }
}
