use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::core::{Motion, Signal};

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x01;

const PROTO_MESSAGE_NULL: u8 = 0x1;
const PROTO_MESSAGE_START: u8 = 0x10;
const PROTO_MESSAGE_SHUTDOWN: u8 = 0x11;
const PROTO_MESSAGE_INSTANCE: u8 = 0x15;

const PROTO_MESSAGE_MOTION: u8 = 0x20;
const PROTO_MESSAGE_SIGNAL: u8 = 0x31;

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
            use bytes::{BufMut, BytesMut};

            let name_bytes = self.name.as_bytes();

            let mut buf = BytesMut::with_capacity(1 + name_bytes.len());

            buf.put_u8(self.flags);
            buf.put(name_bytes);

            buf.to_vec()
        }
    }

    pub struct ProxyService {
        /// Instance unique identifier.
        instance: String,
        /// Instance model.
        model: String,
        /// Instance name.
        name: String,
    }

    impl ProxyService {
        pub fn new(instance: String, model: String, name: String) -> Self {
            Self {
                instance,
                model,
                name,
            }
        }

        #[inline]
        pub fn instance(&self) -> &str {
            &self.instance
        }

        #[inline]
        pub fn model(&self) -> &str {
            &self.model
        }

        #[inline]
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            use bytes::{BufMut, BytesMut};

            let instance_bytes = self.instance.as_bytes();
            let model_bytes = self.model.as_bytes();
            let name_bytes = self.name.as_bytes();

            let mut buf = BytesMut::with_capacity(
                2 + instance_bytes.len() + model_bytes.len() + name_bytes.len(),
            );

            buf.put_u16(instance_bytes.len() as u16);
            buf.put(instance_bytes);
            buf.put_u16(model_bytes.len() as u16);
            buf.put(model_bytes);
            buf.put_u16(name_bytes.len() as u16);
            buf.put(name_bytes);

            buf.to_vec()
        }
    }
}

pub struct Protocol<T> {
    inner: T,
}

impl<T> Protocol<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    fn build_frame<'a>(
        &'a mut self,
        buffer: &'a mut BytesMut,
        message: u8,
        payload: &[u8],
    ) -> &'a [u8] {
        buffer.put(&PROTO_HEADER[..]);
        buffer.put_u8(PROTO_VERSION);
        buffer.put_u8(message);
        buffer.put_u16(payload.len() as u16);
        buffer.put(&[0u8; 3][..]);
        buffer.put(&payload[..]);

        &buffer[..]
    }
}

