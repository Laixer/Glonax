use glonax::{
    core::MachineType,
    runtime::{Component, ComponentContext, MotionSender},
    world::{Actor, ActorBuilder, ActorSegment},
    MachineState,
};
use nalgebra::Vector3;

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct WorldBuilder {
    actor: Actor,
    is_actor_attached: bool,
}

impl<Cnf: Clone> Component<Cnf> for WorldBuilder {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the actor from configuration and machine instance
        let actor = ActorBuilder::new(ROBOT_ACTOR_NAME, MachineType::Excavator)
            .attach_segment(
                "undercarriage",
                ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
            )
            .attach_segment("frame", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)))
            .attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)))
            .attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)))
            .attach_segment(
                "attachment",
                ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
            )
            .build();

        Self {
            actor,
            is_actor_attached: false,
        }
    }

    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        state: &mut MachineState,
        _command_tx: MotionSender,
    ) {
        if !self.is_actor_attached {
            ctx.world.add_actor(self.actor.clone());
            self.is_actor_attached = true;
        }

        // TODO: For now
        if ctx.target.is_none() && !state.program.is_empty() {
            ctx.target = state.program.pop_front();
        }
    }
}
