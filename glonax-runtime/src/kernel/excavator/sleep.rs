use std::time::Duration;

use crate::runtime::program::*;

use super::HydraulicMotion;

pub(super) struct SleepProgram {
    time: Duration,
}

impl SleepProgram {
    pub fn new(params: &Vec<f32>) -> Self {
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

    /// Boot the program.
    ///
    /// This method is called when the runtime accepted
    /// this progam and started its routine.
    fn boot(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::StopAll)
    }

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    async fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        tokio::time::sleep(self.time).await;

        None
    }

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, context: &mut Context) -> bool {
        context.start.elapsed() >= self.time
    }
}
