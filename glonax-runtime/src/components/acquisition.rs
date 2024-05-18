use crate::{
    runtime::{CommandSender, Component, ComponentContext},
    MachineState,
};

pub struct Acquisition {}

impl<Cnf: Clone> Component<Cnf> for Acquisition {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        _state: &mut MachineState,
        _command_tx: CommandSender,
    ) {
        // TODO: Acquire sensor data
        // TODO: Transmit the sensor data to the server peers

        log::debug!("Acquisition tick, number of objects: {}", ctx.signals.len());
    }
}
