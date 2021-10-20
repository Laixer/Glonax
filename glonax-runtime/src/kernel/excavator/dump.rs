use std::convert::TryInto;

use glonax_core::{
    metric::MetricValue,
    motion::Motion,
    operand::{Context, Program},
    position::Position,
};

use crate::kernel::excavator::Metric;

pub struct DumpProgram;

impl DumpProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for DumpProgram {
    fn push(&mut self, id: u32, value: MetricValue, _context: &mut Context) {
        match value {
            MetricValue::Acceleration(acc) => {
                trace!("{:?}", acc);
                let pos = Position::from(acc.get_ref());
                let id: Metric = id.try_into().unwrap();
                trace!("ID: {:?} {:?}", id, pos);
            }
            _ => {}
        }
    }

    fn step(&mut self, context: &mut Context) -> Option<Motion> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
