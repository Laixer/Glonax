use crate::algorithm::ik::InverseKinematics;

#[derive(Clone, Copy)]
pub struct RigidBody {
    pub length_boom: f32,
    pub length_arm: f32,
}

#[derive(Clone, Copy)]
pub struct Rig {
    angle_slew: Option<f32>,
    angle_boom: Option<f32>,
    angle_arm: Option<f32>,
    angle_attachment: Option<f32>,
}

impl Rig {
    pub fn angle_boom(&self) -> Option<f32> {
        self.angle_boom
    }

    pub fn angle_arm(&self) -> Option<f32> {
        self.angle_arm
    }
}

pub struct Body {
    rigid: RigidBody,
    chain: Pose,
}

impl Body {
    pub const fn new(rigid: RigidBody) -> Self {
        Self {
            rigid,
            chain: Pose {
                rigid,
                rig: Rig {
                    angle_slew: None,
                    angle_boom: None,
                    angle_arm: None,
                    angle_attachment: None,
                },
            },
        }
    }

    pub fn update_slew_angle(&mut self, angle: f32) {
        self.chain.rig.angle_slew = Some(angle)
    }

    pub fn update_boom_angle(&mut self, angle: f32) {
        self.chain.rig.angle_boom = Some(angle)
    }

    pub fn update_arm_angle(&mut self, angle: f32) {
        self.chain.rig.angle_arm = Some(angle)
    }

    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        self.chain.effector_point()
    }

    pub fn effector_point_abs(&self) -> Option<nalgebra::Point3<f32>> {
        if let Some(effector_point) = self.chain.effector_point() {
            Some(nalgebra::Point3::new(
                effector_point.x,
                effector_point.y + super::consts::FRAME_HEIGHT,
                effector_point.z,
            ))
        } else {
            None
        }
    }
}

pub struct Objective {
    body: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    chain: Pose,
}

impl Objective {
    pub fn new(
        model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
        target: nalgebra::Point3<f32>,
    ) -> Self {
        let target = Pose::from_effector_point(model.try_read().unwrap().rigid, target);
        Self {
            body: model,
            chain: target,
        }
    }

    pub fn erorr_diff(&self) -> Rig {
        let physical_pose = &self.body.try_read().unwrap().chain;

        let rig_error = self.chain.erorr_diff(physical_pose);

        if let Some(angle_boom_error) = rig_error.angle_boom {
            debug!(
                "Normal Boom:\t {:>+5.2}rad {:>+5.2}°  Target Boom:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                physical_pose.rig.angle_boom.unwrap(),
                crate::core::rad_to_deg(physical_pose.rig.angle_boom.unwrap()),
                self.chain.rig.angle_boom.unwrap(),
                crate::core::rad_to_deg(self.chain.rig.angle_boom.unwrap()),
                angle_boom_error,
                crate::core::rad_to_deg(angle_boom_error)
            );
        }

        if let Some(angle_arm_error) = rig_error.angle_arm {
            debug!(
                "Normal Arm:\t\t {:>+5.2}rad {:>+5.2}°  Target Arm:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                physical_pose.rig.angle_arm.unwrap(),
                crate::core::rad_to_deg(physical_pose.rig.angle_arm.unwrap()),
                self.chain.rig.angle_arm.unwrap(),
                crate::core::rad_to_deg(self.chain.rig.angle_arm.unwrap()),
                angle_arm_error,
                crate::core::rad_to_deg(angle_arm_error)
            );
        }

        rig_error
    }
}

struct Pose {
    rigid: RigidBody,
    rig: Rig,
}

impl Pose {
    fn from_effector_point(rigid: RigidBody, point: nalgebra::Point3<f32>) -> Self {
        let ik = InverseKinematics::new(rigid.length_boom, rigid.length_arm);

        let (angle_slew, angle_boom, angle_arm) = ik.solve(point).unwrap();

        Self {
            rigid,
            rig: Rig {
                angle_slew: Some(angle_slew),
                angle_boom: Some(angle_boom),
                angle_arm: Some(angle_arm),
                angle_attachment: None,
            },
        }
    }

    pub fn boom_point(&self) -> Option<nalgebra::Point2<f32>> {
        if let Some(angle_boom) = self.rig.angle_boom {
            let x = self.rigid.length_boom * angle_boom.cos();
            let y = self.rigid.length_boom * angle_boom.sin();

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point_flat(&self) -> Option<nalgebra::Point2<f32>> {
        if let (Some(boom_point), Some(angle_arm)) = (self.boom_point(), self.rig.angle_arm) {
            let x = boom_point.x
                + (self.rigid.length_arm * (angle_arm + self.rig.angle_boom.unwrap()).cos());
            let y = boom_point.y
                + (self.rigid.length_arm * (angle_arm + self.rig.angle_boom.unwrap()).sin());

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        if let (Some(effector_point), Some(angle_slew)) =
            (self.effector_point_flat(), self.rig.angle_slew)
        {
            let x = effector_point.x * angle_slew.cos();
            let y = effector_point.y;
            let z = effector_point.x * angle_slew.sin();

            Some(nalgebra::Point3::new(x, y, z))
        } else {
            None
        }
    }

    pub fn erorr_diff(&self, rhs: &Self) -> Rig {
        let mut rig_error = Rig {
            angle_slew: None,
            angle_boom: None,
            angle_arm: None,
            angle_attachment: None,
        };

        if let (Some(lhs_angle_slew), Some(rhs_angle_slew)) =
            (self.rig.angle_slew, rhs.rig.angle_slew)
        {
            rig_error.angle_slew = Some(lhs_angle_slew - rhs_angle_slew);
        }

        if let (Some(lhs_angle_boom), Some(rhs_angle_boom)) =
            (self.rig.angle_boom, rhs.rig.angle_boom)
        {
            rig_error.angle_boom = Some(lhs_angle_boom - rhs_angle_boom);
        }

        if let (Some(lhs_angle_arm), Some(rhs_angle_arm)) = (self.rig.angle_arm, rhs.rig.angle_arm)
        {
            rig_error.angle_arm = Some(lhs_angle_arm - rhs_angle_arm);
        }

        rig_error
    }
}
