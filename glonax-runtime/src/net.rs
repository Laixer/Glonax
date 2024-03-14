use std::io;

use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

pub use crate::can::{CANSocket, SockAddrCAN};

// TODO: Move to J1939 crate
/// Assign address to node.
pub fn commanded_address(node: u8, address: u8) -> Vec<Frame> {
    let data = vec![0x18, 0xA4, 0x49, 0x24, 0x11, 0x05, 0x06, 0x85, address];

    broadcast_announce(node, PGN::CommandedAddress, &data)
}

// TODO: Move to J1939 crate
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

pub trait Parsable<T>: Send + Sync {
    /// Parse a frame.
    ///
    /// Returns `None` if the frame is not parsable. Returns `Some(T)` if the frame is parsable
    /// and the message is successfully parsed and returned.
    fn parse(&mut self, frame: &Frame) -> Option<T>;
}

/// The router is used to route incoming frames to compatible services.
///
/// Frames are routed based on the PGN and the ECU address. The router
/// supports filtering based on PGN and addresses.
///
/// If the frame size is fixed, the router will always return a frame of
/// equal size. If the frame size is not fixed, it is returned as is.
/// Fixing the frame size avoids the need to check the frame size in each
/// service.
pub struct Router {
    /// The network.
    socket_list: Vec<CANSocket>,
    /// The current frame.
    frame: Option<Frame>,
    /// The priority filter.
    filter_priority: Vec<u8>,
    /// The PGN filter.
    filter_pgn: Vec<u32>,
    /// The address filter.
    filter_address: Vec<u8>,
    /// The fixed frame size.
    fix_frame_size: bool,
}

impl Router {
    /// Construct a new router.
    pub fn new(socket: CANSocket) -> Self {
        Self {
            socket_list: vec![socket],
            frame: None,
            filter_priority: vec![],
            filter_pgn: vec![],
            filter_address: vec![],
            fix_frame_size: true,
        }
    }

    /// Add a filter based on priority.
    #[inline]
    pub fn add_priority_filter(&mut self, priority: u8) {
        self.filter_priority.push(priority);
    }

    /// Add a filter based on PGN.
    #[inline]
    pub fn add_pgn_filter(&mut self, pgn: u32) {
        self.filter_pgn.push(pgn);
    }

    /// Add a filter based on ECU address.
    #[inline]
    pub fn add_address_filter(&mut self, address: u8) {
        self.filter_address.push(address);
    }

    /// Set the fixed frame size.
    #[inline]
    pub fn set_fix_frame_size(mut self, fix_frame_size: bool) -> Self {
        self.fix_frame_size = fix_frame_size;
        self
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

    /// Return the inner network socket.
    #[inline]
    pub fn inner(&self) -> &CANSocket {
        &self.socket_list[0]
    }

    /// Send a frame.
    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.socket_list[0].send(frame).await
    }

    /// Send a vector of frames.
    #[inline]
    pub async fn send_vectored(&self, frames: &Vec<Frame>) -> io::Result<Vec<usize>> {
        self.socket_list[0].send_vectored(frames).await
    }

    /// Filter the frame based on PGN and address.
    ///
    /// Returns `true` if the frame is accepted. Returns `false` if the frame is not accepted.
    /// If no filters are set, all frames are accepted.
    fn filter(&self, frame: &Frame) -> bool {
        if !self.filter_priority.is_empty()
            && !self.filter_priority.contains(&frame.id().priority())
        {
            false
        } else if !self.filter_pgn.is_empty() && !self.filter_pgn.contains(&frame.id().pgn_raw()) {
            false
        } else {
            !(!self.filter_address.is_empty() && !self.filter_address.contains(&frame.id().sa()))
        }
    }

    /// Listen for incoming packets.
    pub async fn listen(&mut self) -> io::Result<()> {
        loop {
            let frame = self.socket_list[0].recv().await?;
            if self.filter(&frame) {
                if self.fix_frame_size {
                    self.frame = Some(
                        FrameBuilder::new(*frame.id())
                            .copy_from_slice(frame.as_ref())
                            .set_len(8)
                            .build(),
                    );
                } else {
                    self.frame = Some(frame);
                }
                break;
            }
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
