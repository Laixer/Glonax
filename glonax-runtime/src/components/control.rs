use crate::{
    core::{Control, Object},
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct ControlComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for ControlComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
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
            } else {
                let control_command = Control::MachineShutdown;
                if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                    log::error!("Failed to send control command: {}", e);
                }
            }

            return;
        }

        if let Some(control_command) = ctx.machine.control_command {
            if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                log::error!("Failed to send control command: {}", e);
            }
        }
    }
}
