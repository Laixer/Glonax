use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// TODO: Should not be public
pub mod frame;

const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];
const PROTO_VERSION: u8 = 0x02;

const MIN_BUFFER_SIZE: usize = PROTO_HEADER.len()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u16>()
    + 3;
const MAX_PAYLOAD_SIZE: usize = 1_024;

const_assert_eq!(MIN_BUFFER_SIZE, 10);
const_assert!(MIN_BUFFER_SIZE < MAX_PAYLOAD_SIZE);
const_assert!(MAX_PAYLOAD_SIZE < 1500);

pub trait Packetize: TryFrom<Vec<u8>> + Sized {
    /// The message type of the packet.
    const MESSAGE_TYPE: u8;
    /// If the packet has a fixed size, this should be set to that size.
    const MESSAGE_SIZE: Option<usize> = None;

    /// Convert packet to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

#[derive(Default)]
pub struct Connection {
    flags: u8,
}

impl Connection {
    pub fn control(&mut self, write: bool) -> &mut Self {
        if write {
            self.flags |= frame::Session::MODE_CONTROL;
        } else {
            self.flags &= !frame::Session::MODE_CONTROL;
        }

        self
    }

    pub fn failsafe(&mut self, failsafe: bool) -> &mut Self {
        if failsafe {
            self.flags |= frame::Session::MODE_FAILSAFE;
        } else {
            self.flags &= !frame::Session::MODE_FAILSAFE;
        }

        self
    }

    pub async fn connect(
        &mut self,
        address: impl tokio::net::ToSocketAddrs,
        session_name: impl ToString,
    ) -> std::io::Result<(Client<tokio::net::TcpStream>, crate::core::Instance)> {
        let stream = tokio::net::TcpStream::connect(address).await?;

        let mut client = Client::new(stream);

        let random_number = rand::random::<i32>();

        client.send_packet(&frame::Echo::new(random_number)).await?;

        let frame = client.read_frame().await?;
        if frame.message != frame::Echo::MESSAGE_TYPE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid response from server",
            ));
        }

        let echo = client
            .recv_packet::<frame::Echo>(frame.payload_length)
            .await?;

        if random_number != echo.payload() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid echo response from server",
            ));
        }

        client
            .send_packet(&frame::Session::new(self.flags, session_name.to_string()))
            .await?;

        let frame = client.read_frame().await?;
        if frame.message != crate::core::Instance::MESSAGE_TYPE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid response from server",
            ));
        }

        let instance = client
            .recv_packet::<crate::core::Instance>(frame.payload_length)
            .await?;

        Ok((client, instance))
    }
}

pub struct Client<T> {
    inner: T,
}

impl Client<std::fs::File> {
    pub fn open_write(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new().write(true).open(path)?;

        Ok(Self::new(file))
    }

    pub fn open_read(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new().read(true).open(path)?;

        Ok(Self::new(file))
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

        let mut frame = frame::Frame::new(P::MESSAGE_TYPE, payload.len());
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

    pub async fn recv_packet<P: Packetize>(&mut self, size: usize) -> std::io::Result<P> {
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
