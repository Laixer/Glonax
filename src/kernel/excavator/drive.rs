use crate::runtime::{Context, Motion, Program};

use super::Actuator;

/// Drive strait forward.
///
/// This program is part of the excavator kernel. It
/// drives both tracks straight forward for 5 seconds
/// then stops.
pub struct DriveProgram;

impl DriveProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for DriveProgram {
    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![
            (Actuator::LimpLeft.into(), 200),
            (Actuator::LimpRight.into(), 200),
        ]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        let sec_since_boot = context.start.elapsed().as_secs();
        info!("Programm running for {} seconds", sec_since_boot);
        sec_since_boot >= 5
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![
            Actuator::LimpLeft.into(),
            Actuator::LimpRight.into(),
        ]))
    }
}
