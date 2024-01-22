use glonax::{
    math::EulerAngles,
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct SinusTest;

impl<Cnf: Configurable> Component<Cnf> for SinusTest {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, _state: &mut MachineState) {
        let actor = ctx.world.get_actor_by_name_mut(ROBOT_ACTOR_NAME).unwrap();

        // TODO: Multiply by time delta
        actor.add_relative_rotation("frame", nalgebra::Rotation3::from_yaw(0.1_f32.to_radians()));
    }
}
