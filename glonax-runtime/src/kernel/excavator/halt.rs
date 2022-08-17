use crate::runtime::operand::*;

use super::HydraulicMotion;

pub(super) struct HaltProgram {}

impl HaltProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for HaltProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        None
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        true
    }

    fn term_action(&self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::StopAll)
    }
}
