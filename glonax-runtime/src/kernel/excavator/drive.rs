use glonax_core::{
    motion::Motion,
    operand::{Context, Program},
};

use super::Actuator;

/// Drive strait forward.
///
/// This program is part of the excavator kernel. It
/// drives both tracks straight forward for 5 seconds
/// then stops.
pub struct DriveProgram;

const DRIVE_SPEED_LEFT: i16 = 200;
const DRIVE_SPEED_RIGHT: i16 = 200;
const DRIVE_TIME: u64 = 5;

impl DriveProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for DriveProgram {
    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![
            (Actuator::LimpLeft.into(), DRIVE_SPEED_LEFT),
            (Actuator::LimpRight.into(), DRIVE_SPEED_RIGHT),
        ]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        let sec_since_boot = context.start.elapsed().as_secs();
        info!("Programm running for {} seconds", sec_since_boot);
        sec_since_boot >= DRIVE_TIME
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![
            Actuator::LimpLeft.into(),
            Actuator::LimpRight.into(),
        ]))
    }
}
