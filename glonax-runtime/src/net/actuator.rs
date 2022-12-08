use std::sync::Arc;

use glonax_j1939::*;

use super::{ControlNet, Routable};

pub enum ActuatorState {
    Nominal,
    Ident,
    Faulty,
}

impl std::fmt::Display for ActuatorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActuatorState::Nominal => write!(f, "no error"),
            ActuatorState::Ident => write!(f, "ident"),
            ActuatorState::Faulty => write!(f, "faulty"),
        }
    }
}

pub struct ActuatorService {
    net: Arc<ControlNet>,
    node: u8,
    firmware_version: Option<(u8, u8, u8)>,
    state: Option<ActuatorState>,
    last_error: Option<u16>,
}

impl Routable for ActuatorService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == PGN::ProprietaryB(65_282) {
            self.state = match frame.pdu()[1] {
                0x14 => Some(ActuatorState::Nominal),
                0x16 => Some(ActuatorState::Ident),
                0xfa => Some(ActuatorState::Faulty),
                _ => None,
            };

            let version = &frame.pdu()[2..5];

            if version != [0xff; 3] {
                self.firmware_version = Some((version[0], version[1], version[2]))
            };

            let error = &frame.pdu()[6..8];

            if error != [0xff; 2] {
                self.last_error = Some(u16::from_le_bytes(error.try_into().unwrap()))
            }

            true
        } else {
            false
        }
    }
}

impl std::fmt::Display for ActuatorService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "State: {}; Version: {}; Last error: {}",
            self.state
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| f.to_string()),
            self.firmware_version.map_or_else(
                || "-".to_owned(),
                |f| { format!("{}.{}.{}", f.0, f.1, f.2) }
            ),
            self.last_error
                .map_or_else(|| "-".to_owned(), |f| { f.to_string() })
        )
    }
}

impl ActuatorService {
    pub fn new(net: Arc<ControlNet>, node: u8) -> Self {
        Self {
            net,
            node,
            firmware_version: None,
            state: None,
            last_error: None,
        }
    }

    async fn set_motion_lock(&self, node: u8, locked: bool) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3)
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, if locked { 0x0 } else { 0x1 }])
        .build();

        self.net.send(&frame).await.unwrap();
    }

    pub async fn lock(&self) {
        self.set_motion_lock(self.node, true).await;

        trace!("Disable motion");
    }

    pub async fn unlock(&self) {
        self.set_motion_lock(self.node, false).await;

        trace!("Enable motion");
    }

    pub async fn actuator_control(&self, actuators: std::collections::HashMap<u8, i16>) {
        const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
        const BANK_SLOTS: u8 = 4;

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            let mut actuator_list_filled: Vec<Option<i16>> = vec![];

            for slot in 0..BANK_SLOTS {
                let offset = (idx as u8 * 4) + slot;

                actuator_list_filled.push(actuators.get(&offset).copied());
            }

            if actuator_list_filled.iter().any(|f| f.is_some()) {
                let pdu = actuator_list_filled
                    .iter()
                    .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                    .collect::<Vec<u8>>()
                    .as_slice()[..8]
                    .try_into()
                    .unwrap();

                let frame = Frame::new(IdBuilder::from_pgn(bank).da(self.node).build(), pdu);
                self.net.send(&frame).await.unwrap();
            }
        }

        for (actuator, value) in &actuators {
            trace!("Change actuator {} to value {}", actuator, value);
        }
    }
}
