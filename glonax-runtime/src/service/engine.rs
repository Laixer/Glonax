use rand::Rng;

use crate::runtime::{MotionSender, Service, SharedOperandState};

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

    async fn tick(&mut self, runtime_state: SharedOperandState, _command_tx: MotionSender) {
        let mut runtime_state = runtime_state.write().await;
        runtime_state.state.engine_signal.driver_demand = self.rng.gen_range(18..=20);
        runtime_state.state.engine_signal.actual_engine = self.rng.gen_range(19..=21);
        runtime_state.state.engine_signal.rpm = self.rng.gen_range(1180..=1200);
    }
}
