use std::time::Duration;

use crate::{
    core::{Engine, Object},
    driver::Governor,
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct EngineComponent {
    governor: Governor,
}

impl<Cnf: Clone> PostComponent<Cnf> for EngineComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {
            governor: Governor::new(800, 2_100, Duration::from_millis(2_000)),
        }
    }

    fn finalize(
        &self,
        ctx: &mut ComponentContext,
        command_tx: CommandSender,
        _signal_tx: std::rc::Rc<SignalSender>,
    ) {
        if ctx.machine.emergency {
            if let Err(e) = command_tx.try_send(Object::Engine(Engine::shutdown())) {
                log::error!("Failed to send engine command: {}", e);
            }

            return;
        }

        let engine_signal = ctx.machine.engine_signal;
        let governor_engine = self.governor.next_state(
            &engine_signal,
            &ctx.machine.engine_command.unwrap_or(engine_signal),
            ctx.machine.engine_command_instant,
        );

        log::trace!("Engine governor: {:?}", governor_engine);

        if let Err(e) = command_tx.try_send(Object::Engine(governor_engine)) {
            log::error!("Failed to send engine command: {}", e);
        }
    }
}
