use std::time::Duration;

use glonax_core::motion::Motion;

use crate::runtime::operand::*;

use super::Actuator;

/// Turn strait forward.
///
/// This program is part of the excavator kernel. It drives both tracks then
/// stops when the time limit is reached.
pub struct TurnProgram {
    drive_time: Duration,
}

const DRIVE_POWER: i16 = 200;

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

impl Program for TurnProgram {
    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![(Actuator::Slew.into(), DRIVE_POWER)]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.start.elapsed() >= self.drive_time
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![Actuator::Slew.into()]))
    }
}
