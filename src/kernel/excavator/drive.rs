use crate::{
    device::MetricValue,
    runtime::{Context, Motion, Program},
};

use super::Actuator;

pub struct DriveProgram;

impl DriveProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for DriveProgram {
    fn boot(&mut self, _: &mut Context) {
        info!("Drive program called");
    }

    fn push(&mut self, id: u32, value: MetricValue, _: &mut Context) {
        match value {
            MetricValue::Temperature(value) => info!(
                "Temperature metric pushed with id: {}; value: {:?}",
                id, value
            ),
            MetricValue::Position(value) => {
                info!("Position metric pushed with id: {}; value: {:?}", id, value)
            }
        }
    }

    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Change(vec![
            (Actuator::LimpLeft.into(), 200),
            (Actuator::LimpRight.into(), 200),
        ]))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        let sec_since_boot = context.start.elapsed().as_secs();
        info!("Running for {} seconds now", sec_since_boot);
        sec_since_boot >= 5
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![
            Actuator::LimpLeft.into(),
            Actuator::LimpRight.into(),
        ]))
    }
}
