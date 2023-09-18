use std::collections::HashMap;

use nalgebra::{Isometry3, Point3, Translation3, UnitQuaternion};

const DEFAULT_TOLERANCE: f32 = 0.01;

pub trait MotionProfile {
    /// Return the power setting for a given error value.
    fn power(&self, value: f32) -> i16;
}

#[derive(Copy, Clone, Debug)]
pub struct LinearMotionProfile {
    pub scale: f32,
    pub offset: f32,
    pub lower_bound: f32,
    pub inverse: bool,
}

impl LinearMotionProfile {
    pub fn new(scale: f32, offset: f32, lower_bound: f32, inverse: bool) -> Self {
        Self {
            scale,
            offset,
            lower_bound,
            inverse,
        }
    }

    pub fn power(&self, value: f32) -> i16 {
        if self.inverse {
            self.proportional_power_inverse(value)
        } else {
            self.proportional_power(value)
        }
    }

    pub fn proportional_power(&self, value: f32) -> i16 {
        if value.abs() > self.lower_bound {
            let power = self.offset + (value.abs() * self.scale).min(32_767.0 - self.offset);
            if value < 0.0 {
                -power as i16
            } else {
                power as i16
            }
        } else {
            0
        }
    }

    pub fn proportional_power_inverse(&self, value: f32) -> i16 {
        if value.abs() > self.lower_bound {
            let power = value * self.scale;

            if value > 0.0 {
                (-power.max(-(32_767.0 - self.offset)) - self.offset) as i16
            } else {
                (-power.min(32_767.0 - self.offset) + self.offset) as i16
            }
        } else {
            0
        }
    }
}

impl MotionProfile for LinearMotionProfile {
    fn power(&self, value: f32) -> i16 {
        self.power(value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone)]
pub struct Joint {
    name: String,
    ty: JointType,
    origin: Isometry3<f32>,
    bounds: (f32, f32),
    tolerance: f32,
    actuator: Option<crate::core::Actuator>,
    profile: Option<LinearMotionProfile>,
}

impl Joint {
    /// Construct a new joint.
    pub fn new(name: impl ToString, ty: JointType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: Isometry3::identity(),
            bounds: (-f32::INFINITY, f32::INFINITY),
            tolerance: DEFAULT_TOLERANCE,
            actuator: None,
            profile: None,
        }
    }

    pub fn with_actuator(
        name: impl ToString,
        ty: JointType,
        actuator: crate::core::Actuator,
        profile: LinearMotionProfile,
    ) -> Self {
        Self {
            name: name.to_string(),
            ty,
            origin: Isometry3::identity(),
            bounds: (-f32::INFINITY, f32::INFINITY),
            tolerance: DEFAULT_TOLERANCE,
            actuator: Some(actuator),
            profile: Some(profile),
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
        self.origin.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, yaw);
        self
    }

