use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

pub struct BucketProgram;

impl BucketProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for BucketProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, _context: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Change(vec![(Actuator::Bucket, 255)]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.last_step.elapsed() >= std::time::Duration::from_secs(2)
    }
}
