use std::{io, time::Duration};

use j1939::{Frame, FrameBuilder, Id, IdBuilder, Name, PGN};

pub use crate::can::{CANSocket, SockAddrCAN};

// TODO: Move to J1939 crate
/// Assign address to node.
pub fn commanded_address(node: u8, address: u8) -> Vec<Frame> {
    // TODO: First 8 bytes are NAME, last byte is address
    let data = vec![0x18, 0xA4, 0x49, 0x24, 0x11, 0x05, 0x06, 0x85, address];

    broadcast_announce_message(node, PGN::CommandedAddress, &data)
}

// TODO: This could be invalid, check priority and destination address.
// TODO: Move to J1939 crate
/// Broadcast Announce Message.
pub fn broadcast_announce_message(node: u8, pgn: PGN, data: &[u8]) -> Vec<Frame> {
    let data_length = (data.len() as u16).to_le_bytes();
    let packets = (data.len() as f32 / 7.0).ceil() as u8;
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

// >[1CEC20FB] Prio: 7 PGN: 60416 DA: 0x20    [10, 10, 00, 03, FF, 00, EF, 00]
// <[18ECFB20] Prio: 6 PGN: 60416 DA: 0xFB    [11, 03, 01, FF, FF, 00, EF, 00]
// >[1CEB20FB] Prio: 7 PGN: 60160 DA: 0x20    [01, 64, 00, 02, 01, 00, 00, 02]
// >[1CEB20FB] Prio: 7 PGN: 60160 DA: 0x20    [02, 01, 00, 00, 32, 00, 7A, 00]
// >[1CEB20FB] Prio: 7 PGN: 60160 DA: 0x20    [03, 00, 06, FF, FF, FF, FF, FF]
// <[18ECFB20] Prio: 6 PGN: 60416 DA: 0xFB    [13, 10, 00, 03, FF, 00, EF, 00]

// TODO: Move to J1939 crate
/// Destination specific transport protocol.
pub fn destination_specific(da: u8, sa: u8, pgn: PGN, data: &[u8]) -> Vec<Frame> {
    let data_length = (data.len() as u16).to_le_bytes();
    let packets = (data.len() as f32 / 7.0).ceil() as u8;
    let byte_array = pgn.to_le_bytes();

    let mut frames = vec![];

    let connection_frame = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .sa(sa)
            .da(da)
            .build(),
    )
    .copy_from_slice(&[
        0x10,
        data_length[0],
        data_length[1],
        packets,
        0xff, // TODO
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
                .sa(sa)
                .da(da)
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

/// The control network is used to accept and store incoming frames.
///
/// Frames are routed based on the PGN and the ECU address. The router
/// supports filtering based on PGN and addresses.
///
/// If the frame size is fixed, the router will always return a frame of
/// equal size. If the frame size is not fixed, it is returned as is.
/// Fixing the frame size avoids the need to check the frame size in each
/// service that uses the control network. This is both a performance
/// optimization and a safety feature.
pub struct ControlNetwork {
    /// The network.
    socket: CANSocket,
    /// The current frame.
    frame: Option<Frame>,
    /// Network filter.
    filter: Filter,
    /// The fixed frame size.
    fix_frame_size: bool,
    /// ECU Name.
    name: Name,
}

impl ControlNetwork {
    // TODO: Rename to `from_socket`.
    /// Construct a new control network.
    pub fn new(socket: CANSocket, name: &Name) -> Self {
        Self {
            socket,
            frame: None,
            filter: Filter::accept(),
            fix_frame_size: true,
            name: *name,
        }
    }

    /// Construct a new control network and bind to an interface.
    pub fn bind(interface: &str, name: &Name) -> io::Result<Self> {
        let socket = CANSocket::bind(&SockAddrCAN::new(interface))?;
        Ok(Self::new(socket, name))
    }

    /// Set the global filter.
    #[inline]
    pub fn set_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
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
    #[deprecated]
    #[inline]
    pub fn take(&mut self) -> Option<Frame> {
        self.frame.take()
    }

    /// Return the name of the control network.
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

    // TODO: This is a mess, split logic.
    // TODO: Rename to `recv`
    /// Listen for incoming packets.
    pub async fn listen(&mut self) -> io::Result<()> {
        loop {
            let frame = self.socket.recv().await?;
            if self.filter.matches(frame.id()) {
                self.frame = if self.fix_frame_size {
                    Some(
                        FrameBuilder::new(*frame.id())
                            .copy_from_slice(frame.as_ref())
                            .set_len(8)
                            .build(),
                    )
                } else {
                    Some(frame)
                };
                break;
            }
        }

        Ok(())
    }

    // TODO: Rename to `recv_timeout`
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

pub enum FilterItem {
    /// Filter by priority.
    Priority(u8),
    /// Filter by PGN.
    Pgn(u32),
    /// Filter by source address.
    SourceAddress(u8),
    /// Filter by destination address.
    DestinationAddress(u8),
}

impl FilterItem {
    fn matches(&self, id: &Id) -> bool {
        match self {
            FilterItem::Priority(priority) => *priority == id.priority(),
            FilterItem::Pgn(pgn) => *pgn == id.pgn_raw(),
            FilterItem::SourceAddress(address) => *address == id.source_address(),
            FilterItem::DestinationAddress(address) => Some(*address) == id.destination_address(),
        }
    }
}

pub struct Filter {
    /// Filter items.
    items: Vec<FilterItem>,
    /// Default policy.
    accept: bool,
}

impl Filter {
    pub fn accept() -> Self {
        Self {
            items: vec![],
            accept: true,
        }
    }

    pub fn reject() -> Self {
        Self {
            items: vec![],
            accept: false,
        }
    }

    pub fn push(&mut self, item: FilterItem) {
        self.items.push(item);
    }

    pub fn matches(&self, id: &Id) -> bool {
        let match_items = self.items.iter().any(|item| item.matches(id));
        if self.accept {
            if !self.items.is_empty() {
                match_items
            } else {
                true
            }
        } else if !self.items.is_empty() {
            !match_items
        } else {
            true
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::accept()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_item_matches_1() {
        let id = IdBuilder::from_pgn(PGN::Request)
            .priority(5)
            .sa(0x01)
            .da(0x02)
            .build();

        let priority = FilterItem::Priority(5);
        let pgn = FilterItem::Pgn(59_904);
        let source_address = FilterItem::SourceAddress(0x01);
        let destination_address = FilterItem::DestinationAddress(0x02);

        assert!(priority.matches(&id));
        assert!(pgn.matches(&id));
        assert!(source_address.matches(&id));
        assert!(destination_address.matches(&id));
    }

    #[test]
    fn test_filter_item_matches_2() {
        let id = IdBuilder::from_pgn(PGN::DashDisplay)
            .priority(3)
            .sa(0x7E)
            .da(0xDA)
            .build();

        let priority = FilterItem::Priority(3);
        let pgn = FilterItem::Pgn(65_276);
        let source_address = FilterItem::SourceAddress(0x7E);
        let destination_address = FilterItem::DestinationAddress(0xDA);

        assert!(priority.matches(&id));
        assert!(pgn.matches(&id));
        assert!(source_address.matches(&id));
        assert!(!destination_address.matches(&id));
    }
}
