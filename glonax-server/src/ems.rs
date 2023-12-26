use glonax::{net::EngineMessage, runtime::Service, RobotState};
use rand::Rng;

use crate::state::Excavator;

pub struct EngineSimService {
    rng: rand::rngs::OsRng,
}

impl Service<Excavator> for EngineSimService {
    fn run(&mut self, state: &mut Excavator) {
        EngineMessage {
            driver_demand: Some(self.rng.gen_range(18..=20)),
            actual_engine: Some(self.rng.gen_range(19..=21)),
            rpm: Some(self.rng.gen_range(1180..=1200)),
            ..Default::default()
        }
        .fill2(state.engine_mut());
    }
}

impl Default for EngineSimService {
    fn default() -> Self {
        log::debug!("Starting EMS service");

        Self {
            rng: rand::rngs::OsRng,
        }
    }
}
