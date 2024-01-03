use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::Point3;

pub struct Kinematic;

impl<Cnf: Configurable> Component<Cnf> for Kinematic {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO: Store if target is reachable in the context, if there is a target
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        // Set actor location
        {
            let _actor = ctx.actor_mut();

            // actor.set_location(Vector3::new(80.0, 0.0, 0.0));
        }

        // Set relative rotations
        {
            let actor = ctx.actor_mut();

            actor.set_relative_rotation("body", state.pose.frame_rotator);
            actor.set_relative_rotation("boom", state.pose.boom_rotator);
            actor.set_relative_rotation("arm", state.pose.arm_rotator);
            actor.set_relative_rotation("bucket", state.pose.attachment_rotator);
        }

        // Print segment world locations
        {
            let actor = ctx.actor_mut();

            let body_world_location = actor.world_location("body");
            log::debug!("FRAM world location: {:?}", body_world_location);

            let boom_world_location = actor.world_location("boom");
            log::debug!("BOOM world location: {:?}", boom_world_location);

            let arm_world_location = actor.world_location("arm");
            log::debug!("ARMM world location: {:?}", arm_world_location);

            let bucket_world_location = actor.world_location("bucket");
            log::debug!("BUKT world location: {:?}", bucket_world_location);
        }

        /////////////// IF THERE IS A TARGET ///////////////

        if let Some(target) = &state.target {
            let actor = ctx.actor_mut();

            let actor_target_distance = nalgebra::distance(&actor.location().into(), &target.point);
            log::debug!("Actor target distance: {}", actor_target_distance);

            let boom_vector = actor.relative_location("boom").unwrap().vector;

            let tt = target.point - boom_vector;

            let kinematic_target_distance = nalgebra::distance(&actor.location().into(), &tt);
            log::debug!("Kinematic target distance: {}", kinematic_target_distance);
        }

        if let Some(target) = &state.target {
            let actor = ctx.actor_mut();

            let boom_length = actor.relative_location("arm").unwrap().vector.x;
            // log::debug!("Boom length: {:?}", boom_length);

            let arm_length = actor.relative_location("bucket").unwrap().vector.x;
            // log::debug!("Arm length: {:?}", arm_length);

            let boom_world_location = actor.world_location("boom");

            let target_distance = nalgebra::distance(&boom_world_location, &target.point);
            log::debug!("Tri-Arm target distance: {}", target_distance);

            let target_direction = (target.point.coords - boom_world_location.coords).normalize();

            /////////////// SLEW YAW ANGLE ///////////////

            let slew_angle = target_direction.y.atan2(target_direction.x);
            log::debug!(
                "  Slew angle: {:.3}rad {:.2}deg",
                slew_angle,
                slew_angle.to_degrees()
            );

            ctx.map(glonax::core::Actuator::Slew as u16, slew_angle);

            /////////////// BOOM PITCH ANGLE ///////////////

            let pitch = target_direction
                .z
                .atan2((target_direction.x.powi(2) + target_direction.y.powi(2)).sqrt());
            // log::debug!("Pitch: {}deg", pitch.to_degrees());

            let theta1 = glonax::math::law_of_cosines(boom_length, target_distance, arm_length);
            // log::debug!("Theta1: {}rad {}deg", theta1, theta1.to_degrees());

            let boom_angle = theta1 + pitch;
            log::debug!(
                "  Boom angle: {:.3}rad {:.2}deg",
                boom_angle,
                boom_angle.to_degrees()
            );

            ctx.map(glonax::core::Actuator::Boom as u16, boom_angle);

            /////////////// ARM PITCH ANGLE ///////////////

            let theta0 = glonax::math::law_of_cosines(boom_length, arm_length, target_distance);
            // log::debug!("Theta0: {}rad {}deg", theta0, theta0.to_degrees());

            let arm_angle = -(std::f32::consts::PI - theta0);
            log::debug!(
                "  Arm angle: {:.3}rad {:.2}deg",
                arm_angle,
                arm_angle.to_degrees()
            );

            ctx.map(glonax::core::Actuator::Arm as u16, arm_angle);
        }
    }
}
