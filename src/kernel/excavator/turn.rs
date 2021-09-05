use crate::runtime::{Context, Motion, Program};

use super::Actuator;

/// Turn strait forward.
///
/// This program is part of the excavator kernel. It
/// drives both tracks straight forward for 5 seconds
/// then stops.
pub struct TurnProgram;

const DRIVE_SPEED: i16 = 200;
const DRIVE_TIME: u64 = 5;

impl TurnProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for TurnProgram {
    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![(Actuator::Slew.into(), DRIVE_SPEED)]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        let sec_since_boot = context.start.elapsed().as_secs();
        sec_since_boot >= DRIVE_TIME
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![Actuator::Slew.into()]))
    }
}
