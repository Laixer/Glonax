use std::{collections::HashMap, io, time};

use glonax_j1939::*;

pub use actuator::*;
pub use encoder::*;
pub use engine::*;
pub use service::*;

mod actuator;
mod encoder;
mod engine;
mod service;

// TODO: Implement connection management.
// TODO: Implement broadcast message.
pub struct J1939Network(J1939Stream);

impl J1939Network {
    pub fn new(ifname: &str, addr: u8) -> io::Result<Self> {
        let stream = J1939Stream::bind(ifname, addr)?;
        stream.set_broadcast(true)?;

        Ok(Self(stream))
    }

    #[inline]
    pub fn set_promisc_mode(&self, on: bool) -> io::Result<()> {
        self.0.set_promisc_mode(on)
    }

    #[inline]
    pub async fn accept(&self) -> io::Result<Frame> {
        self.0.read().await
    }

    // TODO: Change to Commanded Address
    pub async fn set_address(&self, node: u8, address: u8) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage2)
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', address])
        .build();

        self.0.write(&frame).await.unwrap();
    }

    pub async fn reset(&self, node: u8) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, 0x69])
        .build();

        self.0.write(&frame).await.unwrap();
    }

    /// Request a PGN message.
    pub async fn request(&self, node: u8, pgn: PGN) {
        self.0.write(&protocol::request(node, pgn)).await.unwrap();
    }

    /// Broadcast Announce Message.
    pub async fn broadcast(&self, pgn: u16, data: &[u8]) {
        // Byte D1 Total message size, number of
        // bytes (low byte)
        // Byte D2 Total message size, number of
        // bytes (high byte)

        let tt = (data.len() as u16).to_le_bytes();

        // Byte D3 Total number of packets
        let packets = (data.len() as f32 / 8.0).ceil() as u8;

        // Byte D5 PGN of the packeted message
        // (low byte)
        // Byte D6 PGN of the packeted message
        // (mid byte)
        // Byte D7 PGN of the packeted message
        // (high byte)

        let byte_array = u32::to_be_bytes(pgn as u32);

        let connection_frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
                .priority(7)
                .da(0xff)
                .build(),
        )
        .copy_from_slice(&[
            0x20,
            tt[0],
            tt[1],
            packets,
            0xff,
            byte_array[3],
            byte_array[2],
            byte_array[1],
        ])
        .build();

        println!("Conn: {}", connection_frame);

        let data_frame0 = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                .priority(7)
                .da(0xff)
                .build(),
        )
        .copy_from_slice(&[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        .build();

        println!("Data0: {}", data_frame0);

        let data_frame1 = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                .priority(7)
                .da(0xff)
                .build(),
        )
        .copy_from_slice(&[0x02, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        .build();

        println!("Data1: {}", data_frame1);
    }

    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.0.write(frame).await
    }
}

pub trait Routable: Send + Sync {
    fn node(&self) -> u8;

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool;
}

pub struct Router {
    net: std::sync::Arc<J1939Network>,
    frame: Option<Frame>,
    filter_pgn: Vec<u32>,
    filter_node: Vec<u8>,
    node_table: HashMap<u8, std::time::Instant>,
}

impl Router {
    pub fn new(net: std::sync::Arc<J1939Network>) -> Self {
        Self {
            net,
            frame: None,
            filter_pgn: vec![],
            filter_node: vec![],
            node_table: HashMap::new(),
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

    /// Return the table of nodes found on the network.
    #[inline]
    pub fn node_table(&self) -> &HashMap<u8, std::time::Instant> {
        &self.node_table
    }

    /// Listen for incoming packets.
    pub async fn listen(&mut self) -> io::Result<()> {
        loop {
            let frame = self.net.accept().await?;

            let node_address = frame.id().sa();

            if self
                .node_table
                .insert(node_address, time::Instant::now())
                .is_none()
            {
                debug!("Detected new node on network: 0x{:X?}", node_address);
            }

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

    pub fn try_accept(&self, service: &mut impl Routable) -> bool {
        if let Some(frame) = self.frame {
            (service.node() == frame.id().sa() || service.node() == 0xff)
                && service.ingress(frame.id().pgn(), &frame)
        } else {
            false
        }
    }
}
