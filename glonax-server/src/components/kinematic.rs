use glonax::runtime::{CommandSender, Component, ComponentContext};
use nalgebra::Point3;

// TODO: Calculate this from the actor
const MAX_KINEMATIC_DISTANCE: f32 = 700.0;
// TODO: Get this from config
const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct Kinematic {
    target: Option<glonax::core::Target>, // TOD: This is a temporary solution, get the target from the world
}

impl<Cnf: Clone> Component<Cnf> for Kinematic {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self { target: None }
    }

    // TODO: Move the IK into a helper function
    fn tick(&mut self, ctx: &mut ComponentContext, _command_tx: CommandSender) {
        let actor = ctx.world.get_actor_by_name(ROBOT_ACTOR_NAME).unwrap();

        if let Some(target) = &self.target {
            log::debug!("Objective target: {}", target);

            let actor_world_distance =
                nalgebra::distance(&actor.location(), &Point3::new(0.0, 0.0, 0.0));
            log::debug!("Actor origin distance: {:.2}", actor_world_distance);

            let actor_target_distance = nalgebra::distance(&actor.location(), &target.point);
            log::debug!("Actor target distance: {:.2}", actor_target_distance);

            let boom_point = actor.relative_location("boom").unwrap();
            let kinematic_target_distance =
                nalgebra::distance(&actor.location(), &(target.point - boom_point.coords));
            log::debug!(
                "Kinematic target distance: {:.2}",
                kinematic_target_distance
            );

            if kinematic_target_distance > MAX_KINEMATIC_DISTANCE {
                log::warn!("Target is out of reach");
            }
        }

        if let Some(target) = &self.target {
            let boom_length = actor.relative_location("arm").unwrap().x;
            // log::debug!("Boom length: {:?}", boom_length);

            let arm_length = actor.relative_location("attachment").unwrap().x;
            // log::debug!("Arm length: {:?}", arm_length);

            let boom_world_location = actor.world_location("boom");

            let target_distance = nalgebra::distance(&boom_world_location, &target.point);
            log::debug!("Tri-Arm target distance: {:.2}", target_distance);

            let target_direction = (target.point.coords - boom_world_location.coords).normalize();

            /////////////// SLEW YAW ANGLE ///////////////

            let slew_angle = target_direction.y.atan2(target_direction.x);
            log::debug!(
                "  Slew angle: {:.3}rad {:.2}deg",
                slew_angle,
                slew_angle.to_degrees()
            );

            ctx.actuators
                .insert(glonax::core::Actuator::Slew as u16, slew_angle);

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

            ctx.actuators
                .insert(glonax::core::Actuator::Boom as u16, boom_angle);

            /////////////// ARM PITCH ANGLE ///////////////

            let theta0 = glonax::math::law_of_cosines(boom_length, arm_length, target_distance);
            // log::debug!("Theta0: {}rad {}deg", theta0, theta0.to_degrees());

            let arm_angle = -(std::f32::consts::PI - theta0);
            log::debug!(
                "  Arm angle: {:.3}rad {:.2}deg",
                arm_angle,
                arm_angle.to_degrees()
            );

            ctx.actuators
                .insert(glonax::core::Actuator::Arm as u16, arm_angle);
        }
    }
}
