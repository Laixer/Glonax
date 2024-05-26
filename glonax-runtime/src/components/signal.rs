use crate::{
    core::Object,
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct SignalComponent;

impl<Cnf: Clone> PostComponent<Cnf> for SignalComponent {
    fn new(_: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    // FUTURE: This can be optimized by only sending signals that have changed.
    fn finalize(
        &self,
        ctx: &mut ComponentContext,
        _command_tx: CommandSender,
        signal_tx: std::rc::Rc<SignalSender>,
    ) {
        let mut signal_list = Vec::new();

        if ctx.machine.motion_signal_set {
            signal_list.push(Object::Motion(ctx.machine.motion_signal.clone()));
        }
        if ctx.machine.engine_signal_set {
            signal_list.push(Object::Engine(ctx.machine.engine_signal));
        }
        if ctx.machine.vms_signal_set {
            signal_list.push(Object::Host(ctx.machine.vms_signal));
        }
        if ctx.machine.gnss_signal_set {
            signal_list.push(Object::GNSS(ctx.machine.gnss_signal));
        }

        for signal in signal_list.drain(..) {
            if let Err(e) = signal_tx.send(signal) {
                log::error!("Failed to send signal: {}", e);
            }
        }
    }
}
