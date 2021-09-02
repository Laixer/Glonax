use crate::{device::MetricValue, runtime::NormalControl};

use super::Program;

pub struct DriveProgram {}

impl DriveProgram {
    pub fn new() -> Self {
        Self {}
    }
}

impl Program for DriveProgram {
    type Motion = NormalControl;

    fn boot(&mut self) {
        info!("Drive program called")
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

    fn step(&mut self) -> Option<Self::Motion> {
        todo!()
    }

    fn can_terminate(&self) -> bool {
        todo!()
    }
}
