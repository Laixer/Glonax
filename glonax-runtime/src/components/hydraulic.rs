use crate::{
    core::{Motion, Object},
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct HydraulicComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for HydraulicComponent {
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
            let motion_command = Motion::StopAll;
            if let Err(e) = command_tx.try_send(Object::Motion(motion_command)) {
                log::error!("Failed to send motion command: {}", e);
            }

            return;
        }

        if let Some(motion_command) = &ctx.machine.motion_command {
            if let Err(e) = command_tx.try_send(Object::Motion(motion_command.clone())) {
                log::error!("Failed to send motion command: {}", e);
            }
        }
    }
}
