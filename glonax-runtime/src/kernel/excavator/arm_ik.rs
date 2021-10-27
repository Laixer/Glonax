use glonax_core::{
    algorithm::ik::InverseKinematics, metric::MetricValue, motion::Motion, nalgebra,
};

use crate::runtime::{operand::*, Signal};

pub struct ArmIkProgram;

impl ArmIkProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for ArmIkProgram {
    fn push(&mut self, domain: Signal) {
        if let MetricValue::Acceleration(vec) = domain.value {
            let _signal_angle = -vec.x.atan2(vec.y);
            // debug!("XY Angle: {:>+5.2}", signal_angle);

            let ik = InverseKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

            let target = nalgebra::Point3::new(5.21, 0.0, 0.0);

            if let Some((theta_0, theta_1, theta_2)) = ik.solve(target) {
                debug!(
                    "Theta 0 {:>+5.2} Theta 1 {:>+5.2} Theta 2 {:>+5.2}",
                    theta_0, theta_1, theta_2
                );
            } else {
                warn!("Target out of range");
            }
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
