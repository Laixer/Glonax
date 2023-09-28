use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::core::{Instance, Motion, Signal, Status};

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x02;

// const MIN_BUFFER_SIZE: usize = PROTO_HEADER.len()
//     + std::mem::size_of::<u8>()
//     + std::mem::size_of::<u8>()
//     + std::mem::size_of::<u16>()
//     + 3;
const MIN_BUFFER_SIZE: usize = 10;

pub enum Message {
    Null,
    Start(frame::Start),
    Shutdown,
    Motion(Motion),
    Signal(Signal),
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
    }

    impl FrameMessage {
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
            Self {
                buffer: BytesMut::with_capacity(MIN_BUFFER_SIZE + payload_length),
                message,
                payload_length,
            }
        }

        pub fn put(&mut self, payload: &[u8]) {
            self.buffer.put(&PROTO_HEADER[..]);
            self.buffer.put_u8(PROTO_VERSION);
            self.buffer.put_u8(self.message.to_u8());
            self.buffer.put_u16(self.payload_length as u16);
            self.buffer.put(&[0u8; 3][..]);
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

        // TODO: Write name length to buffer
        pub fn to_bytes(&self) -> Vec<u8> {
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

        // TODO: Write name length to buffer
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut buf = BytesMut::with_capacity(1);

            buf.put_u8(self.message.to_u8());

            buf.to_vec()
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
            .handshake(self.flags, session_name.to_string())
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
    pub async fn handshake(
        &mut self,
        mode: u8,
        session_name: impl ToString,
    ) -> std::io::Result<()> {
        self.send_start(mode, session_name).await
    }

    pub async fn send_start(
        &mut self,
        mode: u8,
        session_name: impl ToString,
    ) -> std::io::Result<()> {
        let start = frame::Start::new(mode, session_name.to_string());
        let payload = start.to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Start, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    // TODO: Pass ref
    pub async fn send_instance(&mut self, instance: Instance) -> std::io::Result<()> {
        let payload = instance.to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Instance, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    pub async fn send_status(&mut self, status: &Status) -> std::io::Result<()> {
        let payload = status.to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Status, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        let frame = frame::Frame::new(frame::FrameMessage::Shutdown, 0);

        self.inner.write_all(frame.as_ref()).await
    }

    pub async fn send_request_status(&mut self) -> std::io::Result<()> {
        let payload = frame::Request::new(frame::FrameMessage::Status).to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Request, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    // TODO: Pass ref
    pub async fn send_motion(&mut self, motion: Motion) -> std::io::Result<()> {
        let payload = motion.to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Motion, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    // TODO: Pass ref
    pub async fn send_signal(&mut self, signal: Signal) -> std::io::Result<()> {
        let payload = signal.to_bytes();

        let mut frame = frame::Frame::new(frame::FrameMessage::Signal, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }
}

impl<T: AsyncRead + Unpin> Client<T> {
    pub async fn read_frame(&mut self) -> std::io::Result<frame::Frame> {
        let mut header_buffer = [0u8; MIN_BUFFER_SIZE];

        self.inner.read_exact(&mut header_buffer).await?;

        Ok(frame::Frame::try_from(&header_buffer[..]).unwrap())
    }

    pub async fn request(&mut self, size: usize) -> std::io::Result<frame::Request> {
        let payload_buffer = &mut vec![0u8; size];

        self.inner.read_exact(payload_buffer).await?;

        let message = frame::FrameMessage::from_u8(payload_buffer[0]);
        if message.is_none() {
            log::warn!("Invalid message type {}", payload_buffer[0]);
        }

        return Ok(frame::Request::new(message.unwrap()));
    }

    pub async fn status(&mut self, size: usize) -> std::io::Result<Status> {
        let payload_buffer = &mut vec![0u8; size];

        self.inner.read_exact(payload_buffer).await?;

        Ok(Status::try_from(&payload_buffer[..]).unwrap())
    }

    pub async fn signal(&mut self, size: usize) -> std::io::Result<Signal> {
        let payload_buffer = &mut vec![0u8; size];

        self.inner.read_exact(payload_buffer).await?;

        Ok(Signal::try_from(&payload_buffer[..]).unwrap())
    }

    pub async fn motion(&mut self, size: usize) -> std::io::Result<Motion> {
        let payload_buffer = &mut vec![0u8; size];

        self.inner.read_exact(payload_buffer).await?;

        Ok(Motion::try_from(&payload_buffer[..]).unwrap())
    }

    pub async fn recv_start(&mut self) -> std::io::Result<frame::Start> {
        loop {
            let frame = self.read_frame().await?;

            if frame.message == frame::FrameMessage::Start {
                let payload_buffer = &mut vec![0u8; frame.payload_length];

                self.inner.read_exact(payload_buffer).await?;

                let flags = payload_buffer[0];

                let mut session_name = String::new();

                for c in &payload_buffer[1..] {
                    session_name.push(*c as char);
                }

                return Ok(frame::Start::new(flags, session_name));
            }
        }
    }

    pub async fn recv_status(&mut self) -> std::io::Result<Status> {
        loop {
            let frame = self.read_frame().await?;

            if frame.message == frame::FrameMessage::Status {
                return self.status(frame.payload_length).await;
            }
        }
    }

    pub async fn recv_signal(&mut self) -> std::io::Result<Signal> {
        loop {
            let frame = self.read_frame().await?;

            if frame.message == frame::FrameMessage::Signal {
                return self.signal(frame.payload_length).await;
            }
        }
    }

    pub async fn recv_motion(&mut self) -> std::io::Result<Motion> {
        loop {
            let frame = self.read_frame().await?;

            if frame.message == frame::FrameMessage::Motion {
                return self.motion(frame.payload_length).await;
            }
        }
    }
}
