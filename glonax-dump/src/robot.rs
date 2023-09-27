use glonax::robot::{
    Device, DeviceType, Joint, JointType, LinearMotionProfile, Robot, RobotBuilder, RobotType,
};
use na::{Translation3, UnitQuaternion};
use nalgebra as na;

use rapier3d::prelude::*;

pub struct Excavator {
    robot: Robot,
    pub boom_length: f32,
    pub arm_length: f32,
}

impl Excavator {
    fn new(robot: Robot) -> Self {
        let boom_length = robot
            .joint_by_name("boom")
            .unwrap()
            .origin()
            .translation
            .vector
            .x;
        let arm_length = robot
            .joint_by_name("attachment")
            .unwrap()
            .origin()
            .translation
            .vector
            .x;

        Self {
            robot,
            boom_length,
            arm_length,
        }
    }

    pub fn frame_device_id(&self) -> u8 {
        self.robot.device_by_name("frame_encoder").unwrap().id()
    }

    pub fn boom_device_id(&self) -> u8 {
        self.robot.device_by_name("boom_encoder").unwrap().id()
    }

    pub fn arm_device_id(&self) -> u8 {
        self.robot.device_by_name("arm_encoder").unwrap().id()
    }

    pub fn attachment_device_id(&self) -> u8 {
        self.robot
            .device_by_name("attachment_encoder")
            .unwrap()
            .id()
    }

    pub fn kinematic_chain(&self) -> glonax::robot::Chain {
        glonax::robot::Chain::new(self.robot.clone())
            .add_link("frame")
            .add_link("boom")
            .add_link("arm")
            .add_link("attachment")
    }
}

pub(crate) fn excavator(config: &crate::config::DumpConfig) -> Excavator {
    let robot = RobotBuilder::new(config.instance.id.clone(), RobotType::Excavator)
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
                LinearMotionProfile::new(7_000.0, 12_000.0, 0.01, false),
            )
            .set_height(1.295),
        )
        .add_joint(
            Joint::with_actuator(
                "boom",
                JointType::Revolute,
                glonax::core::Actuator::Boom,
                LinearMotionProfile::new(15_000.0, 12_000.0, 0.01, false),
            )
            .set_origin_translation(0.16, 0.0, 0.595)
            // .set_pitch(-59.35_f32.to_radians())
            .set_bounds(-59.35_f32.to_radians(), 45.0_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "arm",
                JointType::Revolute,
                glonax::core::Actuator::Arm,
                LinearMotionProfile::new(15_000.0, 12_000.0, 0.01, true),
            )
            .set_length(6.0)
            .set_bounds(38.96_f32.to_radians(), 158.14_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "attachment",
                JointType::Revolute,
                glonax::core::Actuator::Attachment,
                LinearMotionProfile::new(15_000.0, 12_000.0, 0.05, false),
            )
            .set_length(2.97)
            // .set_pitch(-55_f32.to_radians())
            .set_bounds(-55_f32.to_radians(), 125_f32.to_radians())
            .set_tolerance(0.05),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).set_length(1.5))
        .build();

    Excavator::new(robot)
}

#[allow(dead_code)]
pub(crate) fn kaas() {
    // The set that will contain our rigid-bodies.
    let mut rigid_body_set = RigidBodySet::new();

    let frame = RigidBodyBuilder::kinematic_position_based()
        .translation(vector![0.0, 0.0, 1.295])
        .build();

    let _frame_handle = rigid_body_set.insert(frame);

    let boom = RigidBodyBuilder::kinematic_position_based()
        .position(Isometry::from_parts(
            Translation3::new(0.16, 0.0, 0.595),
            UnitQuaternion::from_euler_angles(0.0, -59.35_f32.to_radians(), 0.0),
        ))
        .build();

    let _boom_handle = rigid_body_set.insert(boom);

    let arm = RigidBodyBuilder::kinematic_position_based()
        .translation(nalgebra::Vector3::new(6.0, 0.0, 0.0))
        .build();

    let _arm_handle = rigid_body_set.insert(arm);

    let attachment = RigidBodyBuilder::kinematic_position_based()
        .position(Isometry::from_parts(
            Translation3::new(2.97, 0.0, 0.0),
            UnitQuaternion::from_euler_angles(0.0, -55_f32.to_radians(), 0.0),
        ))
        .build();

    let _attachment_handle = rigid_body_set.insert(attachment);

    // let joint_set = JointSet::new();

    // let joint = RevoluteJointBuilder::new(Vector::y_axis())
    //     .local_anchor1(point![0.0, 0.0, 1.0])
    //     .local_anchor2(point![0.0, 0.0, -3.0]);
    // joint_set.insert(&mut rigid_body_set, body_handle1, body_handle2, joint, true);
}
