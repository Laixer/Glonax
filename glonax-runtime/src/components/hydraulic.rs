use crate::runtime::{CommandSender, ComponentContext, PostComponent};

pub struct HydraulicComponent {}

impl<Cnf: Clone> PostComponent<Cnf> for HydraulicComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn finalize(&self, _ctx: &mut ComponentContext, _command_tx: CommandSender) {
        // if let Err(e) = command_tx.try_send(Object::Engine(governor_engine)) {
        //     log::error!("Failed to send engine command: {}", e);
        // }
    }
}
