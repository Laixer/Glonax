use crate::{
    device::MetricValue,
    runtime::{Motion, Program},
};

use super::Actuator;

pub struct DriveProgram {
    start: std::time::Instant,
}

impl DriveProgram {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Program for DriveProgram {
    fn boot(&mut self) {
        info!("Drive program called");
        self.start = std::time::Instant::now();
    }

    fn push(&mut self, id: u32, value: MetricValue) {
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

    fn step(&mut self) -> Option<Motion> {
        Some(Motion::Change(vec![
            (Actuator::LimpLeft.into(), 200),
            (Actuator::LimpRight.into(), 200),
        ]))
    }

    fn can_terminate(&self) -> bool {
        let sec_since_boot = self.start.elapsed().as_secs();
        info!("Running for {} seconds now", sec_since_boot);
        sec_since_boot >= 5
    }

    fn term_action(&self) -> Option<Motion> {
        Some(Motion::Stop(vec![
            Actuator::LimpLeft.into(),
            Actuator::LimpRight.into(),
        ]))
    }
}
