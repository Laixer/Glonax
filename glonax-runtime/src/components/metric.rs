use crate::runtime::{CommandSender, ComponentContext, PostComponent, SignalSender};

pub struct MetricComponent;

impl<Cnf: Clone> PostComponent<Cnf> for MetricComponent {
    fn new(_: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn finalize(
        &self,
        _ctx: &mut ComponentContext,
        _command_tx: CommandSender,
        _signal_tx: std::rc::Rc<SignalSender>,
    ) {
    }
}