    pub fn set_pitch(mut self, pitch: f32) -> Self {
        self.origin.rotation = UnitQuaternion::from_euler_angles(0.0, pitch, 0.0);
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
        self.origin.rotation =
            UnitQuaternion::from_euler_angles(origin_roll, origin_pitch, origin_yaw);
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
    pub fn ty(&self) -> &JointType {
        &self.ty
    }

    #[inline]
    pub fn origin(&self) -> &Isometry3<f32> {
        &self.origin
    }

    #[inline]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    #[inline]
    pub fn bounds(&self) -> (f32, f32) {
        self.bounds
    }

    #[inline]
    pub fn actuator(&self) -> Option<crate::core::Actuator> {
        self.actuator
    }

    #[inline]
    pub fn profile(&self) -> Option<&LinearMotionProfile> {
        self.profile.as_ref()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DeviceType {
    EncoderAbsoluteMultiTurn,
    EncoderAbsoluteSingleTurn,
}

#[allow(dead_code)]
#[derive(Clone)]
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

pub struct JointDiff<'a> {
    pub joint: &'a Joint,
    pub rotation: UnitQuaternion<f32>,
}

pub struct Chain {
    robot: Robot,
    joint_state: Vec<(String, Option<UnitQuaternion<f32>>)>,
    previous_state: Vec<(String, Option<UnitQuaternion<f32>>)>,
    last_update: std::time::Instant,
}

impl Chain {
    pub fn new(robot: Robot) -> Self {
        Self {
            robot,
            joint_state: vec![],
            previous_state: vec![],
            last_update: std::time::Instant::now(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.joint_state.iter().all(|(_, joint)| joint.is_some())
    }

    pub fn last_update(&self) -> std::time::Instant {
        self.last_update
    }

    // TODO: HACK: XXX: REMOVE: This is a temporary hack to get the absolute pitch of the arm
    // #[deprecated]
    // pub fn abs_pitch(&self) -> Option<f32> {
    //     if self.joint_state[1].1.is_some() && self.joint_state[2].1.is_some() {
    //         let theta_2 = self.joint_state[1]
    //             .1
    //             .unwrap()
    //             .axis()
    //             .map(|axis| axis.y)
    //             .unwrap_or_default()
    //             * self.joint_state[1].1.unwrap().angle();

    //         let theta_3 = self.joint_state[2]
    //             .1
    //             .unwrap()
    //             .axis()
    //             .map(|axis| axis.y)
    //             .unwrap_or_default()
    //             * self.joint_state[2].1.unwrap().angle();

    //         // Some((-59.35_f32.to_radians() + theta_2) + theta_3)
    //         Some((theta_2) + theta_3)
    //     } else {
    //         None
    //     }
    // }

    // TODO: HACK: XXX: REMOVE: This is a temporary hack to get the absolute pitch of the arm
    // #[deprecated]
    // pub fn abs_pitch_with_attachment(&self) -> Option<f32> {
    //     if self.abs_pitch().is_some() && self.joint_state[3].1.is_some() {
    //         let theta_4 = self.joint_state[3]
    //             .1
    //             .unwrap()
    //             .axis()
    //             .map(|axis| axis.y)
    //             .unwrap_or_default()
    //             * self.joint_state[3].1.unwrap().angle();

    //         Some(self.abs_pitch().unwrap() + theta_4)
    //     } else {
    //         None
    //     }
    // }

    pub fn reset(&mut self) {
        for (_, joint) in &mut self.joint_state {
            *joint = None;
        }
        for (_, joint) in &mut self.previous_state {
            *joint = None;
        }
    }

    pub fn add_link(&mut self, link: impl ToString) -> &mut Self {
        self.joint_state.push((link.to_string(), None));
        self.previous_state.push((link.to_string(), None));
        self
    }

    // TODO: When we set the position of a joint, limit to the joint axis
    pub fn set_joint_position(&mut self, name: impl ToString, rotation: UnitQuaternion<f32>) {
        let joint_idx = self
            .joint_state
            .iter()
            .position(|(joint_name, _)| joint_name == &name.to_string())
            .unwrap();

        self.previous_state[joint_idx].1 = self.joint_state[joint_idx].1;
        self.joint_state[joint_idx].1 = Some(rotation);

        self.last_update = std::time::Instant::now();
    }

    // TODO: Maybe remove this function
    // TODO: When we set the position of a joint, limit to the joint axis
    #[deprecated]
    pub fn set_joint_positions(&mut self, rotations: Vec<UnitQuaternion<f32>>) {
        for (joint_idx, rotation) in rotations.iter().enumerate() {
            self.previous_state[joint_idx].1 = self.joint_state[joint_idx].1;
            self.joint_state[joint_idx].1 = Some(*rotation);
        }

        self.last_update = std::time::Instant::now();
    }

    pub fn transformation_until(&self, end_joint_name: impl ToString) -> Isometry3<f32> {
        let mut pose = Isometry3::identity();

        for (joint_name, rotation) in &self.joint_state {
            let joint = self.robot.joint_by_name(joint_name).unwrap();

            if rotation.is_some() {
                pose = pose * joint.origin() * rotation.unwrap();
            } else {
                pose = pose * joint.origin();
            }

            if joint_name == &end_joint_name.to_string() {
                break;
            }
        }

        pose
    }

    pub fn transformation(&self) -> Isometry3<f32> {
        let mut pose = Isometry3::identity();

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

    pub fn effector_point(&self) -> Point3<f32> {
        self.transformation().translation.vector.into()
    }

    pub fn error(&self, rhs: &Self) -> Vec<JointDiff> {
        let mut error_vec = vec![];

        for (joint_name, lhs_rotation, rhs_rotation) in self
            .joint_state
            .iter()
            .zip(&rhs.joint_state)
            .filter(|(lhs, rhs)| lhs.0 == rhs.0 && lhs.1.is_some() && rhs.1.is_some())
            .map(|((name, lhs), (_, rhs))| (name, lhs.unwrap(), rhs.unwrap()))
        {
            error_vec.push(JointDiff {
                joint: self.robot.joint_by_name(joint_name).unwrap(),
                rotation: lhs_rotation.rotation_to(&rhs_rotation),
            });
        }

        error_vec
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vector = self.transformation().translation.vector;
        // TODO: This is an incomplete solution to the problem of getting the pitch of the arm
        let (_, pitch, _) = self.transformation().rotation.euler_angles();

        write!(
            f,
            "({:.2}, {:.2}, {:.2}) [{:.2}rad {:.2}°]",
            vector.x,
            vector.y,
            vector.z,
            pitch,
            pitch.to_degrees()
        )
    }
}

impl std::fmt::Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for (joint_name, lhs_rotation, rhs_rotation) in self
            .joint_state
            .iter()
            .filter(|(_, lhs_rotation)| lhs_rotation.is_some())
            .map(|(name, lhs)| {
                let rhs_rotation = self.robot.joint_by_name(&name).unwrap().origin().rotation;

                (name.to_string(), lhs.unwrap(), rhs_rotation)
            })
        {
            // let chain_angle = lhs_rotation
            //     .axis()
            //     .map(|axis| {
            //         axis.x * lhs_rotation.angle()
            //             + axis.y * lhs_rotation.angle()
            //             + axis.z * lhs_rotation.angle()
            //     })
            //     .unwrap_or_default();

            let relative_rotation = lhs_rotation * rhs_rotation;
            let joint_angle = relative_rotation
                .axis()
                .map(|axis| {
                    axis.x * relative_rotation.angle()
                        + axis.y * relative_rotation.angle()
                        + axis.z * relative_rotation.angle()
                })
                .unwrap_or_default();

            // s.push_str(&format!(
            //     "{}={:.2}rad/{:.2}° [{:.2}rad/{:.2}°] ",
            //     joint_name,
            //     chain_angle,
            //     chain_angle.to_degrees(),
            //     joint_angle,
            //     joint_angle.to_degrees(),
            // ));
            s.push_str(&format!(
                "{}={:.2}rad/{:.2}° ",
                joint_name,
                joint_angle,
                joint_angle.to_degrees(),
            ));
        }

        write!(f, "{s} {}", self)
    }
}

impl Clone for Chain {
    fn clone(&self) -> Self {
        let mut this = Self {
            robot: self.robot.clone(),
            joint_state: self.joint_state.clone(),
            previous_state: self.previous_state.clone(),
            last_update: std::time::Instant::now(),
        };
        this.reset();
        this
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RobotType {
    Excavator,
    WheelLoader,
    Dozer,
    Grader,
    Hauler,
    Forestry,
}

#[derive(Clone)]
pub struct Robot {
    instance: String, // TODO: Replace with UUID
    name: String,
    model: String,
    ty: RobotType,
    joints: Vec<Joint>,
    devices: Vec<Device>,
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
    devices: Vec<Device>,
}

impl RobotBuilder {
    pub fn new(instance: impl ToString, ty: RobotType) -> Self {
        Self {
            instance: instance.to_string(),
            name: String::new(),
            model: String::new(),
            ty,
            joints: Vec::new(),
            devices: Vec::new(),
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
            devices: self.devices,
        }
    }
}
