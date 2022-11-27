use std::io;

use glonax_j1939::*;

pub use service::ActuatorService;
pub use service::StatusService;

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
    pub async fn request(&self, node: u8, pgn: u16) {
        let byte_array = u32::to_be_bytes(pgn as u32);

        let frame = FrameBuilder::new(IdBuilder::from_pgn(PGN::Request.into()).da(node).build())
            .copy_from_slice(&[byte_array[3], byte_array[2], byte_array[1]])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    #[inline]
    pub async fn send(&self, frame: &Frame) -> io::Result<usize> {
        self.stream.write(&frame).await
    }
}

pub enum State {
    Nominal,
    Ident,
    Faulty,
}

pub fn spn_state(value: u8) -> Option<State> {
    match value {
        0x14 => Some(State::Nominal),
        0x16 => Some(State::Ident),
        0xfa => Some(State::Faulty),
        _ => None,
    }
}

// TODO: Maybe move?
pub fn spn_firmware_version(value: &[u8; 3]) -> Option<(u8, u8, u8)> {
    if value != &[0xff; 3] {
        Some((value[0], value[1], value[2]))
    } else {
        None
    }
}

// TODO: Maybe move?
pub fn spn_last_error(value: &[u8; 2]) -> Option<u16> {
    if value != &[0xff; 2] {
        Some(u16::from_le_bytes(value[..2].try_into().unwrap()))
    } else {
        None
    }
}
