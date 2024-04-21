use rand::Rng;

use crate::runtime::{Service, SharedOperandState};

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

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        let mut runtime_state = runtime_state.write().await;
        runtime_state.state.engine.driver_demand = self.rng.gen_range(18..=20);
        runtime_state.state.engine.actual_engine = self.rng.gen_range(19..=21);
        runtime_state.state.engine.rpm = self.rng.gen_range(1180..=1200);
    }
}
