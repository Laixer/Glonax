use std::{io, time::Duration};

use j1939::{Frame, FrameBuilder, Id, IdBuilder, Name, PGN};

pub use crate::can::{CANSocket, SockAddrCAN};

// TODO: Move to J1939 crate
/// Assign address to node.
pub fn commanded_address(node: u8, address: u8) -> Vec<Frame> {
    // TODO: First 8 bytes are NAME, last byte is address
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
pub struct ControlNetwork {
    /// The network.
    socket: CANSocket,
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
    /// ECU Name.
    name: Name,
}

impl ControlNetwork {
    /// Construct a new control network.
    pub fn new(socket: CANSocket, name: &Name) -> Self {
        Self {
            socket,
            frame: None,
            filter_priority: vec![],
            filter_pgn: vec![],
            filter_address: vec![],
            fix_frame_size: true,
            name: *name,
        }
    }

    /// Construct a new control network and bind to an interface.
    pub fn bind(interface: &str, name: &Name) -> io::Result<Self> {
        let socket = CANSocket::bind(&SockAddrCAN::new(interface))?;
        Ok(Self::new(socket, name))
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
        self.frame.map(|f| f.id().source_address())
    }

    /// Take the frame from the router.
    #[inline]
    pub fn take(&mut self) -> Option<Frame> {
        self.frame.take()
    }

    /// Return the name of the ECU.
    #[inline]
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Send a frame.
    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.socket.send(frame).await
    }

    /// Send a vector of frames.
    #[inline]
    pub async fn send_vectored(&self, frames: &Vec<Frame>) -> io::Result<Vec<usize>> {
        self.socket.send_vectored(frames).await
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
            !(!self.filter_address.is_empty()
                && !self.filter_address.contains(&frame.id().source_address()))
        }
    }

    // TODO: This is a mess, split logic.
    // TODO: Rename to `recv` and `recv_timeout`
    /// Listen for incoming packets.
    pub async fn listen(&mut self) -> io::Result<()> {
        loop {
            let frame = self.socket.recv().await?;
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

    /// Listen for incoming packets.
    pub async fn listen_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        if let Ok(result) = tokio::time::timeout(timeout, self.listen()).await {
            result
        } else {
            // TODO: We just ignore the timeout for now.
            // Err(io::Error::new(io::ErrorKind::TimedOut, "Timeout"))
            Ok(())
        }
    }

    /// Try to accept a frame and parse it.
    ///
    /// This method will return `None` if the frame is not accepted. Otherwise, it will return
    /// `Some` with the resulting message.
    pub fn try_accept<T>(&self, service: &mut impl Parsable<T>) -> Option<T> {
        self.frame.and_then(|frame| service.parse(&frame))
    }
}

enum FilterItem {
    Priority(u8),
    PGN(u32),
    SourceAddress(u8),
    DestinationAddress(u8),
}

impl FilterItem {
    fn matches(&self, id: &Id) -> bool {
        match self {
            FilterItem::Priority(priority) => *priority == id.priority(),
            FilterItem::PGN(pgn) => *pgn == id.pgn_raw(),
            FilterItem::SourceAddress(address) => *address == id.source_address(),
            FilterItem::DestinationAddress(address) => Some(*address) == id.destination_address(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_item_matches() {
        let id = IdBuilder::from_pgn(PGN::Request)
            .priority(0x05)
            .sa(0x01)
            .da(0x02)
            .build();

        let priority = FilterItem::Priority(0x05);
        let pgn = FilterItem::PGN(59_904);
        let source_address = FilterItem::SourceAddress(0x01);
        let destination_address = FilterItem::DestinationAddress(0x02);

        assert!(priority.matches(&id));
        assert!(pgn.matches(&id));
        assert!(source_address.matches(&id));
        assert!(destination_address.matches(&id));
    }
}
