use glonax::{
    robot::ActorSegment,
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::Vector3;

pub struct Vehicle;

impl<Cnf: Configurable> Component<Cnf> for Vehicle {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, _state: &mut MachineState) {
        ctx.actor.attach_segment(
            "undercarriage",
            ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
        );
        ctx.actor
            .attach_segment("body", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)));
        ctx.actor
            .attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)));
        ctx.actor
            .attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)));
        ctx.actor.attach_segment(
            "bucket",
            ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
        );
    }
}
