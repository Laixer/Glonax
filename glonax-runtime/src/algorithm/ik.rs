pub struct InverseKinematics {
    l1: f32,
    l2: f32,
}

impl InverseKinematics {
    pub fn new(l1: f32, l2: f32) -> Self {
        Self { l1, l2 }
    }

    // TODO: return result.
    pub fn solve(&self, target: nalgebra::Point3<f32>) -> Option<(f32, f32, f32)> {
        let l4 = (target.x.powi(2) + target.z.powi(2)).sqrt();
        let l5 = (l4.powi(2) + target.y.powi(2)).sqrt();

        let theta_0 = target.z.atan2(target.x);

        let theta_1 = target.y.atan2(l4)
            + ((self.l1.powi(2) + l5.powi(2) - self.l2.powi(2)) / (2.0 * self.l1 * l5)).acos();

        let theta_2 =
            ((self.l1.powi(2) + self.l2.powi(2) - l5.powi(2)) / (2.0 * self.l1 * self.l2)).acos();

        let theta_2 = std::f32::consts::PI - theta_1 - theta_2;

        if l5 >= self.l1 + self.l2 {
            None
        } else {
            Some((theta_0, theta_1, -theta_2))
        }
    }
}
