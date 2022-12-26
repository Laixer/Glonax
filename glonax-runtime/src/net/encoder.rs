use std::sync::Arc;

use glonax_j1939::*;

use super::{J1939Network, Routable};

pub enum EncoderState {
    NoError,
    GeneralSensorError,
    InvalidMUR,
    InvalidTMR,
    InvalidPreset,
    Other,
}

impl std::fmt::Display for EncoderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncoderState::NoError => write!(f, "no error"),
            EncoderState::GeneralSensorError => write!(f, "general error in sensor"),
            EncoderState::InvalidMUR => write!(f, "invalid MUR value"),
            EncoderState::InvalidTMR => write!(f, "invalid TMR value"),
            EncoderState::InvalidPreset => write!(f, "invalid preset value"),
            EncoderState::Other => write!(f, "unknown error"),
        }
    }
}

pub struct KueblerEncoderService {
    _net: Arc<J1939Network>,
    node: u8,
    position: u32,
    speed: u16,
    state: Option<EncoderState>,
}

#[async_trait::async_trait]
impl Routable for KueblerEncoderService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == PGN::ProprietaryB(65_450) {
            let position_bytes = &frame.pdu()[0..4];
            if position_bytes != [0xff; 4] {
                self.position = u32::from_le_bytes(position_bytes.try_into().unwrap());
            };

            let speed_bytes = &frame.pdu()[4..6];
            if speed_bytes != [0xff; 2] {
                self.speed = u16::from_le_bytes(speed_bytes.try_into().unwrap());
            };

            let state_bytes = &frame.pdu()[6..8];
            if state_bytes != [0xff; 2] {
                let state = u16::from_le_bytes(state_bytes.try_into().unwrap());

                self.state = Some(match state {
                    0x0 => EncoderState::NoError,
                    0xee00 => EncoderState::GeneralSensorError,
                    0xee01 => EncoderState::InvalidMUR,
                    0xee02 => EncoderState::InvalidTMR,
                    0xee03 => EncoderState::InvalidPreset,
                    _ => EncoderState::Other,
                });
            }

            true
        } else {
            false
        }
    }
}

impl std::fmt::Display for KueblerEncoderService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {}; Speed {}; State: {}",
            self.position,
            self.speed,
            self.state
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| f.to_string()),
        )
    }
}

impl KueblerEncoderService {
    pub fn new(net: std::sync::Arc<J1939Network>, node: u8) -> Self {
        Self {
            _net: net,
            node,
            position: 0,
            speed: 0,
            state: None,
        }
    }

    #[inline]
    pub fn position(&self) -> u32 {
        self.position
    }

    #[inline]
    pub fn speed(&self) -> u16 {
        self.speed
    }
}
