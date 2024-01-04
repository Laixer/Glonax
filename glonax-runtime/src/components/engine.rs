use rand::Rng;

use crate::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct EngineSimulator {
    rng: rand::rngs::OsRng,
}

impl<Cnf: Configurable> Component<Cnf> for EngineSimulator {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting EMS component");

        Self {
            rng: rand::rngs::OsRng,
        }
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut MachineState) {
        state.engine.driver_demand = self.rng.gen_range(18..=20);
        state.engine.actual_engine = self.rng.gen_range(19..=21);
        state.engine.rpm = self.rng.gen_range(1180..=1200);
    }
}
