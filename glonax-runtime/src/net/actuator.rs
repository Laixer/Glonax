use std::{sync::Arc, time::{Duration, Instant}};

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
    last_interval: Instant,
    actuator_set: Option<std::collections::HashMap<u8, i16>>,
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

            if version != &[0xff; 3] {
                self.firmware_version = Some((version[0], version[1], version[2]))
            };

            let error = &frame.pdu()[6..8];

            if error != &[0xff; 2] {
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
            last_interval: Instant::now(),
            actuator_set: None,
            firmware_version: None,
            state: None,
            last_error: None,
        }
    }

    pub async fn interval(&mut self) {
        if self.last_interval.elapsed() >= Duration::from_millis(50) {
            if let Some(actuators) = &self.actuator_set {
                for (actuator, value) in actuators {
                    trace!("Keepalive: Change actuator {} to value {}", actuator, value);
                }

                self.set_actuator_control(self.node, actuators.clone())
                    .await;
            }
            self.last_interval = Instant::now();
        }
    }

    async fn set_motion_lock(&self, node: u8, locked: bool) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3.into())
                .da(node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, if locked { 0x0 } else { 0x1 }])
        .build();

        self.net.send(&frame).await.unwrap();
    }

    pub async fn lock(&mut self) {
        self.actuator_set = None;
        self.set_motion_lock(self.node, true).await;

        trace!("Disable motion");
    }

    pub async fn unlock(&mut self) {
        self.actuator_set = None;
        self.set_motion_lock(self.node, false).await;

        trace!("Enable motion");
    }

    async fn set_actuator_control(&self, node: u8, actuators: std::collections::HashMap<u8, i16>) {
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
                self.net.send(&frame).await.unwrap();
            }
        }
    }

    pub async fn actuator_stop(&mut self, actuators: Vec<u8>) {
        // TODO: Log after await
        for actuator in &actuators {
            trace!("Stop actuator {}", actuator);
        }

        self.set_actuator_control(
            self.node,
            actuators.into_iter().map(|k| (k as u8, 0)).collect(),
        )
        .await;
    }

    pub async fn actuator_control(&mut self, actuators: std::collections::HashMap<u8, i16>) {
        // TODO: Log after await
        for (actuator, value) in &actuators {
            trace!("Change actuator {} to value {}", actuator, value);
        }

        self.set_actuator_control(self.node, actuators.clone())
            .await;

        // self.actuator_set = Some(actuators)
    }
}
