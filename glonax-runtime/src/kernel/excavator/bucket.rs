use glonax_core::motion::Motion;

use crate::runtime::operand::*;

use super::Actuator;

pub struct BucketProgram;

impl BucketProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for BucketProgram {
    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![(Actuator::Bucket.into(), 255)]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.last_step.elapsed() >= std::time::Duration::from_secs(2)
    }
}
