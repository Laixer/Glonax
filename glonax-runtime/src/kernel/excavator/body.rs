use crate::algorithm::ik::InverseKinematics;

#[derive(Clone, Copy)]
pub struct RigidBody {
    pub length_boom: f32,
    pub length_arm: f32,
}

pub struct Body {
    rigid: RigidBody,
    physical: DynamicBody,
}

impl Body {
    pub const fn new(rigid: RigidBody) -> Self {
        Self {
            rigid,
            physical: DynamicBody::with_rigid_body(rigid),
        }
    }

    pub fn project(&self, target: nalgebra::Point3<f32>) -> Projection {
        let target = super::body::DynamicBody::from_effector_point(self.rigid, target);
        Projection { body: self, target }
    }

    pub fn update_slew_angle(&mut self, angle: f32) {
        self.physical.angle_slew = Some(angle)
    }

    pub fn update_boom_angle(&mut self, angle: f32) {
        self.physical.angle_boom = Some(angle)
    }

    pub fn update_arm_angle(&mut self, angle: f32) {
        self.physical.angle_arm = Some(angle)
    }

    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        self.physical.effector_point()
    }

    pub fn effector_point_abs(&self) -> Option<nalgebra::Point3<f32>> {
        if let Some(effector_point) = self.physical.effector_point() {
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

pub struct Projection<'a> {
    body: &'a Body,
    target: DynamicBody,
}

impl<'a> Projection<'a> {
    pub fn erorr_diff(&self) -> Option<(f32, f32)> {
        let xx = self.target.erorr_diff(&self.body.physical);

        if let Some((angle_boom_error, angle_arm_error)) = xx {
            debug!(
                "Normal Boom:\t {:>+5.2}rad {:>+5.2}°  Target Boom:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                self.body.physical.angle_boom.unwrap(),
                crate::core::rad_to_deg(self.body.physical.angle_boom.unwrap()),
                self.target.angle_boom.unwrap(),
                crate::core::rad_to_deg(self.target.angle_boom.unwrap()),
                angle_boom_error,
                crate::core::rad_to_deg(angle_boom_error)
            );
            debug!(
                "Normal Arm:\t\t {:>+5.2}rad {:>+5.2}°  Target Arm:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                self.body.physical.angle_arm.unwrap(),
                crate::core::rad_to_deg(self.body.physical.angle_arm.unwrap()),
                self.target.angle_arm.unwrap(),
                crate::core::rad_to_deg(self.target.angle_arm.unwrap()),
                angle_arm_error,
                crate::core::rad_to_deg(angle_arm_error)
            );

            Some((angle_boom_error, angle_arm_error))
        } else {
            None
        }
    }
}

pub struct DynamicBody {
    rigid: RigidBody,
    pub angle_slew: Option<f32>,
    pub angle_boom: Option<f32>,
    pub angle_arm: Option<f32>,
    pub angle_attachment: Option<f32>,
}

impl DynamicBody {
    pub fn from_effector_point(rigid: RigidBody, point: nalgebra::Point3<f32>) -> Self {
        let ik = InverseKinematics::new(rigid.length_boom, rigid.length_arm);

        let (angle_slew, angle_boom, angle_arm) = ik.solve(point).unwrap();

        Self {
            rigid,
            angle_slew: Some(angle_slew),
            angle_boom: Some(angle_boom),
            angle_arm: Some(angle_arm),
            angle_attachment: None,
        }
    }

    pub const fn with_rigid_body(rigid: RigidBody) -> Self {
        Self {
            rigid,
            angle_slew: None,
            angle_boom: None,
            angle_arm: None,
            angle_attachment: None,
        }
    }

    pub fn update_slew_angle(&mut self, angle: f32) {
        self.angle_slew = Some(angle)
    }

    pub fn update_boom_angle(&mut self, angle: f32) {
        self.angle_boom = Some(angle)
    }

    pub fn update_arm_angle(&mut self, angle: f32) {
        self.angle_arm = Some(angle)
    }

    // pub fn update_attachment_angle(&mut self, angle: f32) {
    //     self.angle_attachment = Some(angle)
    // }

    pub fn boom_point(&self) -> Option<nalgebra::Point2<f32>> {
        if let Some(angle_boom) = self.angle_boom {
            let x = self.rigid.length_boom * angle_boom.cos();
            let y = self.rigid.length_boom * angle_boom.sin();

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point_flat(&self) -> Option<nalgebra::Point2<f32>> {
        if let (Some(boom_point), Some(angle_arm)) = (self.boom_point(), self.angle_arm) {
            let x = boom_point.x
                + (self.rigid.length_arm * (angle_arm + self.angle_boom.unwrap()).cos());
            let y = boom_point.y
                + (self.rigid.length_arm * (angle_arm + self.angle_boom.unwrap()).sin());

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        if let (Some(effector_point), Some(angle_slew)) =
            (self.effector_point_flat(), self.angle_slew)
        {
            let x = effector_point.x * angle_slew.cos();
            let y = effector_point.y;
            let z = effector_point.x * angle_slew.sin();

            Some(nalgebra::Point3::new(x, y, z))
        } else {
            None
        }
    }

    pub fn erorr_diff(&self, rhs: &Self) -> Option<(f32, f32)> {
        if let (
            Some(lhs_angle_boom),
            Some(lhs_angle_arm),
            Some(rhs_angle_boom),
            Some(rhs_angle_arm),
        ) = (
            self.angle_boom,
            self.angle_arm,
            rhs.angle_boom,
            rhs.angle_arm,
        ) {
            Some((
                lhs_angle_boom - rhs_angle_boom,
                lhs_angle_arm - rhs_angle_arm,
            ))
        } else {
            None
        }
    }
}
