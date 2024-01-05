use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::Point3;

const ACTOR_SELF: usize = 0;

pub struct Kinematic;

impl<Cnf: Configurable> Component<Cnf> for Kinematic {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO: Move the IK into a helper function
    // TODO: Store if target is reachable in the context, if there is a target
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        // TODO: Move to setup?
        // Set relative rotations
        {
            let actor = ctx.world_mut().get_actor_mut(ACTOR_SELF).unwrap();

            // actor.add_relative_rotation(
            //     "body",
            //     nalgebra::Rotation3::from_euler_angles(0.0, 0.0, 0.1_f32.to_radians()),
            // );

            {
                // let r = state.pose.frame_rotator.euler_angles().0.to_degrees();
                // let p = state.pose.frame_rotator.euler_angles().1.to_degrees();
                // let y = state.pose.frame_rotator.euler_angles().2.to_degrees();
                // log::debug!("Body: Roll: {:.2} Pitch: {:.2} Yaw: {:.2}", r, p, y);

                if let Some(value) = state.encoders.get(&0x6A) {
                    log::trace!("Body encoder: {}", value);

                    let frame_enc_conv = glonax::device::EncoderConverter::new(
                        1000.0,
                        0.0,
                        true,
                        nalgebra::Vector3::z_axis(),
                    );

                    let rotator = frame_enc_conv.to_rotation(*value as u32);

                    log::debug!(
                        "Body: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                        rotator.euler_angles().0.to_degrees(),
                        rotator.euler_angles().1.to_degrees(),
                        rotator.euler_angles().2.to_degrees()
                    );

                    actor.set_relative_rotation("body", rotator);
                }
            }

            {
                // let r = state.pose.boom_rotator.euler_angles().0.to_degrees();
                // let p = state.pose.boom_rotator.euler_angles().1.to_degrees();
                // let y = state.pose.boom_rotator.euler_angles().2.to_degrees();
                // log::debug!("Boom: Roll: {:.2} Pitch: {:.2} Yaw: {:.2}", r, p, y);

                if let Some(value) = state.encoders.get(&0x6B) {
                    log::trace!("Boom encoder: {}", value);

                    // let offset = 60_f32.to_radians();
                    // let position = position as f32 / 1000.0;
                    // let position = (position - offset) * -1.0;
                    // let rotator = Rotation3::from_euler_angles(0.0, position, 0.0);

                    // log::debug!(
                    //     "Boom: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                    //     rotator.euler_angles().0.to_degrees(),
                    //     rotator.euler_angles().1.to_degrees(),
                    //     rotator.euler_angles().2.to_degrees()
                    // );

                    // actor.set_relative_rotation("boom", rotator);
                }
            }

            {
                // let r = state.pose.arm_rotator.euler_angles().0.to_degrees();
                // let p = state.pose.arm_rotator.euler_angles().1.to_degrees();
                // let y = state.pose.arm_rotator.euler_angles().2.to_degrees();
                // log::debug!("Arm: Roll: {:.2} Pitch: {:.2} Yaw: {:.2}", r, p, y);

                if let Some(value) = state.encoders.get(&0x6C) {
                    log::trace!("Arm encoder: {}", value);

                    let arm_enc_conv = glonax::device::EncoderConverter::new(
                        1000.0,
                        0.0,
                        true,
                        nalgebra::Vector3::y_axis(),
                    );

                    let rotator = arm_enc_conv.to_rotation(*value as u32);

                    log::debug!(
                        "Arm: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                        rotator.euler_angles().0.to_degrees(),
                        rotator.euler_angles().1.to_degrees(),
                        rotator.euler_angles().2.to_degrees()
                    );

                    actor.set_relative_rotation("arm", rotator);
                }
            }

            {
                // let r = state.pose.attachment_rotator.euler_angles().0.to_degrees();
                // let p = state.pose.attachment_rotator.euler_angles().1.to_degrees();
                // let y = state.pose.attachment_rotator.euler_angles().2.to_degrees();
                // log::debug!("Bucket: Roll: {:.2} Pitch: {:.2} Yaw: {:.2}", r, p, y);

                if let Some(value) = state.encoders.get(&0x6D) {
                    log::trace!("Attachment encoder: {}", value);

                    let attachment_enc_conv = glonax::device::EncoderConverter::new(
                        1000.0,
                        0.0,
                        true,
                        nalgebra::Vector3::y_axis(),
                    );

                    let rotator = attachment_enc_conv.to_rotation(*value as u32);

                    log::debug!(
                        "Attachment: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                        rotator.euler_angles().0.to_degrees(),
                        rotator.euler_angles().1.to_degrees(),
                        rotator.euler_angles().2.to_degrees()
                    );

                    actor.set_relative_rotation("bucket", rotator);
                }
            }

            // actor.set_relative_rotation("body", state.pose.frame_rotator);
            // actor.set_relative_rotation("boom", state.pose.boom_rotator);
            // actor.set_relative_rotation("arm", state.pose.arm_rotator);
            // actor.set_relative_rotation("bucket", state.pose.attachment_rotator);
        }

        // Print segment world locations
        {
            let actor = ctx.world().get_actor(ACTOR_SELF).unwrap();

            let body_world_location = actor.world_location("body");
            log::debug!(
                "Body: world location: X={:.2} Y={:.2} Z={:.2}",
                body_world_location.x,
                body_world_location.y,
                body_world_location.z
            );

            let boom_world_location = actor.world_location("boom");
            log::debug!(
                "Boom: world location: X={:.2} Y={:.2} Z={:.2}",
                boom_world_location.x,
                boom_world_location.y,
                boom_world_location.z
            );

            let arm_world_location = actor.world_location("arm");
            log::debug!(
                "Arm: world location: X={:.2} Y={:.2} Z={:.2}",
                arm_world_location.x,
                arm_world_location.y,
                arm_world_location.z
            );

            let bucket_world_location = actor.world_location("bucket");
            log::debug!(
                "Bucket: world location: X={:.2} Y={:.2} Z={:.2}",
                bucket_world_location.x,
                bucket_world_location.y,
                bucket_world_location.z
            );
        }

        /////////////// IF THERE IS A TARGET ///////////////

        // Print distances
        if let Some(target) = &state.target {
            let actor = ctx.world().get_actor(ACTOR_SELF).unwrap();

            let actor_world_distance =
                nalgebra::distance(&actor.location(), &Point3::new(0.0, 0.0, 0.0));
            log::debug!("Actor world distance: {:.2}", actor_world_distance);

            let actor_target_distance = nalgebra::distance(&actor.location(), &target.point);
            log::debug!("Actor target distance: {:.2}", actor_target_distance);

            let boom_point = actor.relative_location("boom").unwrap();

            let kinematic_target_distance =
                nalgebra::distance(&actor.location(), &(target.point - boom_point.coords));
            log::debug!(
                "Kinematic target distance: {:.2}",
                kinematic_target_distance
            );
        }

        if let Some(target) = &state.target {
            let actor = ctx.world().get_actor(ACTOR_SELF).unwrap();

            let boom_length = actor.relative_location("arm").unwrap().x;
            // log::debug!("Boom length: {:?}", boom_length);

            let arm_length = actor.relative_location("bucket").unwrap().x;
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
