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
        _ctx: &mut ComponentContext,
        _state: &mut MachineState,
        _command_tx: CommandSender,
    ) {
        // TODO: Transmit the sensor data to the server peers
    }
}
