use std::time::Duration;

use crate::runtime::operand::*;

use super::HydraulicMotion;

pub(super) struct SleepProgram {
    time: Duration,
}

impl SleepProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 1 {
            panic!("Expected 1 parameter, got {}", params.len());
        } else if params[0] == 0.0 {
            panic!("Duration cannot be zero");
        }

        Self {
            time: std::time::Duration::from_secs(params[0] as u64),
        }
    }
}

#[async_trait::async_trait]
impl Program for SleepProgram {
    type MotionPlan = HydraulicMotion;

    fn boot(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::StopAll)
    }

    async fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        tokio::time::sleep(self.time).await;

        None
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.start.elapsed() >= self.time
    }
}
