use std::sync::Arc;

use glonax_j1939::*;

use super::{ControlNet, Routable};

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
    _net: Arc<ControlNet>,
    node: u8,
    position: u32,
    speed: u16,
    state: Option<EncoderState>,
}

impl Routable for KueblerEncoderService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == PGN::ProprietaryB(65_450) {
            let position_bytes = &frame.pdu()[0..4];
            if position_bytes != &[0xff; 4] {
                self.position = u32::from_le_bytes(position_bytes.try_into().unwrap());
            };

            let speed_bytes = &frame.pdu()[4..6];
            if speed_bytes != &[0xff; 2] {
                self.speed = u16::from_le_bytes(speed_bytes.try_into().unwrap());
            };

            let state_bytes = &frame.pdu()[6..8];
            if state_bytes != &[0xff; 2] {
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
    pub fn new(net: std::sync::Arc<ControlNet>, node: u8) -> Self {
        Self {
            _net: net,
            node,
            position: 0,
            speed: 0,
            state: None,
        }
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn speed(&self) -> u16 {
        self.speed
    }
}

////////////

#[deprecated]
pub enum LaixerEncoderState {
    Nominal,
    Ident,
    Faulty,
}

impl std::fmt::Display for LaixerEncoderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LaixerEncoderState::Nominal => write!(f, "no error"),
            LaixerEncoderState::Ident => write!(f, "ident"),
            LaixerEncoderState::Faulty => write!(f, "faulty"),
        }
    }
}

#[deprecated]
pub struct LaixerEncoderService {
    _net: Arc<ControlNet>,
    node: u8,
    position: u32,
    firmware_version: Option<(u8, u8, u8)>,
    state: Option<LaixerEncoderState>,
    last_error: Option<u16>,
}

impl Routable for LaixerEncoderService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == PGN::ProprietaryB(65_282) {
            self.state = match frame.pdu()[1] {
                0x14 => Some(LaixerEncoderState::Nominal),
                0x16 => Some(LaixerEncoderState::Ident),
                0xfa => Some(LaixerEncoderState::Faulty),
                _ => None,
            };

            let version = &frame.pdu()[2..5];

            if version != &[0xff; 3] {
                self.firmware_version = Some((version[0], version[1], version[2]))
            };

            let error = &frame.pdu()[6..8];

            if error != &[0xff; 2] {
                self.last_error = Some(u16::from_le_bytes(error.try_into().unwrap()))
            }

            true
        } else if pgn == PGN::Other(64_258) {
            self.position = u32::from_le_bytes(frame.pdu()[0..4].try_into().unwrap());

            true
        } else {
            false
        }
    }
}

impl std::fmt::Display for LaixerEncoderService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {}; State: {}",
            self.position,
            self.state
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| f.to_string()),
        )
    }
}

impl LaixerEncoderService {
    pub fn new(net: std::sync::Arc<ControlNet>, node: u8) -> Self {
        Self {
            _net: net,
            node,
            position: 0,
            firmware_version: None,
            state: None,
            last_error: None,
        }
    }

    pub fn position(&self) -> u32 {
        self.position
    }
}
