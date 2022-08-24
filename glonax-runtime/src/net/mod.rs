use std::{collections::HashMap, io};

use glonax_j1939::{Frame, FrameBuilder, IdBuilder, J1939Stream};

pub use service::ActuatorService;
pub use service::ControlService;
pub use service::StatusService;

pub mod motion;
mod service;

pub struct ControlNet {
    stream: J1939Stream,
}

impl ControlNet {
    pub fn new(ifname: &str, addr: u8) -> io::Result<Self> {
        let stream = glonax_j1939::J1939Stream::bind(ifname, addr)?;
        stream.set_broadcast(true)?;
        Ok(Self { stream })
    }

    pub fn stream(&self) -> &J1939Stream {
        &self.stream
    }

    pub async fn accept(&self) -> io::Result<Frame> {
        self.stream.read().await
    }

    pub async fn announce_status(&self) {
        let state = 0xff;

        const PKG_VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
        const PKG_VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
        const PKG_VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

        let major: u8 = PKG_VERSION_MAJOR.parse().unwrap();
        let minor: u8 = PKG_VERSION_MINOR.parse().unwrap();
        let patch: u8 = PKG_VERSION_PATCH.parse().unwrap();

        let frame = FrameBuilder::new(IdBuilder::from_pgn(65_282).build())
            .from_slice(&[0xff, state, major, minor, patch, 0xff, 0xff, 0xff])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn set_led(&self, node: u8, led_on: bool) {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', if led_on { 0x1 } else { 0x0 }])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn set_address(&self, node: u8, address: u8) {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(45_568).da(node).build())
            .from_slice(&[b'Z', b'C', address])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn reset(&self, node: u8) {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, 0x69])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn enable_encoder(&self, node: u8, encoder: u8, encoder_on: bool) {
        let state = match (encoder, encoder_on) {
            (0, true) => 0b1101,
            (0, false) => 0b1100,
            (1, true) => 0b0111,
            (1, false) => 0b0011,
            _ => panic!(),
        };

        let frame = FrameBuilder::new(IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', state])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn set_motion_lock(&self, node: u8, locked: bool) {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, if locked { 0x0 } else { 0x1 }])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    pub async fn request(&self, node: u8, _pgn: u32) {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(59_904).da(node).build())
            .from_slice(&[0xfe, 0x18, 0xda])
            .build();

        self.stream.write(&frame).await.unwrap();
    }

    /// Control actuators.
    pub async fn actuator_control(&self, node: u8, actuators: HashMap<u8, i16>) {
        const BANK_PGN_LIST: [u16; 2] = [40_960, 41_216];
        const BANK_SLOTS: u8 = 4;

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            let mut actuator_list_filled: Vec<Option<i16>> = vec![];

            for slot in 0..BANK_SLOTS {
                let offset = (idx as u8 * 4) + slot;

                actuator_list_filled.push(actuators.get(&offset).map_or(None, |a| Some(*a)));
            }

            if actuator_list_filled.iter().any(|f| f.is_some()) {
                let pdu = actuator_list_filled
                    .iter()
                    .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                    .collect::<Vec<u8>>()
                    .as_slice()[..8]
                    .try_into()
                    .unwrap();

                let frame = Frame::new(IdBuilder::from_pgn(bank).da(node).build(), pdu);
                self.stream.write(&frame).await.unwrap();
            }
        }
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
