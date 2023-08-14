// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use na::{Isometry, IsometryMatrix3, Rotation3, Translation3};
use nalgebra as na;

enum JointType {
    Revolute,
    Prismatic,
    Continuous,
    Fixed,
}

struct Joint {
    name: String,
    ty: JointType,
    origin: na::IsometryMatrix3<f32>,
}

impl Joint {
    pub fn new(name: impl ToString, ty: JointType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: Isometry::identity(),
        }
    }

    pub fn origin(mut self, origin_x: f32, origin_y: f32, origin_z: f32) -> Self {
        self.origin = na::IsometryMatrix3::from_parts(
            Translation3::new(origin_x, origin_y, origin_z),
            Rotation3::identity(),
        );
        self
    }
}

struct Robot {
    instance: String, // TODO: Replace with UUID
    name: String,
    model: String,
    joints: Vec<Joint>,
    chains: Vec<String>,
    devices: Vec<String>,
    position_state: Vec<f64>,
}

impl Robot {
    pub fn joint_by_name(&self, name: impl ToString) -> Option<&Joint> {
        self.joints
            .iter()
            .find(|joint| joint.name == name.to_string())
    }
}

struct RobotBuilder {
    instance: String, // TODO: Replace with UUID
    name: String,
    model: String,
    joints: Vec<Joint>,
    chains: Vec<String>,
    devices: Vec<String>,
    position_state: Vec<f64>,
}

impl RobotBuilder {
    pub fn new(instance: impl ToString) -> Self {
        Self {
            instance: instance.to_string(),
            name: String::new(),
            model: String::new(),
            joints: Vec::new(),
            chains: Vec::new(),
            devices: Vec::new(),
            position_state: Vec::new(),
        }
    }

    pub fn name(mut self, instance: impl ToString) -> Self {
        self.name = instance.to_string();
        self
    }

    pub fn model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn add_joint(mut self, joint: Joint) -> Self {
        self.joints.push(joint);
        self
    }

    pub fn add_chain(mut self, chain: String) -> Self {
        self.chains.push(chain);
        self
    }

    pub fn add_device(mut self, device: String) -> Self {
        self.devices.push(device);
        self
    }

