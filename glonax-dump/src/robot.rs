use glonax::robot::{
    Device, DeviceType, Joint, JointType, MotionProfile, Robot, RobotBuilder, RobotType,
};

pub(crate) fn excavator(config: &crate::config::DumpConfig) -> Robot {
    RobotBuilder::new(config.instance.id.clone(), RobotType::Excavator)
        .model(config.instance.model.clone())
        .name(config.instance.name.clone())
        .add_device(Device::new(
            "frame_encoder",
            0x6A,
            DeviceType::EncoderAbsoluteMultiTurn,
        ))
        .add_device(Device::new(
            "boom_encoder",
            0x6B,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_device(Device::new(
            "arm_encoder",
            0x6C,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_device(Device::new(
            "attachment_encoder",
            0x6D,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_joint(Joint::new("undercarriage", JointType::Fixed))
        .add_joint(
            Joint::with_actuator(
                "frame",
                JointType::Continuous,
                glonax::core::Actuator::Slew,
                MotionProfile::new(7_000.0, 12_000.0, 0.01, false),
            )
            .set_height(1.295),
        )
        .add_joint(
            Joint::with_actuator(
                "boom",
                JointType::Revolute,
                glonax::core::Actuator::Boom,
                MotionProfile::new(15_000.0, 12_000.0, 0.01, false),
            )
            .set_origin_translation(0.16, 0.0, 0.595)
            .set_pitch(-59.35_f32.to_radians())
            .set_bounds(-59.35_f32.to_radians(), 45.0_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "arm",
                JointType::Revolute,
                glonax::core::Actuator::Arm,
                MotionProfile::new(15_000.0, 12_000.0, 0.01, true),
            )
            .set_length(6.0)
            .set_bounds(38.96_f32.to_radians(), 158.14_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "attachment",
                JointType::Revolute,
                glonax::core::Actuator::Attachment,
                MotionProfile::new(15_000.0, 12_000.0, 0.05, false),
            )
            .set_length(2.97)
            .set_pitch(-55_f32.to_radians())
            .set_bounds(-55_f32.to_radians(), 125_f32.to_radians())
            .set_tolerance(0.05),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).set_length(1.5))
        .build()
}
