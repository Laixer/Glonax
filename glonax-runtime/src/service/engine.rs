use rand::Rng;

use crate::runtime::{CommandSender, Service, SharedOperandState};

pub struct EngineSimulator {
    rng: rand::rngs::OsRng,
}

impl<C> Service<C> for EngineSimulator {
    fn new(_: C) -> Self
    where
        Self: Sized,
    {
        Self {
            rng: rand::rngs::OsRng,
        }
    }

    async fn tick(&mut self, _runtime_state: SharedOperandState, _command_tx: CommandSender) {
        let engine_signal = crate::core::Engine {
            driver_demand: self.rng.gen_range(18..=20),
            actual_engine: self.rng.gen_range(19..=21),
            rpm: self.rng.gen_range(1180..=1200),
            state: crate::core::EngineState::Request,
        };

        log::trace!("Engine signal: {:?}", engine_signal);
    }
}
