use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glonax_j1939::*;

use super::ControlNet;

pub struct StatusService {
    net: Arc<ControlNet>,
    node: u8,
    last_interval: Instant,
}

impl StatusService {
    pub fn new(net: Arc<ControlNet>, node: u8) -> Self {
        Self {
            net,
            node,
            last_interval: Instant::now(),
        }
    }

    async fn announce_status(&self) {
        let state = 0xff;

        const PKG_VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
        const PKG_VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
        const PKG_VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

        let major: u8 = PKG_VERSION_MAJOR.parse().unwrap();
        let minor: u8 = PKG_VERSION_MINOR.parse().unwrap();
        let patch: u8 = PKG_VERSION_PATCH.parse().unwrap();

        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ProprietaryB(65_282).into()).build())
                .copy_from_slice(&[0xff, state, major, minor, patch, 0xff, 0xff, 0xff])
                .build();

        self.net.send(&frame).await.unwrap();
    }

    pub async fn set_led(&self, led_on: bool) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1.into())
                .da(self.node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', if led_on { 0x1 } else { 0x0 }])
        .build();

        self.net.send(&frame).await.unwrap();
    }

    pub async fn interval(&mut self) {
        if self.last_interval.elapsed() >= Duration::from_secs(1) {
            self.announce_status().await;

            trace!("Announce host on network");

            self.last_interval = Instant::now();
        }
    }
}

pub struct ActuatorService {
    net: Arc<ControlNet>,
    node: u8,
    last_interval: Instant,
    actuator_set: Option<std::collections::HashMap<u8, i16>>,
}

impl ActuatorService {
    pub fn new(net: Arc<ControlNet>, node: u8) -> Self {
        Self {
            net,
            node,
            last_interval: Instant::now(),
            actuator_set: None,
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
