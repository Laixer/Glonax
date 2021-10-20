use std::convert::TryInto;

use glonax_core::{
    metric::MetricValue,
    motion::Motion,
    operand::{Context, Program},
    position::Position,
};

use crate::kernel::excavator::Metric;

pub struct ArmProgram;

impl ArmProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for ArmProgram {
    fn push(&mut self, id: u32, value: MetricValue, _context: &mut Context) {
        match value {
            MetricValue::Acceleration(acc) => {
                let pos = Position::from(acc.get_ref());
                let id: Metric = id.try_into().unwrap();
                trace!("ID: {:?} {:?}", id, pos);
            }
            _ => {}
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
