use crate::{
    core::Object,
    runtime::{CommandSender, ComponentContext, PostComponent},
};

pub struct ControlComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for ControlComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn finalize(&self, ctx: &mut ComponentContext, command_tx: CommandSender) {
        if let Some(control_command) = ctx.machine.control_command {
            if let Err(e) = command_tx.try_send(Object::Control(control_command)) {
                log::error!("Failed to send control command: {}", e);
            }
        }
    }
}
