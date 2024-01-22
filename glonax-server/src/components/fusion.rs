use glonax::{
    device::EncoderConverter,
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

const FRAME_ENCODER: u8 = 0x6A;
const BOOM_ENCODER: u8 = 0x6B;
const ARM_ENCODER: u8 = 0x6C;
const ATTACHMENT_ENCODER: u8 = 0x6D;

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

pub struct SensorFusion {
    frame_encoder_converter: EncoderConverter,
    arm_encoder_converter: EncoderConverter,
    attachment_encoder_converter: EncoderConverter,
}

impl<Cnf: Configurable> Component<Cnf> for SensorFusion {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        let frame_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::z_axis());

        let arm_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());

        let attachment_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());

        Self {
            frame_encoder_converter,
            arm_encoder_converter,
            attachment_encoder_converter,
        }
    }

    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        let actor = ctx.world.get_actor_by_name_mut(ROBOT_ACTOR_NAME).unwrap();

        if let Some(value) = state.encoders.get(&FRAME_ENCODER) {
            log::trace!("Frame encoder: {}", value);

            let rotator = self.frame_encoder_converter.to_rotation(*value as u32);

            log::debug!(
                "Frame: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("frame", rotator);
        }

        if let Some(value) = state.encoders.get(&BOOM_ENCODER) {
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

        if let Some(value) = state.encoders.get(&ARM_ENCODER) {
            log::trace!("Arm encoder: {}", value);

            let rotator = self.arm_encoder_converter.to_rotation(*value as u32);

            log::debug!(
                "Arm: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("arm", rotator);
        }

        if let Some(value) = state.encoders.get(&ATTACHMENT_ENCODER) {
            log::trace!("Attachment encoder: {}", value);

            let rotator = self.attachment_encoder_converter.to_rotation(*value as u32);

            log::debug!(
                "Attachment: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("attachment", rotator);
        }

        // Print segment world locations
        let body_world_location = actor.world_location("frame");
        log::trace!(
            "Frame: X={:.2} Y={:.2} Z={:.2}",
            body_world_location.x,
            body_world_location.y,
            body_world_location.z
        );

        let boom_world_location = actor.world_location("boom");
        log::trace!(
            "Boom: X={:.2} Y={:.2} Z={:.2}",
            boom_world_location.x,
            boom_world_location.y,
            boom_world_location.z
        );

        let arm_world_location = actor.world_location("arm");
        log::trace!(
            "Arm: X={:.2} Y={:.2} Z={:.2}",
            arm_world_location.x,
            arm_world_location.y,
            arm_world_location.z
        );

        let bucket_world_location = actor.world_location("attachment");
        log::trace!(
            "Attachment: X={:.2} Y={:.2} Z={:.2}",
            bucket_world_location.x,
            bucket_world_location.y,
            bucket_world_location.z
        );
    }
}
