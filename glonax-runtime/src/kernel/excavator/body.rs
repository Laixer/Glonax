use crate::algorithm::ik::InverseKinematics;

#[derive(Clone, Copy)]
pub struct RigidBody {
    pub length_boom: f32,
    pub length_arm: f32,
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

    pub fn with_rigid_body(rigid: RigidBody) -> Self {
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
            let x = boom_point.x + (self.rigid.length_arm * angle_arm.cos());
            let y = boom_point.y + (self.rigid.length_arm * angle_arm.sin());

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
}