impl<T: AsyncWrite + Unpin> Protocol<T> {
    pub async fn write_frame0(&mut self, start: frame::Start) -> std::io::Result<()> {
        let payload = start.to_bytes();

        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

        self.build_frame(&mut buffer, PROTO_MESSAGE_START, &payload);

        assert_eq!(buffer.len(), MIN_BUFFER_SIZE + payload.len());

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_frame1(&mut self) -> std::io::Result<()> {
        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE);

        self.build_frame(&mut buffer, PROTO_MESSAGE_SHUTDOWN, &[]);

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_frame2(&mut self, instance: frame::ProxyService) -> std::io::Result<()> {
        let payload = instance.to_bytes();

        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

        self.build_frame(&mut buffer, PROTO_MESSAGE_INSTANCE, &payload);

        assert_eq!(buffer.len(), MIN_BUFFER_SIZE + payload.len());

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_frame5(&mut self, motion: Motion) -> std::io::Result<()> {
        let payload = motion.to_bytes();

        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

        self.build_frame(&mut buffer, PROTO_MESSAGE_MOTION, &payload);

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_frame6(&mut self, signal: Signal) -> std::io::Result<()> {
        let payload = signal.to_bytes();

        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

        self.build_frame(&mut buffer, PROTO_MESSAGE_SIGNAL, &payload);

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_all6(&mut self, signals: Vec<Signal>) -> std::io::Result<()> {
        for signal in signals {
            let payload = signal.to_bytes();

            let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

            self.build_frame(&mut buffer, PROTO_MESSAGE_SIGNAL, &payload);

            self.inner.write_all(&buffer[..]).await?;
        }

        Ok(())
    }
}

impl<T: AsyncRead + Unpin> Protocol<T> {
    pub async fn read_frame(&mut self) -> std::io::Result<Message> {
        loop {
            let mut header_buffer = [0u8; MIN_BUFFER_SIZE];

            self.inner.read_exact(&mut header_buffer).await?;

            // Check header
            if &header_buffer[0..3] != &PROTO_HEADER[..] {
                log::warn!("Invalid header");
                continue;
            }

            // Check protocol version
            let version = header_buffer[3];
            if version != PROTO_VERSION {
                log::warn!("Invalid version {}", version);
                continue;
            }

            let message = header_buffer[4];

            let proto_length = u16::from_be_bytes([header_buffer[5], header_buffer[6]]) as usize;
            if proto_length > 4096 {
                log::warn!("Invalid proto length {}", proto_length);
                continue;
            }

            // Check padding
            if &header_buffer[7..10] != &[0u8; 3] {
                log::warn!("Invalid padding");
                continue;
            }

            let payload_buffer = &mut vec![0u8; proto_length];

            self.inner.read_exact(payload_buffer).await?;

            if message == PROTO_MESSAGE_NULL {
                return Ok(Message::Null);
            } else if message == PROTO_MESSAGE_START {
                let flags = payload_buffer[0];

                let mut session_name = String::new();

                for c in &payload_buffer[1..] {
                    session_name.push(*c as char);
                }

                return Ok(Message::Start(frame::Start::new(flags, session_name)));
            } else if message == PROTO_MESSAGE_SHUTDOWN {
                return Ok(Message::Shutdown);
            } else if message == PROTO_MESSAGE_MOTION {
                match Motion::try_from(&payload_buffer[..]) {
                    Ok(motion) => {
                        return Ok(Message::Motion(motion));
                    }
                    Err(_) => {
                        log::warn!("Invalid motion payload");
                        continue;
                    }
                }
            } else if message == PROTO_MESSAGE_SIGNAL {
                match Signal::try_from(&payload_buffer[..]) {
                    Ok(signal) => {
                        return Ok(Message::Signal(signal));
                    }
                    Err(_) => {
                        log::warn!("Invalid signal payload");
                        continue;
                    }
                }
            } else {
                log::error!("Invalid message type: {}", message);
            }
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
    inner: Protocol<T>,
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

    pub fn into_split(
        self,
    ) -> (
        Client<tokio::net::tcp::OwnedReadHalf>,
        Client<tokio::net::tcp::OwnedWriteHalf>,
    ) {
        let (r, w) = tokio::net::TcpStream::into_split(self.inner.inner);

        (Client::new(r), Client::new(w))
    }
}

impl<T> Client<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Protocol::new(inner),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner.inner
    }
}

impl<T: AsyncWrite + Unpin> Client<T> {
    pub async fn handshake(
        &mut self,
        mode: u8,
        session_name: impl ToString,
    ) -> std::io::Result<()> {
        let start = frame::Start::new(mode, session_name.to_string());
        self.inner.write_frame0(start).await
    }

    pub async fn send_start(
        &mut self,
        mode: u8,
        session_name: impl ToString,
    ) -> std::io::Result<()> {
        let start = frame::Start::new(mode, session_name.to_string());
        self.inner.write_frame0(start).await
    }

    pub async fn send_instance(
        &mut self,
        id: String,
        model: String,
        name: String,
    ) -> std::io::Result<()> {
        let instance = frame::ProxyService::new(id, model, name);

        self.inner.write_frame2(instance).await
    }

    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        self.inner.write_frame1().await
    }

    pub async fn send_motion(&mut self, motion: Motion) -> std::io::Result<()> {
        self.inner.write_frame5(motion).await
    }

    pub async fn send_signal(&mut self, signal: Signal) -> std::io::Result<()> {
        self.inner.write_frame6(signal).await
    }
}

impl<T: AsyncRead + Unpin> Client<T> {
    pub async fn read_frame(&mut self) -> std::io::Result<Message> {
        self.inner.read_frame().await
    }

    pub async fn recv_start(&mut self) -> std::io::Result<frame::Start> {
        loop {
            if let Message::Start(session) = self.inner.read_frame().await? {
                return Ok(session);
            }
        }
    }

    pub async fn recv_signal(&mut self) -> std::io::Result<Signal> {
        loop {
            if let Message::Signal(signal) = self.inner.read_frame().await? {
                return Ok(signal);
            }
        }
    }

    pub async fn recv_motion(&mut self) -> std::io::Result<Motion> {
        loop {
            if let Message::Motion(motion) = self.inner.read_frame().await? {
                return Ok(motion);
            }
        }
    }
}
