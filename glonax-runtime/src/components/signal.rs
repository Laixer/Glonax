use crate::{
    core::Object,
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

pub struct SignalComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for SignalComponent {
    fn new(_: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn finalize(
        &self,
        ctx: &mut ComponentContext,
        _command_tx: CommandSender,
        signal_tx: std::rc::Rc<SignalSender>,
    ) {
        if ctx.machine.motion_signal_set {
            if let Err(e) = signal_tx.send(Object::Motion(ctx.machine.motion_signal.clone())) {
                log::error!("Failed to send engine signal: {}", e);
            }
        }
        if ctx.machine.engine_signal_set {
            if let Err(e) = signal_tx.send(Object::Engine(ctx.machine.engine_signal)) {
                log::error!("Failed to send engine signal: {}", e);
            }
        }
        if ctx.machine.vms_signal_set {
            if let Err(e) = signal_tx.send(Object::Host(ctx.machine.vms_signal)) {
                log::error!("Failed to send host signal: {}", e);
            }
        }
        if ctx.machine.gnss_signal_set {
            if let Err(e) = signal_tx.send(Object::GNSS(ctx.machine.gnss_signal)) {
                log::error!("Failed to send gnss signal: {}", e);
            }
        }
    }
}
