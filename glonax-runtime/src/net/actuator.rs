use std::sync::Arc;

use glonax_j1939::*;

use super::{ControlNet, Routable};

pub struct ActuatorService {
    net: Arc<ControlNet>,
    node: u8,
    actuators: std::collections::HashMap<u8, i16>,
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
            actuators: std::collections::HashMap::new(),
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

    pub async fn actuator_control(&mut self, actuators: std::collections::HashMap<u8, i16>) {
        const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
        const BANK_SLOTS: u8 = 4;

        for (act, val) in &actuators {
            self.actuators.insert(*act, *val);
        }

        trace!("Actuator state {:?}", self.actuators);

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            let mut actuator_list_filled: Vec<Option<i16>> = vec![];

            for slot in 0..BANK_SLOTS {
                let offset = (idx as u8 * 4) + slot;

                actuator_list_filled.push(self.actuators.get(&offset).copied());
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
