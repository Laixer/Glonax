use glonax::{
    core::MachineType,
    runtime::ComponentContext,
    world::{ActorBuilder, ActorSegment},
};
use nalgebra::Vector3;

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub fn construct(ctx: &mut ComponentContext) {
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

    ctx.world.add_actor(actor);
}
