use std::io;

use glonax_j1939::*;

pub use actuator::*;
pub use encoder::*;
pub use engine::*;
pub use service::*;

mod actuator;
mod encoder;
mod engine;
pub mod motion;
mod service;

// TODO: Implement connection management.
// TODO: Implement broadcast message.
pub struct ControlNet {
    stream: J1939Stream,
}

impl ControlNet {
    pub fn new(ifname: &str, addr: u8) -> io::Result<Self> {
        let stream = glonax_j1939::J1939Stream::bind(ifname, addr)?;
        stream.set_broadcast(true)?;
        Ok(Self { stream })
    }

    #[inline]
    pub async fn accept(&self) -> io::Result<Frame> {
        self.stream.read().await
    }

    // TODO: Change to Commanded Address
    pub async fn set_address(&self, node: u8, address: u8) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage2.into())
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', address])
        .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn reset(&self, node: u8) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1.into())
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, 0x69])
        .build();

        self.stream.write(&frame).await.unwrap();
    }

    // TODO: Maybe remove.
    pub async fn enable_encoder(&self, node: u8, encoder: u8, encoder_on: bool) {
        let state = match (encoder, encoder_on) {
            (0, true) => 0b1101,
            (0, false) => 0b1100,
            (1, true) => 0b0111,
            (1, false) => 0b0011,
            _ => panic!(),
        };

        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3.into())
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', state])
        .build();

        self.stream.write(&frame).await.unwrap();
    }

    /// Request a PGN message.
    pub async fn request(&self, node: u8, pgn: PGN) {
        self.stream
            .write(&protocol::request(node, pgn))
            .await
            .unwrap();
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
            IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement.into())
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
            IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer.into())
                .priority(7)
                .da(0xff)
                .build(),
        )
        .copy_from_slice(&[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        .build();

        println!("Data0: {}", data_frame0);

        let data_frame1 = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer.into())
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
        self.stream.write(&frame).await
    }
}

pub trait Routable: Send + Sync {
    fn node(&self) -> u8;

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool;
}

pub struct Router {
    net: std::sync::Arc<ControlNet>,
    frame: Option<Frame>,
    filter_pgn: Vec<u16>,
    filter_node: Vec<u8>,
}

impl Router {
    pub fn new(net: std::sync::Arc<ControlNet>) -> Self {
        Self {
            net,
            frame: None,
            filter_pgn: vec![],
            filter_node: vec![],
        }
    }

    pub fn add_pgn_filter(&mut self, pgn: u16) {
        self.filter_pgn.push(pgn);
    }

    pub fn add_node_filter(&mut self, node: u8) {
        self.filter_node.push(node);
    }

    pub fn frame_source(&self) -> Option<u8> {
        self.frame.map(|f| f.id().sa())
    }

    pub fn take(&mut self) -> Option<Frame> {
        self.frame.take()
    }

    pub async fn accept(&mut self) -> io::Result<()> {
        loop {
            let frame = self.net.accept().await?;

            if !self.filter_pgn.is_empty() {
                let pgn = frame.id().pgn_raw();
                if !self.filter_pgn.contains(&pgn) {
                    continue;
                }
            }

            if !self.filter_node.is_empty() {
                let node = frame.id().sa();
                if !self.filter_node.contains(&node) {
                    continue;
                }
            }

            self.frame = Some(frame);
            break;
        }

        Ok(())
    }

    pub fn try_accept(&self, service: &mut impl Routable) -> bool {
        if let Some(frame) = self.frame {
            (service.node() == frame.id().sa() || service.node() == 0xff)
                && service.ingress(frame.id().pgn().into(), &frame)
        } else {
            false
        }
    }
}
