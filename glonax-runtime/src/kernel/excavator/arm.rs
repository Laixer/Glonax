use std::convert::TryInto;

use glonax_core::{
    metric::MetricValue,
    motion::Motion,
    operand::{Context, Program},
    position::Position,
};

use crate::kernel::excavator::Metric;

pub struct ArmProgram;

const DRIVE_TIME: u64 = 600;

impl ArmProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for ArmProgram {
    fn push(&mut self, id: u32, value: MetricValue, _: &mut Context) {
        match value {
            MetricValue::Acceleration(acc) => {
                let pos = Position::from(acc.get_ref());
                let id: Metric = id.try_into().unwrap();
                trace!("ID: {:?} {:?}", id, pos);
            }
            _ => {}
        }
    }

    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        None
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        let sec_since_boot = context.start.elapsed().as_secs();
        sec_since_boot >= DRIVE_TIME
    }
}