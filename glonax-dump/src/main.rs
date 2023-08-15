// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use std::collections::HashMap;

use na::{IsometryMatrix3, Rotation3, Translation3};
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
    bounds: (f32, f32),
}

impl Joint {
    pub fn new(name: impl ToString, ty: JointType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: IsometryMatrix3::identity(),
            bounds: (-f32::INFINITY, f32::INFINITY),
        }
    }

    pub fn origin_translation(mut self, origin_x: f32, origin_y: f32, origin_z: f32) -> Self {
        self.origin.translation = Translation3::new(origin_x, origin_y, origin_z);
        self
    }

    pub fn origin_rotation(mut self, origin_roll: f32, origin_pitch: f32, origin_yaw: f32) -> Self {
        self.origin.rotation = Rotation3::from_euler_angles(origin_roll, origin_pitch, origin_yaw);
        self
    }
}

enum DeviceType {
    EncoderAbsoluteMultiTurn,
    EncoderAbsoluteSingleTurn,
}

struct Device {
    name: String,
    id: u8,
    ty: DeviceType,
    options: HashMap<String, String>,
}

impl Device {
    fn new(name: impl ToString, id: u8, ty: DeviceType) -> Self {
        Self {
            name: name.to_string(),
            id,
            ty,
            options: HashMap::new(),
        }
    }
}

enum RobotType {
    Excavator,
    WheelLoader,
    Dozer,
    Grader,
    Hauler,
    Forestry,
}

struct Robot {
    instance: String, // TODO: Replace with UUID
    name: String,
    model: String,
    ty: RobotType,
    joints: Vec<Joint>,
    chains: Vec<String>,
    devices: Vec<Device>,
    position_state: Vec<f64>,
}

impl Robot {
    pub fn joint_by_name(&self, name: impl ToString) -> Option<&Joint> {
        self.joints
            .iter()
            .find(|joint| joint.name == name.to_string())
    }

    pub fn device_by_name(&self, name: impl ToString) -> Option<&Device> {
        self.devices
            .iter()
            .find(|device| device.name == name.to_string())
    }
}

struct RobotBuilder {
    instance: String, // TODO: Replace with UUID
    name: String,
    model: String,
    ty: RobotType,
    joints: Vec<Joint>,
    chains: Vec<String>,
    devices: Vec<Device>,
    position_state: Vec<f64>,
}

impl RobotBuilder {
    pub fn new(instance: impl ToString, ty: RobotType) -> Self {
        Self {
            instance: instance.to_string(),
            name: String::new(),
            model: String::new(),
            ty,
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

    pub fn model(mut self, model: impl ToString) -> Self {
        self.model = model.to_string();
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

    pub fn add_device(mut self, device: Device) -> Self {
        self.devices.push(device);
        self
    }

    pub fn build(self) -> Robot {
        Robot {
            instance: self.instance,
            name: self.name,
            model: self.model,
            ty: self.ty,
            joints: self.joints,
            chains: self.chains,
            devices: self.devices,
            position_state: self.position_state,
        }
    }
}

trait EulerAngles {
    fn from_roll(roll: f32) -> Self;
    fn from_pitch(pitch: f32) -> Self;
    fn from_yaw(pitch: f32) -> Self;
}

impl EulerAngles for Rotation3<f32> {
    fn from_roll(roll: f32) -> Self {
        Rotation3::from_euler_angles(roll, 0.0, 0.0)
    }

    fn from_pitch(pitch: f32) -> Self {
        Rotation3::from_euler_angles(0.0, pitch, 0.0)
    }

    fn from_yaw(yaw: f32) -> Self {
        Rotation3::from_euler_angles(0.0, 0.0, yaw)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let instance = glonax::instance_config("/etc/glonax.conf")?;

    let mut client = glonax::transport::Client::connect("localhost:30051", "glonax-dump").await?;

    let robot = RobotBuilder::new(instance.instance, RobotType::Excavator)
        .model(instance.model)
        .name(instance.name.unwrap_or("Unnamed".to_string()))
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
        .add_joint(Joint::new("frame", JointType::Continuous).origin_translation(0.0, 0.0, 1.295))
        .add_joint(
            Joint::new("boom", JointType::Revolute)
                .origin_translation(0.16, 0.0, 0.595)
                .origin_rotation(0.0, -1.0472, 0.0),
        )
        .add_joint(Joint::new("arm", JointType::Revolute).origin_translation(6.0, 0.0, 0.0))
        .add_joint(
            Joint::new("attachment", JointType::Revolute)
                .origin_translation(2.97, 0.0, 0.0)
                .origin_rotation(0.0, -0.962, 0.0),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).origin_translation(1.5, 0.0, 0.0))
        .build();

    let point = na::Point3::new(0.0, 0.0, 0.0);

    let mut frame_yaw = 0.0;
    let mut boom_pitch = 0.0;
    let mut arm_pitch = 0.0;
    let mut attachment_pitch = 0.0;

    let frame_encoder = robot.device_by_name("frame_encoder").unwrap();
    let boom_encoder = robot.device_by_name("boom_encoder").unwrap();
    let arm_encoder = robot.device_by_name("arm_encoder").unwrap();
    let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap();

    let frame_joint = robot.joint_by_name("frame").unwrap();
    let boom_joint = robot.joint_by_name("boom").unwrap();
    let arm_joint = robot.joint_by_name("arm").unwrap();
    let attachment_joint = robot.joint_by_name("attachment").unwrap();
    let effector_joint = robot.joint_by_name("effector").unwrap();

    while let Ok(signal) = client.recv_signal().await {
        match signal.metric {
            glonax::core::Metric::EncoderAbsAngle((node, value)) => match node {
                node if frame_encoder.id == node => frame_yaw = value,
                node if boom_encoder.id == node => boom_pitch = value,
                node if arm_encoder.id == node => arm_pitch = value,
                node if attachment_encoder.id == node => attachment_pitch = value,
                _ => {}
            },
            _ => {}
        }

        let link_point = (frame_joint.origin * Rotation3::from_yaw(frame_yaw))
            * (boom_joint.origin * Rotation3::from_pitch(boom_pitch))
            * (arm_joint.origin * Rotation3::from_pitch(arm_pitch))
            * (attachment_joint.origin * Rotation3::from_pitch(attachment_pitch))
            * point;

        let effector_point = (frame_joint.origin * Rotation3::from_yaw(frame_yaw))
            * (boom_joint.origin * Rotation3::from_pitch(boom_pitch))
            * (arm_joint.origin * Rotation3::from_pitch(arm_pitch))
            * (attachment_joint.origin * Rotation3::from_pitch(attachment_pitch))
            * effector_joint.origin
            * point;

        println!(
            "F Angle: {:5.2}rad {:5.2}°\tB Angle: {:5.2}rad {:5.2}°\tA Angle: {:5.2}rad {:5.2}°\tT Angle: {:5.2}rad {:5.2}°\tLink: [{:.2}, {:.2}, {:.2}]\tEffector: [{:.2}, {:.2}, {:.2}]",
            frame_yaw,
            glonax::core::rad_to_deg(frame_yaw),
            boom_pitch,
            glonax::core::rad_to_deg(boom_pitch),
            arm_pitch,
            glonax::core::rad_to_deg(arm_pitch),
            attachment_pitch,
            glonax::core::rad_to_deg(attachment_pitch),
            link_point.x, link_point.y, link_point.z,
            effector_point.x, effector_point.y, effector_point.z
        );
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    client.shutdown().await?;

    Ok(())
}
