use glonax_core::{metric::MetricValue, motion::Motion};

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

            let theta1: f32 = (super::ARM_LENGTH / super::BOOM_LENGTH).asin();
            let theta2: f32 = signal_angle;

            let fk_x = (super::BOOM_LENGTH * theta1.cos()) + (super::ARM_LENGTH * theta2.cos());
            let fk_y = (super::BOOM_LENGTH * theta1.sin()) + (super::ARM_LENGTH * theta2.sin());

            let fk_y = fk_y + super::FRAME_HEIGHT;

            let reach = glonax_core::nalgebra::Point2::new(fk_x, fk_y);
            debug!("Effector point: X {:>+5.2} Y {:>+5.2}", reach.x, reach.y);
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
