use crate::runtime::operand::*;

use super::HydraulicMotion;

pub(super) struct NoopProgram;

impl NoopProgram {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Program for NoopProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        if let Ok((source, signal)) = context.reader.recv().await {
            debug!("Source {} ⇨ {}", source, signal.value);
        }

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
