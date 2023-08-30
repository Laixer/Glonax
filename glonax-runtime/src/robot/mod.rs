use std::collections::HashMap;

use nalgebra::{IsometryMatrix3, Point3, Rotation3, Translation3};

#[derive(Clone)]
pub enum JointType {
    Revolute,
    Prismatic,
    Continuous,
    Fixed,
}

// #[allow(dead_code)]
#[derive(Clone)]
pub struct Joint {
    name: String,
    ty: JointType,
    origin: IsometryMatrix3<f32>,
    bounds: (f32, f32),
    rotation: Rotation3<f32>,
}

impl Joint {
    pub fn new(name: impl ToString, ty: JointType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: IsometryMatrix3::identity(),
            bounds: (-f32::INFINITY, f32::INFINITY),
            rotation: Rotation3::identity(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn rotation(&self) -> Rotation3<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.rotation = rotation;
    }

    pub fn rotation_angle(&self) -> Option<f32> {
        if let Some(axis) = self.rotation.axis() {
            Some(
                (axis.x * self.rotation.angle())
                    + (axis.y * self.rotation.angle())
                    + (axis.z * self.rotation.angle()),
            )
        } else {
            None
        }
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

pub struct Chain {
    joints: Vec<Joint>,
    joint_state: Vec<(String, Option<Rotation3<f32>>)>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            joints: vec![],
            joint_state: vec![],
        }
    }

    pub fn is_ready(&self) -> bool {
        self.joint_state.iter().all(|(_, joint)| joint.is_some())
    }

    pub fn add_joint(&mut self, joint: Joint) -> &mut Self {
        self.joint_state.push((joint.name.clone(), None));
        self.joints.push(joint);
        self
    }

    pub fn joint_by_name(&mut self, name: impl ToString) -> Option<&mut Joint> {
        self.joints
            .iter_mut()
            .find(|joint| joint.name == name.to_string())
    }

    pub fn joint_rotation_angle(&mut self, name: impl ToString) -> Option<f32> {
        if let Some(joint) = self
            .joints
            .iter()
            .find(|joint| joint.name == name.to_string())
        {
            joint.rotation_angle()
        } else {
            None
        }
    }

    // TODO: Return error instead of Option
    pub fn joint_position(&mut self, name: impl ToString) -> Option<Rotation3<f32>> {
        if let Some(joint) = self
            .joints
            .iter()
            .find(|joint| joint.name == name.to_string())
        {
            Some(joint.rotation)
        } else {
            None
        }
    }

    // TODO: Return error instead of Option
    pub fn set_joint_position(&mut self, name: impl ToString, rotation: Rotation3<f32>) {
        if let Some(joint) = self
            .joints
            .iter_mut()
            .find(|joint| joint.name == name.to_string())
        {
            joint.set_rotation(rotation);
            self.joint_state
                .iter_mut()
                .find(|(joint_name, _)| joint_name == &name.to_string())
                .unwrap()
                .1 = Some(rotation);
        }
    }

    pub fn joint_positions(&mut self) -> Vec<Rotation3<f32>> {
        let mut rotations = vec![];
        for joint in &self.joints {
            rotations.push(joint.rotation);
        }
        rotations
    }

    pub fn set_joint_positions(&mut self, rotations: Vec<Rotation3<f32>>) {
        for (joint, rotation) in self.joints.iter_mut().zip(rotations) {
            joint.set_rotation(rotation);
            self.joint_state
                .iter_mut()
                .find(|(joint_name, _)| joint_name == &joint.name)
                .unwrap()
                .1 = Some(rotation);
        }
    }

    pub fn world_transformation(&self) -> IsometryMatrix3<f32> {
        let mut pose = IsometryMatrix3::identity();

        for joint in &self.joints {
            pose = pose * joint.origin() * joint.rotation;
        }

        pose
    }

    pub fn distance(&self, rhs: &Self) -> f32 {
        let lhs_point = self.world_transformation() * Point3::origin();
        let rhs_point = rhs.world_transformation() * Point3::origin();

        nalgebra::distance(&lhs_point, &rhs_point)
    }

    pub fn error(&self, rhs: &Self) -> Vec<(&Joint, Rotation3<f32>)> {
        let mut error_vec = vec![];

        for (lhs_joint, rhs_joint) in self.joints.iter().zip(&rhs.joints) {
            error_vec.push((
                lhs_joint,
                lhs_joint.rotation.rotation_to(&rhs_joint.rotation),
            ));
        }

        error_vec
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let point = self.world_transformation() * Point3::origin();

        write!(f, "[{:.2}, {:.2}, {:.2}]", point.x, point.y, point.z)
    }
}

impl std::fmt::Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let point = self.world_transformation() * Point3::origin();

        let mut s = String::new();

        for (joint, rotation) in &self.joint_state {
            s.push_str(&format!(
                "{}={:.2}rad/{:5.2}° ",
                joint,
                rotation.unwrap().angle(),
                rotation.unwrap().angle().to_degrees()
            ));
        }

        write!(
            f,
            "{s} Endpoint [{:.2}, {:.2}, {:.2}]",
            point.x, point.y, point.z
        )
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
        write!(
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
