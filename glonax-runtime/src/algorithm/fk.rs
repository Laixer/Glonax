pub struct ForwardKinematics {
    l1: f32,
    l2: f32,
}

impl ForwardKinematics {
    pub fn new(l1: f32, l2: f32) -> Self {
        Self { l1, l2 }
    }

    pub fn solve(&self, angles: (f32, f32, f32)) -> nalgebra::Point3<f32> {
        let (_theta_0, theta_1, theta_2) = angles;

        let fk_x = (self.l1 * theta_1.cos()) + (self.l2 * theta_2.cos());
        let fk_y = (self.l1 * theta_1.sin()) + (self.l2 * theta_2.sin());

        nalgebra::Point3::new(fk_x, fk_y, 0.0)
    }
}
