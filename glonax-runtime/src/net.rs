use std::io;

use j1939::{Frame, FrameBuilder, Id, IdBuilder, Name, PGN};

pub use crate::can::{CANSocket, SockAddrCAN};

// TODO: Move to J1939 crate
/// Assign address to node.
pub fn commanded_address(sa: u8, name: &Name, address: u8) -> Vec<Frame> {
    let data = name
        .to_bytes()
        .into_iter()
        .chain(std::iter::once(address))
        .collect::<Vec<u8>>();

    broadcast_announce_message(sa, PGN::CommandedAddress, &data)
}

pub enum ConnectionManagement {
    RequestToSend = 0x10,
    ClearToSend = 0x11,
    EndOfMessageAcknowledgment = 0x13,
    BroadcastAnnounceMessage = 0x20,
    Abort = 0xff,
}

pub enum BroadcastTransportState {
    ConnectionManagement,
    DataTransfer(u8),
}

pub struct BroadcastTransport {
    sa: u8,
    pgn: PGN,
    data: [u8; 1785],
    length: usize,
    state: BroadcastTransportState,
}

impl BroadcastTransport {
    pub fn new(sa: u8, pgn: PGN) -> Self {
        Self {
            sa,
            pgn,
            data: [0xFF; 1785],
            length: 0,
            state: BroadcastTransportState::ConnectionManagement,
        }
    }

    pub fn with_data(mut self, data: &[u8]) -> Self {
        self.data[..data.len()].copy_from_slice(data);
        self.length = data.len();
        self
    }

    pub fn packets(&self) -> usize {
        (self.length as f32 / 7.0).ceil() as usize
    }

    pub fn next_frame(&mut self) -> Frame {
        match self.state {
            BroadcastTransportState::ConnectionManagement => {
                let data_length = (self.length as u16).to_le_bytes();
                let packets = self.packets() as u8;
                let byte_array = self.pgn.to_le_bytes(); // TODO: Move to J1939 crate

                let frame = FrameBuilder::new(
                    IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
                        .priority(7)
                        .sa(self.sa)
                        .da(0xff)
                        .build(),
                )
                .copy_from_slice(&[
                    ConnectionManagement::BroadcastAnnounceMessage as u8,
                    data_length[0],
                    data_length[1],
                    packets,
                    0xff,
                    byte_array[0],
                    byte_array[1],
                    byte_array[2],
                ])
                .build();

                self.state = BroadcastTransportState::DataTransfer(0);

                frame
            }
            // TODO: Return error frame if packet is out of bounds.
            BroadcastTransportState::DataTransfer(packet) => {
                let sequence = packet + 1;

                let mut frame_builder = FrameBuilder::new(
                    IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                        .priority(7)
                        .sa(self.sa)
                        .da(0xff)
                        .build(),
                );

                let payload = frame_builder.as_mut();
                payload[0] = sequence;

                let data_chunk = &self.data[packet as usize * 7..(packet as usize + 1) * 7];

                if data_chunk.len() == 7 {
                    payload[1..8].copy_from_slice(data_chunk);
                } else {
                    payload[1..(data_chunk.len() + 1)].copy_from_slice(data_chunk);
                }

                let frame = frame_builder.set_len(8).build();

                self.state = BroadcastTransportState::DataTransfer(packet + 1);

                frame
            }
        }
    }
}

