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

// TODO: Maybe rename to J1939Application?
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

    // TODO: Remove from this layer.
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
    #[inline]
    pub async fn request(&self, node: u8, pgn: PGN) {
        self.0.write(&protocol::request(node, pgn)).await.unwrap();
    }

    /// Broadcast Announce Message.
    pub async fn broadcast(&self, node: u8, pgn: PGN, data: &[u8]) {
        let data_length = (data.len() as u16).to_le_bytes();

        let packets = (data.len() as f32 / 8.0).ceil() as u8;

        let byte_array = pgn.to_le_bytes();

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

        println!("XConn: {}", connection_frame);
        // self.0.write(&connection_frame).await.unwrap();

        for data_packet in 0..packets {
            let mut data_frame0_b = FrameBuilder::new(
                IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                    .priority(7)
                    .da(node)
                    .build(),
            )
            .copy_from_slice(&[data_packet + 1]);

            let offset = data_packet as usize * 7;

            let qq = data_frame0_b.as_mut();
            if data_packet + 1 == packets {
                let stride_limit = offset + (data.len() - offset);
                qq[1..(data.len() - offset + 1)].copy_from_slice(&data[offset..stride_limit]);
            } else {
                let stride_limit = offset + 7;
                qq[1..8].copy_from_slice(&data[offset..stride_limit]);
            }

            let data_frame0 = data_frame0_b.set_len(8).build();

            println!("Data{}: {}", data_packet, data_frame0);
            // self.0.write(&data_frame0).await.unwrap();
        }
    }

    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.0.write(frame).await
    }
}

pub trait Routable: Send + Sync {
    fn ingress(&mut self, frame: &Frame) -> bool;
}

pub struct Router {
    net: Vec<J1939Network>,
    frame: Option<Frame>,
    filter_pgn: Vec<u32>,
    filter_node: Vec<u8>,
    node_table: HashMap<u8, std::time::Instant>,
}

impl FromIterator<J1939Network> for Router {
    fn from_iter<T: IntoIterator<Item = J1939Network>>(iter: T) -> Self {
        Self {
            net: Vec::from_iter(iter),
            frame: None,
            filter_pgn: vec![],
            filter_node: vec![],
            node_table: HashMap::new(),
        }
    }
}

impl Router {
    pub fn new(net: J1939Network) -> Self {
        Self {
            net: vec![net],
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
            let frame = self.net.first().unwrap().accept().await?;

            let node_address = frame.id().sa();

            if self
                .node_table
                .insert(node_address, time::Instant::now())
                .is_none()
            {
                debug!("Detected new node on network: 0x{:X?}", node_address);
            }

            self.node_table.retain(|node_address, last_seen| {
                let active = last_seen.elapsed() < std::time::Duration::from_millis(1_500);
                if !active {
                    debug!("Node 0x{:X?} not seen, kicking...", node_address);
                }
                active
            });

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
        self.frame.map_or(false, |frame| service.ingress(&frame))
    }
}
