use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x02;

// const MIN_BUFFER_SIZE: usize = PROTO_HEADER.len()
//     + std::mem::size_of::<u8>()
//     + std::mem::size_of::<u8>()
//     + std::mem::size_of::<u16>()
//     + 3;
const MIN_BUFFER_SIZE: usize = 10;

// TODO: Add Display
pub trait Packetize: TryFrom<Vec<u8>> {
    const MESSAGE: frame::FrameMessage;

    /// Convert packet to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

pub mod frame {
    use bytes::{BufMut, BytesMut};

    use super::{MIN_BUFFER_SIZE, PROTO_HEADER, PROTO_VERSION};

    #[derive(Debug, PartialEq, Eq)]
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
            self.buffer.put(&payload[..]);
        }

        pub fn payload_range(&self) -> std::ops::Range<usize> {
            MIN_BUFFER_SIZE..MIN_BUFFER_SIZE + self.payload_length
        }
    }

    impl TryFrom<&[u8]> for Frame {
        type Error = ();

        fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
            if buffer.len() < MIN_BUFFER_SIZE {
                log::warn!("Invalid buffer size");
                return Err(());
            }

            // Check header
            if &buffer[0..3] != &PROTO_HEADER[..] {
                log::warn!("Invalid header");
                return Err(());
            }

            // Check protocol version
            let version = buffer[3];
            if version != PROTO_VERSION {
                log::warn!("Invalid version {}", version);
                return Err(());
            }

            let message = FrameMessage::from_u8(buffer[4]);
            if message.is_none() {
                log::warn!("Invalid message type {}", buffer[4]);
                return Err(());
            }

            let payload_length = u16::from_be_bytes([buffer[5], buffer[6]]) as usize;
            if payload_length > 4096 {
                log::warn!("Invalid proto length {}", payload_length);
                return Err(());
            }

            // Check padding
            if &buffer[7..10] != &[0u8; 3] {
                log::warn!("Invalid padding");
                return Err(());
            }

            Ok(Self::new(message.unwrap(), payload_length))
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

    impl Start {
        pub const MODE_READ: u8 = 0b0000_0001;
        pub const MODE_WRITE: u8 = 0b0000_0010;
        pub const MODE_FAILSAFE: u8 = 0b0001_0000;

        pub fn new(mode: u8, name: String) -> Self {
            Self { flags: mode, name }
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
        type Error = ();

        fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
            let flags = buffer[0];

            let mut session_name = String::new();

            for c in &buffer[1..] {
                session_name.push(*c as char);
            }

            Ok(Self::new(flags, session_name))
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
        type Error = ();

        fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
            if buffer.len() != 1 {
                log::warn!("Invalid buffer size");
                return Err(());
            }

            let message = FrameMessage::from_u8(buffer[0]);
            if message.is_none() {
                log::warn!("Invalid message type {}", buffer[0]);
                return Err(());
            }

            Ok(Self::new(message.unwrap()))
        }
    }

    impl super::Packetize for Request {
        const MESSAGE: FrameMessage = FrameMessage::Request;

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
        type Error = ();

        fn try_from(_value: Vec<u8>) -> Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl super::Packetize for Shutdown {
        const MESSAGE: FrameMessage = FrameMessage::Shutdown;

        fn to_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    pub struct Null;

    impl Null {
        pub fn to_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    impl TryFrom<Vec<u8>> for Null {
        type Error = ();

        fn try_from(_value: Vec<u8>) -> Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl super::Packetize for Null {
        const MESSAGE: FrameMessage = FrameMessage::Null;

        fn to_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }
}

pub struct ConnectionOptions {
    flags: u8,
}

impl ConnectionOptions {
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

        Ok(frame::Frame::try_from(&header_buffer[..]).unwrap())
    }

    pub async fn packet<P: Packetize>(&mut self, size: usize) -> std::io::Result<P> {
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
