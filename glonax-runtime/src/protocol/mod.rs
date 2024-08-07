use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// TODO: Should not be public
pub mod client;
pub mod frame;

// TODO: Maybe move up
pub use client::{connect, connect_safe, unix_connect, unix_connect_safe};

/// The protocol header.
///
/// This is used to identify the protocol. The header is always the same and is
/// always present at the start of a frame. The bytes shown here are the ASCII
/// representation of the header. This simplifies the process of identifying the
/// protocol and makes it easier to debug.
const PROTO_HEADER: [u8; 3] = [b'L', b'X', b'R'];

/// The protocol version.
///
/// This is used to identify the protocol version. If the version is not the
/// same as the expected version, the frame is considered invalid. The version
/// is only  changed when the protocol is changed in a way that is not backwards
/// compatible. This is done to ensure that the protocol can be changed without
/// breaking existing implementations.
const PROTO_VERSION: u8 = 0x03;

/// The minimum buffer size required to read a frame.
const PROTO_BUFFER_SIZE: usize = PROTO_HEADER.len()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u8>()
    + std::mem::size_of::<u16>()
    + 3;

/// The maximum payload size.
///
/// This is the maximum size of the payload of a frame. The maximum size of a
/// frame is `PROTO_BUFFER_SIZE + MAX_PAYLOAD_SIZE`. The maximum payload size
/// ensures that the maximum frame size is within the maximum MTU of a network.
///
/// The maximum payload size is also used to limit the maximum size of a packet
/// and to reject packets that are too large.
const MAX_PAYLOAD_SIZE: usize = 1_024;

/// A packet that can be sent over the network.
///
/// This trait is implemented for all packets that can be sent over the network.
pub trait Packetize: TryFrom<Vec<u8>> + Sized {
    /// The message type of the packet.
    const MESSAGE_TYPE: u8;
    /// If the packet has a fixed size, this is the size of the packet. If the
    /// packet has a variable size, this is `None`.
    ///
    /// This is used to validate the size of the packet when receiving a packet.
    const MESSAGE_SIZE: Option<usize> = None;

    /// Convert packet to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

pub struct Stream<T> {
    inner: T,
}

impl<T> Stream<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
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
}

impl<T: AsyncRead + Unpin> Stream<T> {
    pub async fn read_frame(&mut self) -> std::io::Result<frame::Frame> {
        let mut header_buffer = [0u8; PROTO_BUFFER_SIZE];

        self.inner.read_exact(&mut header_buffer).await?;

        frame::Frame::try_from(&header_buffer[..]).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse frame: {}", e),
            )
        })
    }

    pub async fn recv_packet<P: Packetize>(&mut self, size: usize) -> std::io::Result<P> {
        if size == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid data, packet size too small: {}", size),
            ));
        }

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

        if size > MAX_PAYLOAD_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid data, packet size too large: {}", size),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_header() {
        assert_eq!(PROTO_HEADER, [b'L', b'X', b'R']);
    }

    #[test]
    fn test_proto_version() {
        assert_eq!(PROTO_VERSION, 0x03);
    }

    #[test]
    fn test_proto_buffer_size() {
        assert_eq!(PROTO_BUFFER_SIZE, 10);
    }
}
