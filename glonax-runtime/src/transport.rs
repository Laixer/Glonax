use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x02;

// FUTURE: Next protocol version should be 0x03
// - Payload should end with 0x00
// - Vary data should have an explicit length so that we can read them in one go and check against the length
// - After session is established, the host should send an instance message

const MIN_BUFFER_SIZE: usize = PROTO_HEADER.len()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u16>()
    + 3;
const MAX_PAYLOAD_SIZE: usize = 1_024;

const_assert_eq!(MIN_BUFFER_SIZE, 10);
const_assert!(MIN_BUFFER_SIZE < MAX_PAYLOAD_SIZE);
const_assert!(MAX_PAYLOAD_SIZE < 1500);

pub trait Packetize: TryFrom<Vec<u8>> + std::fmt::Display + Sized {
    /// The message type of the packet.
    const MESSAGE: frame::FrameMessage;
    /// If the packet has a fixed size, this should be set to that size.
    const MESSAGE_SIZE: Option<usize> = None;

    /// Convert packet to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

pub mod frame {
    use bytes::{BufMut, BytesMut};

    use super::{MAX_PAYLOAD_SIZE, MIN_BUFFER_SIZE, PROTO_HEADER, PROTO_VERSION};

    pub enum FrameError {
        FrameTooSmall,
        InvalidHeader,
        VersionMismatch(u8),
        InvalidMessage(u8),
        ExcessivePayloadLength(usize),
        InvalidPadding,
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
            }
        }
    }

    // TODO: Split connection messages and data messages
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum FrameMessage {
        Null = 0x1,
        Start = 0x10,
        Shutdown = 0x11,
        Request = 0x12,
        Instance = 0x15,
        Status = 0x16,
        Motion = 0x20,
        Signal = 0x31,
        Pose = 0x40,
        VMS = 0x41,
        GNSS = 0x42,
        Engine = 0x43,
        Error = 0xFF, // TODO: Only for connection messages and set ID to 0x00
    }

    impl FrameMessage {
        // TODO: TryFrom
        pub fn from_u8(value: u8) -> Option<Self> {
            match value {
                0x1 => Some(Self::Null),
                0x10 => Some(Self::Start),
                0x11 => Some(Self::Shutdown),
                0x12 => Some(Self::Request),
                0x15 => Some(Self::Instance),
                0x16 => Some(Self::Status),
                0x20 => Some(Self::Motion),
                0x31 => Some(Self::Signal),
                0x40 => Some(Self::Pose),
                0x41 => Some(Self::VMS),
                0x42 => Some(Self::GNSS),
                0x43 => Some(Self::Engine),
                0xFF => Some(Self::Error),
                _ => None,
            }
        }

        pub fn to_u8(&self) -> u8 {
            match self {
                Self::Null => 0x1,
                Self::Start => 0x10,
                Self::Shutdown => 0x11,
                Self::Request => 0x12,
                Self::Instance => 0x15,
                Self::Status => 0x16,
                Self::Motion => 0x20,
                Self::Signal => 0x31,
                Self::Pose => 0x40,
                Self::VMS => 0x41,
                Self::GNSS => 0x42,
                Self::Engine => 0x43,
                Self::Error => 0xFF,
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

        pub fn put(&mut self, payload: &[u8]) {
            self.buffer.put(payload);
        }

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

    // TODO: maybe this should be a session?
    pub struct Start {
        flags: u8,
        name: String,
    }

    // TODO: Maybe change 'write' to 'control'?
    impl Start {
        pub const MODE_READ: u8 = 0b0000_0001;
        pub const MODE_WRITE: u8 = 0b0000_0010;
        pub const MODE_FAILSAFE: u8 = 0b0001_0000;

        pub fn new(mode: u8, name: String) -> Self {
            Self {
                flags: mode,
                name: name.chars().take(64).collect::<String>(),
            }
        }

        #[inline]
        pub fn is_read(&self) -> bool {
            self.flags & Self::MODE_READ != 0
        }

        #[inline]
        pub fn is_write(&self) -> bool {
            self.flags & Self::MODE_WRITE != 0
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

    impl TryFrom<Vec<u8>> for Start {
        type Error = FrameError;

        fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
            // TODO: Check that the buffer is not empty
            // TODO: Check all other bits are zero
            let flags = buffer[0];

            // TODO: Announce session name length

            Ok(Self::new(
                flags,
                String::from_utf8_lossy(&buffer[1..]).into_owned(),
            ))
        }
    }

    impl super::Packetize for Start {
        const MESSAGE: FrameMessage = FrameMessage::Start;

        // TODO: Whenever we have a string, we should send its length first
        fn to_bytes(&self) -> Vec<u8> {
            let name_bytes = self.name.as_bytes();

            let mut buf = BytesMut::with_capacity(1 + name_bytes.len());

            buf.put_u8(self.flags);
            buf.put(name_bytes);

            buf.to_vec()
        }
    }

    impl std::fmt::Display for Start {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} ({}{}{})",
                self.name,
                if self.is_read() { "R" } else { "" },
                if self.is_write() { "W" } else { "" },
                if self.is_failsafe() { "F" } else { "" },
            )
        }
    }

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
        const MESSAGE_SIZE: Option<usize> = Some(1);

        fn to_bytes(&self) -> Vec<u8> {
            vec![self.message.to_u8()]
        }
    }

    impl std::fmt::Display for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.message)
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

    impl std::fmt::Display for Shutdown {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "")
        }
    }

    // TODO: Replace by Echo. This should be a packet that is sent back to the client with the
    // same payload.
    pub struct Null;

    impl Null {
        pub fn to_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    impl TryFrom<Vec<u8>> for Null {
        type Error = FrameError;

        fn try_from(_value: Vec<u8>) -> Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl super::Packetize for Null {
        const MESSAGE: FrameMessage = FrameMessage::Null;
        const MESSAGE_SIZE: Option<usize> = Some(0);

        fn to_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    impl std::fmt::Display for Null {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "")
        }
    }
}