    pub fn build(self) -> Robot {
        Robot {
            instance: self.instance,
            name: self.name,
            model: self.model,
            joints: self.joints,
            chains: Vec::new(),
            devices: Vec::new(),
            position_state: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client =
        glonax::transport::Client::connect(&"localhost".to_string(), "glonax-dump".to_string())
            .await?;

    let robot = RobotBuilder::new("36ebd674-21fa-4497-a712-eb2ee6b2cda1")
        .name("gustav")
        .add_joint(Joint::new("undercarriage", JointType::Fixed))
        .add_joint(Joint::new("frame", JointType::Continuous).origin(0.0, 0.0, 1.295))
        .add_joint(Joint::new("boom", JointType::Revolute).origin(0.16, 0.0, 0.595))
        .add_joint(Joint::new("arm", JointType::Revolute).origin(6.0, 0.0, 0.0))
        .add_joint(Joint::new("attachment", JointType::Revolute).origin(2.97, 0.0, 0.0))
        .add_joint(Joint::new("end", JointType::Fixed))
        .build();

    let point = na::Point3::new(0.0, 0.0, 0.0);

    let mut frame_yaw = 0.0; //std::f32::consts::FRAC_PI_2; //-0.523599; //-std::f64::consts::FRAC_PI_3;
    let mut boom_pitch = 0.0; //-0.523599; //-std::f64::consts::FRAC_PI_3;
    let mut arm_pitch = 0.0; //-0.523599; //-std::f64::consts::FRAC_PI_3;
    let mut attachment_pitch = 0.0; //-0.523599; //-std::f64::consts::FRAC_PI_3;

    let frame_joint = robot.joint_by_name("frame").unwrap();
    let boom_joint = robot.joint_by_name("boom").unwrap();
    let arm_joint = robot.joint_by_name("arm").unwrap();
    let attachment_joint = robot.joint_by_name("attachment").unwrap();

    // let effector_point = (frame_joint.origin * Rotation3::from_euler_angles(0.0, 0.0, frame_yaw))
    //     * (boom_joint.origin * Rotation3::from_euler_angles(0.0, boom_pitch, 0.0))
    //     * (arm_joint.origin * Rotation3::from_euler_angles(0.0, arm_pitch, 0.0))
    //     * (attachment_joint.origin * Rotation3::from_euler_angles(0.0, attachment_pitch, 0.0))
    //     * point;

    // println!(
    //     "effector_point: [{:.2}, {:.2}, {:.2}]",
    //     effector_point.x, effector_point.y, effector_point.z
    // );

    while let Ok(signal) = client.recv_signal().await {
        match signal.metric {
            glonax::core::Metric::EncoderAbsAngle((node, value)) => match node {
                0x6A => {
                    // println!(
                    //     "Frame Abs Angle: {:.2}rad {:.2}°",
                    //     value,
                    //     glonax::core::rad_to_deg(value)
                    // );
                    frame_yaw = value;

                    let effector_point = (frame_joint.origin
                        * Rotation3::from_euler_angles(0.0, 0.0, frame_yaw))
                        * (boom_joint.origin * Rotation3::from_euler_angles(0.0, boom_pitch, 0.0))
                        * (arm_joint.origin * Rotation3::from_euler_angles(0.0, arm_pitch, 0.0))
                        * (attachment_joint.origin
                            * Rotation3::from_euler_angles(0.0, attachment_pitch, 0.0))
                        * point;

                    // println!(
                    //     "effector_point: [{:.2}, {:.2}, {:.2}]",
                    //     effector_point.x, effector_point.y, effector_point.z
                    // );
                }
                0x6B => {
                    //     println!(
                    //     "Boom Abs Angle: {:.2}rad {:.2}°",
                    //     value,
                    //     glonax::core::rad_to_deg(value)
                    // );
                    boom_pitch = value;

                    let effector_point = (frame_joint.origin
                        * Rotation3::from_euler_angles(0.0, 0.0, frame_yaw))
                        * (boom_joint.origin * Rotation3::from_euler_angles(0.0, boom_pitch, 0.0))
                        * (arm_joint.origin * Rotation3::from_euler_angles(0.0, arm_pitch, 0.0))
                        * (attachment_joint.origin
                            * Rotation3::from_euler_angles(0.0, attachment_pitch, 0.0))
                        * point;

                    // println!(
                    //     "effector_point: [{:.2}, {:.2}, {:.2}]",
                    //     effector_point.x, effector_point.y, effector_point.z
                    // );
                }
                0x6C => {
                    // println!(
                    //     "Arm Abs Angle: {:.2}rad {:.2}°",
                    //     value,
                    //     glonax::core::rad_to_deg(value)
                    // );

                    arm_pitch = value;

                    let effector_point = (frame_joint.origin
                        * Rotation3::from_euler_angles(0.0, 0.0, frame_yaw))
                        * (boom_joint.origin * Rotation3::from_euler_angles(0.0, boom_pitch, 0.0))
                        * (arm_joint.origin * Rotation3::from_euler_angles(0.0, arm_pitch, 0.0))
                        * (attachment_joint.origin
                            * Rotation3::from_euler_angles(0.0, attachment_pitch, 0.0))
                        * point;

                    // println!(
                    //     "effector_point: [{:.2}, {:.2}, {:.2}]",
                    //     effector_point.x, effector_point.y, effector_point.z
                    // );
                }
                0x6D => {
                    // println!(
                    //     "Attachment Abs Angle: {:.2}rad {:.2}°",
                    //     value,
                    //     glonax::core::rad_to_deg(value)
                    // );

                    attachment_pitch = value;

                    let effector_point = (frame_joint.origin
                        * Rotation3::from_euler_angles(0.0, 0.0, frame_yaw))
                        * (boom_joint.origin * Rotation3::from_euler_angles(0.0, boom_pitch, 0.0))
                        * (arm_joint.origin * Rotation3::from_euler_angles(0.0, arm_pitch, 0.0))
                        * (attachment_joint.origin
                            * Rotation3::from_euler_angles(0.0, attachment_pitch, 0.0))
                        * point;

                    // println!(
                    //     "effector_point: [{:.2}, {:.2}, {:.2}]",
                    //     effector_point.x, effector_point.y, effector_point.z
                    // );
                }
                _ => {}
            },
            _ => {}
        }

        println!(
            "F Angle: {:5.2}rad {:5.2}°\tB Angle: {:5.2}rad {:5.2}°\tA Angle: {:5.2}rad {:5.2}°\tT Angle: {:5.2}rad {:5.2}°",
            frame_yaw,
            glonax::core::rad_to_deg(frame_yaw),
            boom_pitch,
            glonax::core::rad_to_deg(boom_pitch),
            arm_pitch,
            glonax::core::rad_to_deg(arm_pitch),
            attachment_pitch,
            glonax::core::rad_to_deg(attachment_pitch),
        );
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    client.shutdown().await?;

    Ok(())
}
