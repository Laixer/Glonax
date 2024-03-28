use std::io;

use j1939::{Frame, FrameBuilder, IdBuilder, Name, NameBuilder, PGN};

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

// TODO: Get from configuration.

/// J1939 name manufacturer code.
const J1939_NAME_MANUFACTURER_CODE: u16 = 0x717;
/// J1939 name function instance.
const J1939_NAME_FUNCTION_INSTANCE: u8 = 6;
/// J1939 name ECU instance.
const J1939_NAME_ECU_INSTANCE: u8 = 0;
/// J1939 name function.
const J1939_NAME_FUNCTION: u8 = 0x1C;
/// J1939 name vehicle system.
const J1939_NAME_VEHICLE_SYSTEM: u8 = 2;

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
    // / Source address.
    // source_address: u8,
    /// ECU Name.
    name: Name,
}

impl ControlNetwork {
    /// Construct a new control network.
    pub fn new(socket: CANSocket) -> Self {
        Self {
            socket,
            frame: None,
            filter_priority: vec![],
            filter_pgn: vec![],
            filter_address: vec![],
            fix_frame_size: true,
            // source_address: 0x27,
            name: NameBuilder::default()
                .identity_number(0x1)
                .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
                .function_instance(J1939_NAME_FUNCTION_INSTANCE)
                .ecu_instance(J1939_NAME_ECU_INSTANCE)
                .function(J1939_NAME_FUNCTION)
                .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
                .build(),
        }
    }

    /// Construct a new control network and bind to an interface.
    pub fn bind(interface: &str) -> io::Result<Self> {
        let socket = CANSocket::bind(&SockAddrCAN::new(interface))?;
        Ok(Self::new(socket))
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

    // / Return source address.
    // #[inline]
    // pub fn source_address(&self) -> u8 {
    //     self.source_address
    // }

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

    /// Try to accept a frame and parse it.
    ///
    /// This method will return `None` if the frame is not accepted. Otherwise, it will return
    /// `Some` with the resulting message.
    pub fn try_accept<T>(&self, service: &mut impl Parsable<T>) -> Option<T> {
        self.frame.and_then(|frame| service.parse(&frame))
    }
}