pub struct ConnectionOptions {
    flags: u8,
}

impl ConnectionOptions {
    // TOOD: Remove this
    pub fn new() -> Self {
        Self {
            flags: frame::Start::MODE_READ,
        }
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        if write {
            self.flags |= frame::Start::MODE_WRITE;
        } else {
            self.flags &= !frame::Start::MODE_WRITE;
        }

        self
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        if read {
            self.flags |= frame::Start::MODE_READ;
        } else {
            self.flags &= !frame::Start::MODE_READ;
        }

        self
    }

    pub fn failsafe(&mut self, failsafe: bool) -> &mut Self {
        if failsafe {
            self.flags |= frame::Start::MODE_FAILSAFE;
        } else {
            self.flags &= !frame::Start::MODE_FAILSAFE;
        }

        self
    }

    pub async fn connect(
        &self,
        address: impl tokio::net::ToSocketAddrs,
        session_name: impl ToString,
    ) -> std::io::Result<Client<tokio::net::TcpStream>> {
        let stream = tokio::net::TcpStream::connect(address).await?;

        let mut client = Client::new(stream);

        client
            .send_packet(&frame::Start::new(self.flags, session_name.to_string()))
            .await?;

        Ok(client)
    }
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Client<T> {
    inner: T,
}

impl Client<tokio::net::TcpStream> {
    pub async fn connect(
        address: impl tokio::net::ToSocketAddrs,
        session_name: impl ToString,
    ) -> std::io::Result<Self> {
        let client = ConnectionOptions::new()
            .connect(address, session_name)
            .await?;

        Ok(client)
    }
}

impl Client<tokio::fs::File> {
    pub async fn open_write(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = tokio::fs::OpenOptions::new().write(true).open(path).await?;

        Ok(Self::new(file))
    }

    pub async fn open_read(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = tokio::fs::OpenOptions::new().read(true).open(path).await?;

        Ok(Self::new(file))
    }
}

impl<T> Client<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: AsyncWrite + Unpin> Client<T> {
    pub async fn send_packet<P: Packetize>(&mut self, packet: &P) -> std::io::Result<()> {
        let payload = packet.to_bytes();

        let mut frame = frame::Frame::new(P::MESSAGE, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    #[inline]
    pub async fn send_request(
        &mut self,
        frame_message: frame::FrameMessage,
    ) -> std::io::Result<()> {
        self.send_packet(&frame::Request::new(frame_message)).await
    }
}

impl<T: AsyncRead + Unpin> Client<T> {
    pub async fn read_frame(&mut self) -> std::io::Result<frame::Frame> {
        let mut header_buffer = [0u8; MIN_BUFFER_SIZE];

        self.inner.read_exact(&mut header_buffer).await?;

        frame::Frame::try_from(&header_buffer[..]).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse frame: {}", e),
            )
        })
    }

    pub async fn packet<P: Packetize>(&mut self, size: usize) -> std::io::Result<P> {
        if P::MESSAGE_SIZE.is_some() && size != P::MESSAGE_SIZE.unwrap() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid packet size: expected {}, got {}",
                    P::MESSAGE_SIZE.unwrap(),
                    size
                ),
            ));
        }

        let buffer = {
            let payload_buffer = &mut vec![0u8; size];

            self.inner.read_exact(payload_buffer).await?;

            payload_buffer.clone()
        };

        P::try_from(buffer).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse packet")
        })
    }
}
