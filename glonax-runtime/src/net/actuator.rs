use std::{collections::HashMap, sync::Arc};

use glonax_j1939::*;

use super::{ControlNet, Routable};

pub struct ActuatorService {
    net: Arc<ControlNet>,
    node: u8,
    actuators: [Option<i16>; 8],
}

impl Routable for ActuatorService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, _: PGN, _: &Frame) -> bool {
        false
    }
}

impl std::fmt::Display for ActuatorService {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ActuatorService {
    pub fn new(net: Arc<ControlNet>, node: u8) -> Self {
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

    pub async fn actuator_control(&mut self, actuators: HashMap<u8, i16>) {
        const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
        const BANK_SLOTS: usize = 4;

        let mut bank_update = [false; 2];

        for (act, val) in &actuators {
            self.actuators[*act as usize] = Some(*val);

            bank_update[*act as usize / BANK_SLOTS] = true;
        }

        trace!(
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
        );

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