// NOTE: Send with 50ms interval, timeout after 750ms
// TODO: This could be invalid, check priority and destination address.
// TODO: Move to J1939 crate
/// Broadcast Announce Message.
#[deprecated]
pub fn broadcast_announce_message(sa: u8, pgn: PGN, data: &[u8]) -> Vec<Frame> {
    let data_length = (data.len() as u16).to_le_bytes();
    let packets = (data.len() as f32 / 7.0).ceil() as u8;
    let byte_array = pgn.to_le_bytes();

    let mut frames = vec![];

    let connection_frame = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .priority(7)
            .sa(sa)
            .da(0xff)
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
                .sa(sa)
                .da(0xff)
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

    let _cm_cts = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .priority(7)
            .sa(sa)
            .da(da)
            .build(),
    )
    .copy_from_slice(&[
        ConnectionManagement::ClearToSend as u8,
        data_length[0], // TODO: Total number of packets allowed to be sent
        data_length[1], // TODO: Next sequence number expected
        0xff,
        0xff,
        byte_array[0],
        byte_array[1],
        byte_array[2],
    ])
    .build();

    let _cm_ack = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .priority(7)
            .sa(sa)
            .da(da)
            .build(),
    )
    .copy_from_slice(&[
        ConnectionManagement::EndOfMessageAcknowledgment as u8,
        data_length[0],
        data_length[1],
        packets,
        0xff,
        byte_array[0],
        byte_array[1],
        byte_array[2],
    ])
    .build();

    //     The Connection Abort Reasons can be:
    // 1 – Node is already engaged in another session and cannot maintain another connection.
    // 2 – Node is lacking the necessary resources.
    // 3 – A timeout occurred.
    // 4...255 - Reserved.

    let _cm_abort = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .priority(7)
            .sa(sa)
            .da(da)
            .build(),
    )
    .copy_from_slice(&[
        ConnectionManagement::Abort as u8,
        data_length[0], // TOOD: Reason for abort
        data_length[1],
        packets,
        0xff,
        byte_array[0],
        byte_array[1],
        byte_array[2],
    ])
    .build();

    let connection_frame = FrameBuilder::new(
        IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
            .priority(7)
            .sa(sa)
            .da(da)
            .build(),
    )
    .copy_from_slice(&[
        ConnectionManagement::RequestToSend as u8,
        data_length[0],
        data_length[1],
        packets,
        0xff, // TODO: Maximum number of packets
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

/// A trait for parsing frames.
///
/// This trait is used to accept and parse incoming frames and return a message if the frame is
/// parsable. The trait is used to implement a parser for a specific message type.
pub trait Parsable<T>: Send + Sync {
    /// Parse a frame.
    ///
    /// Returns `None` if the frame is not parsable. Returns `Some(T)` if the frame is parsable
    /// and the message is successfully parsed and returned.
    fn parse(&self, frame: &Frame) -> Option<T>;
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
    /// ECU Name.
    name: Name,
    /// Network interface.
    interface: String,
}

impl ControlNetwork {
    /// Construct a new control network.
    fn from_socket(socket: CANSocket, name: &Name, interface: &str) -> Self {
        Self {
            socket,
            frame: None,
            filter: Filter::accept(),
            name: *name,
            interface: interface.to_owned(),
        }
    }

    /// Construct a new control network and bind to an interface.
    pub fn bind(interface: &str, name: &Name) -> io::Result<Self> {
        let socket = CANSocket::bind(&SockAddrCAN::new(interface))?;
        Ok(Self::from_socket(socket, name, interface))
    }

    /// Set the global filter.
    #[inline]
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Return the current frame source.
    #[inline]
    pub fn frame_source(&self) -> Option<u8> {
        self.frame.map(|f| f.id().source_address())
    }

    /// Return the name of the control network.
    #[inline]
    pub fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Return the current frame.
    #[inline]
    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
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

    // TODO: Refactor into a single expression.
    // TODO: This is a mess, split logic.
    /// Listen for incoming packets.
    pub async fn recv(&mut self) -> io::Result<()> {
        loop {
            let frame = self.socket.recv().await?;
            if self.filter.matches(frame.id()) {
                self.frame = Some(
                    FrameBuilder::new(*frame.id())
                        .copy_from_slice(frame.as_ref())
                        .set_len(8)
                        .build(),
                );
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

#[derive(Debug, Copy, Clone)]
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
        let match_items = self.items.iter().all(|item| item.matches(id));
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

    #[test]
    fn test_filter_item_matches_3() {
        let id = IdBuilder::from_pgn(PGN::ProprietaryB(65_282))
            .sa(0x29)
            .build();

        let mut filter = Filter::accept();
        filter.push(FilterItem::Pgn(65_282));
        filter.push(FilterItem::SourceAddress(0x29));

        assert!(filter.matches(&id));
    }

    #[test]
    fn test_filter_item_matches_4() {
        let id = IdBuilder::from_pgn(PGN::CruiseControlVehicleSpeed)
            .sa(0x30)
            .build();

        let mut filter = Filter::accept();
        filter.push(FilterItem::Pgn(PGN::CruiseControlVehicleSpeed.into()));
        filter.push(FilterItem::SourceAddress(0x81));

        assert!(!filter.matches(&id));
    }
}
