use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::net::{ControlNet, EngineService};

pub struct Vecu {
    engine_service: EngineService,
}

impl Vecu {
    pub fn new(_net: Arc<ControlNet>, _publisher: crate::signal::SignalPublisher) -> Self {
        Self {
            engine_service: EngineService::new(0x0),
        }
    }
}

#[async_trait::async_trait]
impl crate::net::Routable for Vecu {
    fn node(&self) -> u8 {
        0x0
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == glonax_j1939::PGN::ElectronicEngineController2 {
            self.engine_service
                .ingress(glonax_j1939::PGN::ElectronicEngineController2, frame);

            debug!("RPM: {}", self.engine_service.rpm().unwrap());
            true
        } else {
            false
        }
    }
}
