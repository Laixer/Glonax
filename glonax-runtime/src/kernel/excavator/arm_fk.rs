use glonax_core::{metric::MetricValue, motion::Motion};

use crate::runtime::operand::*;

pub struct ArmFkProgram(glonax_core::nalgebra::Rotation2<f32>);

impl ArmFkProgram {
    pub fn new() -> Self {
        Self(glonax_core::nalgebra::Rotation2::new(0.0))
    }
}

impl Program for ArmFkProgram {
    fn push(&mut self, id: u32, value: MetricValue, _context: &mut Context) {
        trace!("ID {} ⇨ {}", id, value);

        match value {
            MetricValue::Temperature(_) => (),
            MetricValue::Acceleration(vec) => {
                self.0 = glonax_core::nalgebra::Rotation2::new(-vec.x.atan2(vec.y));
                debug!("XY Angle: {:?}", self.0.angle());
            }
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        let theta2: f32 = super::ARM_RANGE.end;

        let fk_y = super::BOOM_LENGTH * self.0.angle().sin()
            + (super::ARM_LENGTH * (self.0.angle() + theta2).sin());
        let fk_x = super::BOOM_LENGTH * self.0.angle().cos()
            + (super::ARM_LENGTH * (self.0.angle() + theta2).cos());
        let reach = glonax_core::nalgebra::Point2::new(fk_x, fk_y);
        println!("Effector point: {}", reach);

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
