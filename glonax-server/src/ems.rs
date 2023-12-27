use glonax::{
    net::EngineMessage,
    runtime::{Component, ComponentContext},
    RobotState,
};
use rand::Rng;

pub struct EngineSimService {
    rng: rand::rngs::OsRng,
}

impl<R: RobotState> Component<R> for EngineSimService {
    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut R) {
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
