use glonax::{
    math::EulerAngles,
    runtime::{Component, ComponentContext},
};

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct Example;

impl<Cnf: Clone> Component<Cnf> for Example {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext) {
        let delta = ctx.delta();

        let actor = ctx.world.get_actor_by_name_mut(ROBOT_ACTOR_NAME).unwrap();

        if delta.as_millis() < 100 {
            actor.add_relative_rotation(
                "frame",
                nalgebra::Rotation3::from_yaw(delta.as_secs_f32() * 2.5_f32.to_radians()),
            );
        }

        actor.set_relative_rotation("boom", nalgebra::Rotation3::from_pitch(25_f32.to_radians()));
        actor.set_relative_rotation(
            "arm",
            nalgebra::Rotation3::from_pitch(-100_f32.to_radians()),
        );
    }
}
