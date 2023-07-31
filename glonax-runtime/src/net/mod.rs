use std::io;

use glonax_j1939::*;

pub use actuator::*;
pub use encoder::*;
pub use engine::*;
pub use host::*;
pub use service::*;

mod actuator;
mod encoder;
mod engine;
mod host;
mod service;

// TODO: Implement connection management.
pub struct J1939Network(J1939Stream);

// TODO: Maybe rename to J1939Application?
impl J1939Network {
    pub fn new(ifname: &str, addr: u8) -> io::Result<Self> {
        let stream = J1939Stream::bind(ifname, addr)?;
        stream.set_broadcast(true)?;

        Ok(Self(stream))
    }

    /// Set the promiscuous mode.
    #[inline]
    pub fn set_promisc_mode(&self, on: bool) -> io::Result<()> {
        self.0.set_promisc_mode(on)
    }

    /// Accept a frame.
    #[inline]
    pub async fn accept(&self) -> io::Result<Frame> {
        self.0.read().await
    }

    /// Request a PGN message.
    #[inline]
    pub async fn request(&self, node: u8, pgn: PGN) {
        self.0.write(&protocol::request(node, pgn)).await.unwrap();
    }

    /// Assign address to node.
    pub async fn commanded_address(&self, node: u8, address: u8) {
        let data = vec![0x18, 0xA4, 0x49, 0x24, 0x11, 0x05, 0x06, 0x85, address];

        self.send_vectored(&Self::broadcast_announce(
            node,
            PGN::CommandedAddress,
            &data,
        ))
        .await
        .unwrap();
    }

    /// Broadcast Announce Message.
    fn broadcast_announce(node: u8, pgn: PGN, data: &[u8]) -> Vec<Frame> {
        let data_length = (data.len() as u16).to_le_bytes();
        let packets = (data.len() as f32 / 8.0).ceil() as u8;
        let byte_array = pgn.to_le_bytes();

        let mut frames = vec![];

        let connection_frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
                .priority(7)
                .da(node)
                .build(),
        )
        .copy_from_slice(&[
            0x20,
            data_length[0],
            data_length[1],
            packets,
            0xff,
            byte_array[0],
            byte_array[1],
            byte_array[2],
        ])
        .build();

        frames.push(connection_frame);

        for (packet, data_chunk) in data.chunks(7).enumerate() {
            let packet = packet as u8 + 1;

            let mut frame_builder = FrameBuilder::new(
                IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                    .priority(7)
                    .da(node)
                    .build(),
            );

            let payload = frame_builder.as_mut();
            payload[0] = packet;

            if data_chunk.len() == 7 {
                payload[1..8].copy_from_slice(data_chunk);
            } else {
                payload[1..(data_chunk.len() + 1)].copy_from_slice(data_chunk);
            }

            frames.push(frame_builder.set_len(8).build());
        }

        frames
    }

    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.0.write(frame).await
    }

    #[inline]
    pub async fn send_vectored(&self, frames: &Vec<Frame>) -> io::Result<Vec<usize>> {
        let mut v = vec![];
        for frame in frames {
            v.push(self.0.write(frame).await?);
        }
        Ok(v)
    }
}

pub trait Parsable<T>: Send + Sync {
    /// Parse a frame.
    ///
    /// Returns `None` if the frame is not parsable. Returns `Some(T)` if the frame is parsable
    /// and the message is successfully parsed and returned.
    fn parse(&mut self, frame: &Frame) -> Option<T>;
}

pub struct Router {
    /// The network.
    net: Vec<J1939Network>,
    /// The current frame.
    frame: Option<Frame>,
    /// The PGN filter.
    filter_pgn: Vec<u32>,
    /// The node filter.
    filter_node: Vec<u8>,
}

impl FromIterator<J1939Network> for Router {
    /// Create a router from an iterator.
    fn from_iter<T: IntoIterator<Item = J1939Network>>(iter: T) -> Self {
        Self {
            net: Vec::from_iter(iter),
            frame: None,
            filter_pgn: vec![],
            filter_node: vec![],
        }
    }
}

impl Router {
    /// Construct a new router.
    pub fn new(net: J1939Network) -> Self {
        Self {
            net: vec![net],
            frame: None,
            filter_pgn: vec![],
            filter_node: vec![],
        }
    }

    /// Add a filter based on PGN.
    #[inline]
    pub fn add_pgn_filter(&mut self, pgn: u32) {
        self.filter_pgn.push(pgn);
    }

    /// Add a filter based on node id.
    #[inline]
    pub fn add_node_filter(&mut self, node: u8) {
        self.filter_node.push(node);
    }

    /// Return the current frame source.
    #[inline]
    pub fn frame_source(&self) -> Option<u8> {
        self.frame.map(|f| f.id().sa())
    }

    /// Take the frame from the router.
    #[inline]
    pub fn take(&mut self) -> Option<Frame> {
        self.frame.take()
    }

    /// Listen for incoming packets.
    pub async fn listen(&mut self) -> io::Result<()> {
        loop {
            let frame = self.net.first().unwrap().accept().await?;

            let node_address = frame.id().sa();

            if !self.filter_pgn.is_empty() {
                let pgn = frame.id().pgn_raw();
                if !self.filter_pgn.contains(&pgn) {
                    continue;
                }
            }

            if !self.filter_node.is_empty() && !self.filter_node.contains(&node_address) {
                continue;
            }

            self.frame = Some(frame);
            break;
        }

        Ok(())
    }

    /// Try to accept a frame and parse it.
    ///
    /// This method will return `None` if the frame is not accepted. Otherwise, it will return
    /// `Some` with the resulting message.
    pub fn try_accept<T>(&self, service: &mut impl Parsable<T>) -> Option<T> {
        self.frame.and_then(|frame| service.parse(&frame))
    }
}
