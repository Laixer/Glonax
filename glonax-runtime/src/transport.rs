use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::core::{Motion, Signal};

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x01;

const PROTO_MESSAGE_NULL: u8 = 0x1;
const PROTO_MESSAGE_START: u8 = 0x10;
const PROTO_MESSAGE_SHUTDOWN: u8 = 0x11;
const PROTO_MESSAGE_MOTION: u8 = 0x20;
const PROTO_MESSAGE_SIGNAL: u8 = 0x31;

const MIN_BUFFER_SIZE: usize = PROTO_HEADER.len()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u16>();

pub enum Message {
    Null,
    Start(frame::Start),
    Shutdown,
    Motion(Motion),
    Signal(Signal),
}

pub mod frame {
    pub struct Start {
        name: String,
    }

    impl Start {
        pub fn new(name: String) -> Self {
            Self { name }
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn to_bytes(&self) -> &[u8] {
            self.name.as_bytes()
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

    // TOOD: Why not use the buffer directly?
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
        buffer.put(&payload[..]);

        &buffer[..]
    }
}

impl<T: AsyncWrite + Unpin> Protocol<T> {
    pub async fn write_frame0(&mut self, start: frame::Start) -> std::io::Result<()> {
        let payload = start.to_bytes();

        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE + payload.len());

        self.build_frame(&mut buffer, PROTO_MESSAGE_START, payload);

        self.inner.write_all(&buffer[..]).await
    }

    pub async fn write_frame1(&mut self) -> std::io::Result<()> {
        let mut buffer = BytesMut::with_capacity(MIN_BUFFER_SIZE);

        self.build_frame(&mut buffer, PROTO_MESSAGE_SHUTDOWN, &[]);

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
            if header_buffer[0] != PROTO_HEADER[0]
                || header_buffer[1] != PROTO_HEADER[1]
                || header_buffer[2] != PROTO_HEADER[2]
            {
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

            let payload_buffer = &mut vec![0u8; proto_length];

            self.inner.read_exact(payload_buffer).await?;

            if message == PROTO_MESSAGE_NULL {
                return Ok(Message::Null);
            } else if message == PROTO_MESSAGE_START {
                let mut session_name = String::new();

                for c in payload_buffer {
                    session_name.push(*c as char);
                }

                return Ok(Message::Start(frame::Start::new(session_name)));
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

pub struct Client<T> {
    inner: Protocol<T>,
}

impl Client<tokio::net::TcpStream> {
    pub async fn connect(address: &String, session_name: impl ToString) -> std::io::Result<Self> {
        let addr = if !address.contains(':') {
            address.to_owned() + ":30051"
        } else {
            address.to_owned()
        };

        let stream = tokio::net::TcpStream::connect(addr).await?;

        let mut this = Self {
            inner: Protocol::new(stream),
        };

        this.handshake(session_name).await?;

        Ok(this)
    }
}

impl<T> Client<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Protocol::new(inner),
        }
    }
}

impl<T: AsyncWrite + Unpin> Client<T> {
    pub async fn handshake(&mut self, session_name: impl ToString) -> std::io::Result<()> {
        let start = frame::Start::new(session_name.to_string());
        self.inner.write_frame0(start).await
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
