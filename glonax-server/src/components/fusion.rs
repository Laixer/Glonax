use glonax::{
    driver::EncoderConverter,
    runtime::{CommandSender, Component, ComponentContext},
};

const FRAME_ENCODER: u8 = 0x6A;
const BOOM_ENCODER: u8 = 0x6B;
const ARM_ENCODER: u8 = 0x6C;
const ATTACHMENT_ENCODER: u8 = 0x6D;

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

// TODO: Rename to encoder?
pub struct SensorFusion {
    frame_encoder_converter: EncoderConverter,
    boom_encoder_converter: EncoderConverter,
    arm_encoder_converter: EncoderConverter,
    attachment_encoder_converter: EncoderConverter,
}

impl<Cnf: Clone> Component<Cnf> for SensorFusion {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        let frame_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::z_axis());

        let boom_encoder_converter = EncoderConverter::new(
            1000.0,
            60_f32.to_radians(),
            true,
            nalgebra::Vector3::y_axis(),
        );

        let arm_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());

        let attachment_encoder_converter =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());

        Self {
            frame_encoder_converter,
            boom_encoder_converter,
            arm_encoder_converter,
            attachment_encoder_converter,
        }
    }

    fn tick(&mut self, ctx: &mut ComponentContext, _command_tx: CommandSender) {
        let actor = ctx.world.get_actor_by_name_mut(ROBOT_ACTOR_NAME).unwrap();

        if let Some(value) = ctx.machine.encoders.get(&FRAME_ENCODER) {
            log::trace!("Frame encoder: {}", value);

            // FUTURE: Detect outlying values, apply a filter

            let rotator = self.frame_encoder_converter.to_rotation(*value);

            log::trace!(
                "Frame: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("frame", rotator);
        }

        if let Some(value) = ctx.machine.encoders.get(&BOOM_ENCODER) {
            log::trace!("Boom encoder: {}", value);

            // FUTURE: Detect outlying values, apply a filter

            let rotator = self.boom_encoder_converter.to_rotation(*value);

            log::trace!(
                "Boom: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("boom", rotator);
        }

        if let Some(value) = ctx.machine.encoders.get(&ARM_ENCODER) {
            log::trace!("Arm encoder: {}", value);

            // FUTURE: Detect outlying values, apply a filter

            let rotator = self.arm_encoder_converter.to_rotation(*value);

            log::trace!(
                "Arm: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("arm", rotator);
        }

        if let Some(value) = ctx.machine.encoders.get(&ATTACHMENT_ENCODER) {
            log::trace!("Attachment encoder: {}", value);

            // FUTURE: Detect outlying values, apply a filter

            let rotator = self.attachment_encoder_converter.to_rotation(*value);

            log::trace!(
                "Attachment: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                rotator.euler_angles().0.to_degrees(),
                rotator.euler_angles().1.to_degrees(),
                rotator.euler_angles().2.to_degrees()
            );

            actor.set_relative_rotation("attachment", rotator);
        }
    }
}
