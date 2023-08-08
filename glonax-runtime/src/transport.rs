use bytes::{Buf, BufMut, BytesMut};
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
    buffer: BytesMut,
}

impl<T> Protocol<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            buffer: BytesMut::with_capacity(2048),
        }
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
    // TODO: Why not return UnexpectedEof?
    // TODO: This maybe too complex
    async fn read_at_least(&mut self, min_len: usize) -> std::io::Result<Option<usize>> {
        let mut total_bytes_read = None;
        while self.buffer.len() < min_len {
            let bytes_read = self.inner.read_buf(&mut self.buffer).await?;
            if bytes_read == 0 {
                return Ok(Some(0));
            }
            total_bytes_read = Some(bytes_read + total_bytes_read.unwrap_or(0));
        }

        Ok(total_bytes_read)
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Message> {
        loop {
            let bytes_read = self.read_at_least(MIN_BUFFER_SIZE).await.unwrap();
            if let Some(bytes_read) = bytes_read {
                if bytes_read == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "EOF",
                    ));
                }
            }

            // Find header
            for i in 0..(self.buffer.len() - PROTO_HEADER.len()) {
                if self.buffer[i] == PROTO_HEADER[0] {
                    if self.buffer[i + 1] == PROTO_HEADER[1] {
                        if self.buffer[i + 2] == PROTO_HEADER[2] {
                            self.buffer.advance(i + PROTO_HEADER.len());
                            break;
                        }
                    }
                }
            }

            // Check protocol version
            let version = self.buffer.get_u8();
            if version != PROTO_VERSION {
                log::warn!("Invalid version {}", version);
                continue;
            }

            let message = self.buffer.get_u8();

            let proto_length = self.buffer.get_u16() as usize;
            if proto_length > 4096 {
                log::warn!("Invalid proto length {}", proto_length);
                continue;
            }

            let bytes_read = self.read_at_least(proto_length).await.unwrap();
            if let Some(bytes_read) = bytes_read {
                if bytes_read == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "EOF",
                    ));
                }
            }

            if message == PROTO_MESSAGE_NULL {
                return Ok(Message::Null);
            } else if message == PROTO_MESSAGE_START {
                let payload = self.buffer.split_to(proto_length);

                let mut session_name = String::new();

                for c in payload {
                    session_name.push(c as char);
                }

                return Ok(Message::Start(frame::Start::new(session_name)));
            } else if message == PROTO_MESSAGE_SHUTDOWN {
                return Ok(Message::Shutdown);
            } else if message == PROTO_MESSAGE_MOTION {
                let payload = self.buffer.split_to(proto_length);

                match Motion::try_from(&payload[..]) {
                    Ok(motion) => {
                        return Ok(Message::Motion(motion));
                    }
                    Err(_) => {
                        log::warn!("Invalid motion payload");
                        continue;
                    }
                }
            } else if message == PROTO_MESSAGE_SIGNAL {
                let payload = self.buffer.split_to(proto_length);

                match Signal::try_from(&payload[..]) {
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
