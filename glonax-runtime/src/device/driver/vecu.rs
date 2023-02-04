use glonax_j1939::{Frame, PGN};

use crate::{
    core::metric::{MetricValue, Signal},
    net::EngineService,
    signal::SignalPublisher,
};

pub struct Vecu {
    publisher: SignalPublisher,
    engine_service: EngineService,
}

impl Vecu {
    pub fn new(publisher: SignalPublisher) -> Self {
        Self {
            publisher,
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

            if let Some(rpm) = self.engine_service.rpm() {
                trace!("Engine RPM: {}", rpm);

                self.publisher.try_publish(
                    "signal",
                    Signal {
                        address: self.engine_service.node(),
                        subaddress: 0,
                        value: MetricValue::RPM(rpm),
                    },
                );
            }

            if let Some(electronic_control) = self.engine_service.electronic_control() {
                self.publisher
                    .try_publish("engine/power", electronic_control);
            }

            true
        } else {
            false
        }
    }
}
