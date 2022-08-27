use std::time::Duration;

use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

const DRIVE_POWER: i16 = -20_096;

/// Turn strait forward.
///
/// This program is part of the excavator kernel. It drives both tracks then
/// stops when the time limit is reached.
pub(super) struct TurnProgram {
    drive_time: Duration,
}

impl TurnProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 1 {
            panic!("Expected 1 parameter, got {}", params.len());
        } else if params[0] == 0.0 {
            panic!("Duration cannot be zero");
        }

        Self {
            drive_time: Duration::from_secs(params[0] as u64),
        }
    }
}

#[async_trait::async_trait]
impl Program for TurnProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Change(vec![(Actuator::Slew, DRIVE_POWER)]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.start.elapsed() >= self.drive_time
    }

    fn term_action(&self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Stop(vec![Actuator::Slew]))
    }
}
