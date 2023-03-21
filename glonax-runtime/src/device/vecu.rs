use glonax_j1939::{Frame, PGN};

use crate::{
    net::EngineService,
    transport::{signal::Metric, Signal},
};

pub struct Vecu {
    writer: crate::signal::SignalQueueWriter,
    engine_service: EngineService,
}

impl Vecu {
    pub fn new(writer: crate::signal::SignalQueueWriter) -> Self {
        Self {
            writer,
            engine_service: EngineService::new(0x0),
        }
    }
}

impl crate::net::Routable for Vecu {
    fn node(&self) -> u8 {
        self.engine_service.node()
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == glonax_j1939::PGN::ElectronicEngineController2 {
            self.engine_service
                .ingress(glonax_j1939::PGN::ElectronicEngineController2, frame);

            if let Some(electronic_control) = self.engine_service.electronic_control() {
                self.writer
                    .send(Signal::new(0x5, Metric::Rpm(electronic_control.rpm as i32)));
            }

            true
        } else {
            false
        }
    }
}
