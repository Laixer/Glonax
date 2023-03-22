use std::collections::HashMap;

use glonax_j1939::*;

use super::{J1939Network, Routable};

pub struct ActuatorService {
    net: J1939Network,
    node: u8,
    actuators: [Option<i16>; 8],
}

impl Routable for ActuatorService {
    fn ingress(&mut self, frame: &Frame) -> bool {
        if frame.id().sa() != self.node {
            return false;
        }

        if frame.id().pgn() == PGN::Other(40_960) {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                self.actuators[0] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                self.actuators[1] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                self.actuators[2] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                self.actuators[3] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
            true
        } else if frame.id().pgn() == PGN::Other(41_216) {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                self.actuators[4] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                self.actuators[5] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                self.actuators[6] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                self.actuators[7] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
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
            "Actuator state {}",
            self.actuators
                .iter()
                .enumerate()
                .map(|(idx, act)| {
                    format!(
                        "{}: {}",
                        idx,
                        act.map_or("NaN".to_owned(), |f| f.to_string())
                    )
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl ActuatorService {
    pub fn new(net: J1939Network, node: u8) -> Self {
        Self {
            net,
            node,
            actuators: [None; 8],
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

    // TODO: Maybe move into a trait
    pub async fn actuate(&mut self, motion: crate::transport::Motion) {
        match motion.r#type() {
            crate::transport::motion::MotionType::None => panic!("NONE should not be used"),
            crate::transport::motion::MotionType::StopAll => {
                self.lock().await;
            }
            crate::transport::motion::MotionType::ResumeAll => {
                self.unlock().await;
            }
            crate::transport::motion::MotionType::Change => {
                self.actuator_control(
                    motion
                        .changes
                        .into_iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value as i16))
                        .collect(),
                )
                .await;
            }
        }
    }

    // TODO: If possible make immutable.
    pub async fn actuator_control(&mut self, actuators: HashMap<u8, i16>) {
        const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
        const BANK_SLOTS: usize = 4;

        let mut bank_update = [false; 2];

        for (act, val) in &actuators {
            self.actuators[*act as usize] = Some(*val);

            bank_update[*act as usize / BANK_SLOTS] = true;
        }

        trace!("{}", self);

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            if !bank_update[idx] {
                continue;
            }

            let stride = idx * BANK_SLOTS;

            let pdu: [u8; 8] = self.actuators[stride..stride + BANK_SLOTS]
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
}
