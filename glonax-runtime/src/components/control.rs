use std::time::Duration;

use crate::{
    core::{Control, Engine, Motion, Object},
    driver::Governor,
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct ControlComponent {
    governor: Governor,
}

impl<Cnf: Clone> PostComponent<Cnf> for ControlComponent {
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
            if ctx.machine.engine_signal.rpm > 0 {
                let control_command = Control::HydraulicLock(true);
                if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                    log::error!("Failed to send control command: {}", e);
                }

                let motion_command = Motion::StopAll;
                if let Err(e) = command_tx.try_send(Object::Motion(motion_command)) {
                    log::error!("Failed to send motion command: {}", e);
                }

                let control_command = Control::HydraulicBoost(false);
                if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                    log::error!("Failed to send control command: {}", e);
                }

                let control_command = Control::MachineTravelAlarm(true);
                if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                    log::error!("Failed to send control command: {}", e);
                }

                let control_command = Control::MachineStrobeLight(true);
                if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                    log::error!("Failed to send control command: {}", e);
                }

                if let Err(e) = command_tx.try_send(Object::Engine(Engine::shutdown())) {
                    log::error!("Failed to send engine command: {}", e);
                }
            }

            return;
        }

        // if let Some(motion_command) = &ctx.machine.motion_command {
        //     if let Err(e) = command_tx.try_send(Object::Motion(motion_command.clone())) {
        //         log::error!("Failed to send motion command: {}", e);
        //     }
        // }

        if let Some(control_command) = ctx.machine.control_command {
            if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                log::error!("Failed to send control command: {}", e);
            }
        }

        // TODO: Move to planner or some other component
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
