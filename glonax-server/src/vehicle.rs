use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct Vehicle;

impl<Cnf: Configurable> Component<Cnf> for Vehicle {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO: Calculate the error between the target and the inverse kinematics
    fn tick(&mut self, _ctx: &mut ComponentContext, _state: &mut MachineState) {
        //
    }
}
