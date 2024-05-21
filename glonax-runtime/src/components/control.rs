use crate::runtime::{CommandSender, Component, ComponentContext};

pub struct ControlComponent {}

impl<Cnf: Clone> Component<Cnf> for ControlComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn tick(
        &mut self,
        _ctx: &mut ComponentContext,
        _ipc_rx: std::rc::Rc<crate::runtime::IPCReceiver>,
        _command_tx: CommandSender,
    ) {
        // TODO: Implement the control logic
    }
}
