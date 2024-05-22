use crate::{
    core::Object,
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
        if let Some(control_command) = ctx.machine.control_command {
            if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                log::error!("Failed to send control command: {}", e);
            }
        }
    }
}
