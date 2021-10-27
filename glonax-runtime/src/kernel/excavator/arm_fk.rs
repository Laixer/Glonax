use glonax_core::{algorithm::fk::ForwardKinematics, metric::MetricValue, motion::Motion};

use crate::runtime::{operand::*, Signal};

pub struct ArmFkProgram;

impl ArmFkProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for ArmFkProgram {
    fn push(&mut self, domain: Signal) {
        if let MetricValue::Acceleration(vec) = domain.value {
            let signal_angle = -vec.x.atan2(vec.y);
            debug!("XY Angle: {:>+5.2}", signal_angle);

            let fk = ForwardKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

            let mut effector_point = fk.solve((
                0.0,
                (super::ARM_LENGTH / super::BOOM_LENGTH).asin(),
                signal_angle,
            ));

            effector_point.y += super::FRAME_HEIGHT;

            debug!(
                "Effector point: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                effector_point.x, effector_point.y, effector_point.z
            );
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
