use std::collections::HashMap;

use nalgebra::{IsometryMatrix3, Rotation3, Translation3};

pub enum JointType {
    Revolute,
    Prismatic,
    Continuous,
    Fixed,
}

#[allow(dead_code)]
pub struct Joint {
    name: String,
    ty: JointType,
    origin: IsometryMatrix3<f32>,
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

    pub fn origin(&self) -> &IsometryMatrix3<f32> {
        &self.origin
    }
}

pub enum DeviceType {
    EncoderAbsoluteMultiTurn,
    EncoderAbsoluteSingleTurn,
}

#[allow(dead_code)]
pub struct Device {
    name: String,
    id: u8,
    ty: DeviceType,
    options: HashMap<String, String>,
}

impl Device {
    pub fn new(name: impl ToString, id: u8, ty: DeviceType) -> Self {
        Self {
            name: name.to_string(),
            id,
            ty,
            options: HashMap::new(),
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}

#[derive(Debug)]
pub enum RobotType {
    Excavator,
    WheelLoader,
    Dozer,
    Grader,
    Hauler,
    Forestry,
}

#[allow(dead_code)]
pub struct Robot {
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

impl std::fmt::Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Robot: {}; Model: {}; Name: {} Type: {:?}; Joints: {}",
            self.instance,
            self.model,
            self.name,
            self.ty,
            self.joints.len()
        )
    }
}

pub struct RobotBuilder {
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
