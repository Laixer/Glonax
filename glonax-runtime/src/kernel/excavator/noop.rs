use glonax_core::motion::Motion;

use crate::runtime::operand::*;

pub struct NoopProgram;

impl NoopProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for NoopProgram {
    fn step(&mut self, context: &mut Context) -> Option<Motion> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
