use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

// TODO: Get this from config
const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct LocalActor;

impl<Cnf: Configurable> Component<Cnf> for LocalActor {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        let actor = ctx.world.get_actor_by_name(ROBOT_ACTOR_NAME).unwrap();

        let body_world_location = actor.world_location("frame");
        log::debug!(
            "Frame: X={:.2} Y={:.2} Z={:.2}",
            body_world_location.x,
            body_world_location.y,
            body_world_location.z
        );

        let boom_world_location = actor.world_location("boom");
        log::debug!(
            "Boom: X={:.2} Y={:.2} Z={:.2}",
            boom_world_location.x,
            boom_world_location.y,
            boom_world_location.z
        );

        let arm_world_location = actor.world_location("arm");
        log::debug!(
            "Arm: X={:.2} Y={:.2} Z={:.2}",
            arm_world_location.x,
            arm_world_location.y,
            arm_world_location.z
        );

        let bucket_world_location = actor.world_location("attachment");
        log::debug!(
            "Attachment: X={:.2} Y={:.2} Z={:.2}",
            bucket_world_location.x,
            bucket_world_location.y,
            bucket_world_location.z
        );

        // TODO: This is a hack to get the actor into the state
        state.actor = Some(actor.clone());
    }
}
