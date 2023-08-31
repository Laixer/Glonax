use std::collections::HashMap;

use nalgebra::{IsometryMatrix3, Point3, Rotation3, Translation3};

const DEFAULT_TOLERANCE: f32 = 0.01;

#[derive(Clone)]
pub enum JointType {
    /// A joint that provides one degree of freedom about a fixed axis of rotation.
    Revolute,
    /// A joint that provides one degree of freedom about a fixed axis of translation.
    Prismatic,
    /// A joint that provides one degree of freedom about a fixed axis of rotation with a continuous range of motion.
    Continuous,
    /// A joint that provides zero degrees of freedom.
    Fixed,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Joint {
    name: String,
    ty: JointType,
    origin: IsometryMatrix3<f32>,
    bounds: (f32, f32),
    tolerance: f32,
}

impl Joint {
    /// Construct a new joint.
    pub fn new(name: impl ToString, ty: JointType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: IsometryMatrix3::identity(),
            bounds: (-f32::INFINITY, f32::INFINITY),
            tolerance: DEFAULT_TOLERANCE,
        }
    }

    pub fn set_height(mut self, height: f32) -> Self {
        self.origin.translation =
            Translation3::new(self.origin.translation.x, self.origin.translation.y, height);
        self
    }

    pub fn set_length(mut self, length: f32) -> Self {
        self.origin.translation =
            Translation3::new(length, self.origin.translation.y, self.origin.translation.z);
        self
    }

    pub fn set_yaw(mut self, yaw: f32) -> Self {
        self.origin.rotation = Rotation3::from_euler_angles(0.0, 0.0, yaw);
        self
    }

    pub fn set_pitch(mut self, pitch: f32) -> Self {
        self.origin.rotation = Rotation3::from_euler_angles(0.0, pitch, 0.0);
        self
    }

    pub fn set_origin_translation(mut self, origin_x: f32, origin_y: f32, origin_z: f32) -> Self {
        self.origin.translation = Translation3::new(origin_x, origin_y, origin_z);
        self
    }

    pub fn set_origin_rotation(
        mut self,
        origin_roll: f32,
        origin_pitch: f32,
        origin_yaw: f32,
    ) -> Self {
        self.origin.rotation = Rotation3::from_euler_angles(origin_roll, origin_pitch, origin_yaw);
        self
    }

    pub fn set_bounds(mut self, lower: f32, upper: f32) -> Self {
        self.bounds = (lower, upper);
        self
    }

    pub fn set_tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn origin(&self) -> &IsometryMatrix3<f32> {
        &self.origin
    }

    #[inline]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    // pub fn rotation_angle(&self) -> Option<f32> {
    //     if let Some(axis) = self.rotation.axis() {
    //         Some(
    //             (axis.x * self.rotation.angle())
    //                 + (axis.y * self.rotation.angle())
    //                 + (axis.z * self.rotation.angle()),
    //         )
    //     } else {
    //         None
    //     }
    // }
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

pub struct Chain<'a> {
    robot: &'a Robot,
    joint_state: Vec<(String, Option<Rotation3<f32>>)>,
}

impl<'a> Chain<'a> {
    pub fn new(robot: &'a Robot) -> Self {
        Self {
            robot,
            joint_state: vec![],
        }
    }

    pub fn is_ready(&self) -> bool {
        self.joint_state.iter().all(|(_, joint)| joint.is_some())
    }

    pub fn reset(&mut self) {
        for (_, joint) in &mut self.joint_state {
            *joint = None;
        }
    }

    pub fn add_link(&mut self, link: impl ToString) -> &mut Self {
        self.joint_state.push((link.to_string(), None));
        self
    }

    pub fn add_joint(&mut self, joint: Joint) -> &mut Self {
        self.joint_state.push((joint.name.clone(), None));
        self
    }

    pub fn set_joint_position(&mut self, name: impl ToString, rotation: Rotation3<f32>) {
        self.joint_state
            .iter_mut()
            .find(|(joint_name, _)| joint_name == &name.to_string())
            .unwrap()
            .1 = Some(rotation);
    }

    pub fn set_joint_positions(&mut self, rotations: Vec<Rotation3<f32>>) {
        for ((_, state), rotation) in self.joint_state.iter_mut().zip(rotations) {
            *state = Some(rotation);
        }
    }

    pub fn world_transformation(&self) -> IsometryMatrix3<f32> {
        let mut pose = IsometryMatrix3::identity();

        for (joint_name, rotation) in &self.joint_state {
            let joint = self.robot.joint_by_name(joint_name).unwrap();

            if rotation.is_some() {
                pose = pose * joint.origin() * rotation.unwrap();
            } else {
                pose = pose * joint.origin();
            }
        }

        pose
    }

    pub fn distance(&self, rhs: &Self) -> f32 {
        let lhs_point = self.world_transformation() * Point3::origin();
        let rhs_point = rhs.world_transformation() * Point3::origin();

        nalgebra::distance(&lhs_point, &rhs_point)
    }

    pub fn error(&self, rhs: &Self) -> Vec<(&String, Rotation3<f32>)> {
        let mut error_vec = vec![];

        for (joint_name, lhs_rotation, rhs_rotation) in self
            .joint_state
            .iter()
            .zip(&rhs.joint_state)
            .filter(|(lhs, rhs)| lhs.0 == rhs.0 && lhs.1.is_some() && rhs.1.is_some())
            .map(|((name, lhs), (_, rhs))| (name, lhs.unwrap(), rhs.unwrap()))
        {
            error_vec.push((joint_name, lhs_rotation.rotation_to(&rhs_rotation)));
        }

        error_vec
    }
}

impl std::fmt::Display for Chain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let point = self.world_transformation() * Point3::origin();

        write!(f, "[{:.2}, {:.2}, {:.2}]", point.x, point.y, point.z)
    }
}

impl std::fmt::Debug for Chain<'_> {
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
