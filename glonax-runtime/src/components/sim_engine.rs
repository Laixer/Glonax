use rand::{rngs::OsRng, Rng};

use crate::runtime::{CommandSender, Component, ComponentContext};

pub struct EngineSimulator {
    rng: OsRng,
}

impl<Cnf: Clone> Component<Cnf> for EngineSimulator {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self { rng: OsRng }
    }

    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        _ipc_rx: std::rc::Rc<crate::runtime::IPCReceiver>,
        _command_tx: CommandSender,
    ) {
        let engine_signal = crate::core::Engine {
            driver_demand: self.rng.gen_range(18..=20),
            actual_engine: self.rng.gen_range(19..=21),
            rpm: self.rng.gen_range(1180..=1200),
            state: crate::core::EngineState::Request,
        };

        ctx.machine.engine_signal = engine_signal;
        ctx.machine.engine_signal_instant = Some(std::time::Instant::now());
    }
}
