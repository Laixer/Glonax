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

/// A packet that can be sent over the network.
///
/// This trait is implemented for all packets that can be sent over the network.
pub trait Packetize: TryFrom<Vec<u8>> + Sized {
    /// The message type of the packet.
    const MESSAGE_TYPE: u8;
    /// If the packet has a fixed size, this should be set to that size.
    const MESSAGE_SIZE: Option<usize> = None;

    /// Convert packet to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

pub mod client {
    pub mod tcp {
        use crate::protocol::{frame, Stream};

        pub async fn connect(
            address: impl tokio::net::ToSocketAddrs,
            session_name: impl ToString,
        ) -> std::io::Result<(Stream<tokio::net::TcpStream>, crate::core::Instance)> {
            connect_with(address, session_name, false, false).await
        }

        pub async fn connect_with(
            address: impl tokio::net::ToSocketAddrs,
            session_name: impl ToString,
            control: bool,
            failsafe: bool,
        ) -> std::io::Result<(Stream<tokio::net::TcpStream>, crate::core::Instance)> {
            let mut flags = 0;

            if control {
                flags |= frame::Session::MODE_CONTROL;
            } else {
                flags &= !frame::Session::MODE_CONTROL;
            }

            if failsafe {
                flags |= frame::Session::MODE_FAILSAFE;
            } else {
                flags &= !frame::Session::MODE_FAILSAFE;
            }

            let mut client = Stream::new(tokio::net::TcpStream::connect(address).await?);

            let instance = client.handshake(session_name, flags).await?;

            Ok((client, instance))
        }
    }

    pub mod unix {
        use crate::protocol::{frame, Stream};

        pub async fn connect(
            path: impl AsRef<std::path::Path>,
            session_name: impl ToString,
        ) -> std::io::Result<(Stream<tokio::net::UnixStream>, crate::core::Instance)> {
            connect_with(path, session_name, false, false).await
        }

        pub async fn connect_with(
            path: impl AsRef<std::path::Path>,
            session_name: impl ToString,
            control: bool,
            failsafe: bool,
        ) -> std::io::Result<(Stream<tokio::net::UnixStream>, crate::core::Instance)> {
            let mut flags = 0;

            if control {
                flags |= frame::Session::MODE_CONTROL;
            } else {
                flags &= !frame::Session::MODE_CONTROL;
            }

            if failsafe {
                flags |= frame::Session::MODE_FAILSAFE;
            } else {
                flags &= !frame::Session::MODE_FAILSAFE;
            }

            let mut client = Stream::new(tokio::net::UnixStream::connect(path).await?);

            let instance = client.handshake(session_name, flags).await?;

            Ok((client, instance))
        }
    }
}

pub struct Stream<T> {
    inner: T,
}

impl<T> Stream<T> {
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

impl<T: AsyncWrite + AsyncRead + Unpin> Stream<T> {
    pub async fn handshake(
        &mut self,
        session_name: impl ToString,
        flags: u8,
    ) -> std::io::Result<crate::core::Instance> {
        let random_number = rand::random::<i32>();

        self.send_packet(&frame::Echo {
            payload: random_number,
        })
        .await?;

        let frame = self.read_frame().await?;
        if frame.message != frame::Echo::MESSAGE_TYPE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid response from server",
            ));
        }

        let echo = self
            .recv_packet::<frame::Echo>(frame.payload_length)
            .await?;

        if random_number != echo.payload {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid echo response from server",
            ));
        }

        self.send_packet(&frame::Session::new(flags, session_name.to_string()))
            .await?;

        let frame = self.read_frame().await?;
        if frame.message != crate::core::Instance::MESSAGE_TYPE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid response from server",
            ));
        }

        let instance = self
            .recv_packet::<crate::core::Instance>(frame.payload_length)
            .await?;

        Ok(instance)
    }
}

impl<T: AsyncWrite + Unpin> Stream<T> {
    pub async fn send_packet<P: Packetize>(&mut self, packet: &P) -> std::io::Result<()> {
        let payload = packet.to_bytes();

        let mut frame = frame::Frame::new(P::MESSAGE_TYPE, payload.len());
        frame.put(&payload[..]);

        self.inner.write_all(frame.as_ref()).await
    }

    #[inline]
    pub async fn send_request(&mut self, frame_message: u8) -> std::io::Result<()> {
        self.send_packet(&frame::Request::new(frame_message)).await
    }

    // TODO: Maybe add send_shutdown()?
}

impl<T: AsyncRead + Unpin> Stream<T> {
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
