use rand::Rng;

use crate::runtime::{Service, SharedOperandState};

pub struct EngineSimulator {
    rng: rand::rngs::OsRng,
}

impl<Cnf> Service<Cnf> for EngineSimulator {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting EMS component");

        Self {
            rng: rand::rngs::OsRng,
        }
    }

    fn tick(&mut self, runtime_state: SharedOperandState) {
        if let Ok(mut runtime_state) = runtime_state.try_write() {
            runtime_state.state.engine.driver_demand = self.rng.gen_range(18..=20);
            runtime_state.state.engine.actual_engine = self.rng.gen_range(19..=21);
            runtime_state.state.engine.rpm = self.rng.gen_range(1180..=1200);
        }
    }
}
