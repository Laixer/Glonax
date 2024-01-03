use glonax::{
    robot::{Actor, ActorBuilder, ActorSegment},
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::Vector3;

pub struct World {
    actor: Actor,
}

impl<Cnf: Configurable> Component<Cnf> for World {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the actor from configuration
        let actor = ActorBuilder::default()
            .attach_segment(
                "undercarriage",
                ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
            )
            .attach_segment("body", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)))
            .attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)))
            .attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)))
            .attach_segment(
                "bucket",
                ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
            )
            .build();

        Self { actor }
    }

    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        ctx.replace_actor(self.actor.clone());

        state.target = Some(glonax::core::Target::from_point(300.0, 400.0, 330.0));
    }
}
