use glonax::{
    robot::{ActorBuilder, ActorSegment},
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::Vector3;

pub struct World;

impl<Cnf: Configurable> Component<Cnf> for World {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, _state: &mut MachineState) {
        ctx.replace_actor(
            ActorBuilder::new(vec![
                (
                    "undercarriage".to_string(),
                    ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
                ),
                (
                    "body".to_string(),
                    ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)),
                ),
                (
                    "boom".to_string(),
                    ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)),
                ),
                (
                    "arm".to_string(),
                    ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)),
                ),
                (
                    "bucket".to_string(),
                    ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
                ),
            ])
            .build(),
        );
    }
}
