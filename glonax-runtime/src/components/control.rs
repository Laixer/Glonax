use crate::runtime::{CommandSender, ComponentContext, PostComponent};

pub struct ControlComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for ControlComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn finalize(&self, _ctx: &mut ComponentContext, _command_tx: CommandSender) {
        // TODO: Implement the control logic
    }
}
