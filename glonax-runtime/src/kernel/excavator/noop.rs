use crate::runtime::operand::*;

use super::HydraulicMotion;

pub struct NoopProgram;

impl NoopProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for NoopProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        if let Some(guard) = context.reader.try_lock() {
            if let Some((source, signal)) = guard.most_recent() {
                debug!("Source {} ⇨ {}", source, signal.value);
            }
        }

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
