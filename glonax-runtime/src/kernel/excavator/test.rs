use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

const POWER: i16 = 32_000;

pub struct TestProgram {
    time: std::time::Instant,
    program: u32,
}

impl TestProgram {
    pub fn new() -> Self {
        Self {
            time: std::time::Instant::now(),
            program: 0,
        }
    }
}

impl Program for TestProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        if self.time.elapsed().as_secs() >= 5 {
            self.time = std::time::Instant::now();
            self.program += 1;

            debug!("DONE DONE DONE");

            return Some(HydraulicMotion::StopAll);
        }

        match self.program {
            0 => {
                info!("Testing actuator: boom up");
                Some(HydraulicMotion::Change(vec![(Actuator::Boom, POWER)]))
            }
            1 => {
                info!("Testing actuator: boom down");
                Some(HydraulicMotion::Change(vec![(Actuator::Boom, -POWER)]))
            }
            2 => {
                info!("Testing actuator: arm up");
                Some(HydraulicMotion::Change(vec![(Actuator::Arm, POWER)]))
            }
            3 => {
                info!("Testing actuator: arm down");
                Some(HydraulicMotion::Change(vec![(Actuator::Arm, -POWER)]))
            }
            4 => {
                info!("Testing actuator: bucket up");
                Some(HydraulicMotion::Change(vec![(Actuator::Bucket, POWER)]))
            }
            5 => {
                info!("Testing actuator: bucket down");
                Some(HydraulicMotion::Change(vec![(Actuator::Bucket, -POWER)]))
            }
            6 => {
                info!("Testing actuator: slew up");
                Some(HydraulicMotion::Change(vec![(Actuator::Slew, POWER)]))
            }
            7 => {
                info!("Testing actuator: slew down");
                Some(HydraulicMotion::Change(vec![(Actuator::Slew, -POWER)]))
            }
            8 => {
                info!("Testing actuator: drive left up");
                Some(HydraulicMotion::Change(vec![(Actuator::LimpLeft, POWER)]))
            }
            9 => {
                info!("Testing actuator: drive left down");
                Some(HydraulicMotion::Change(vec![(Actuator::LimpLeft, -POWER)]))
            }
            10 => {
                info!("Testing actuator: drive right up");
                Some(HydraulicMotion::Change(vec![(Actuator::LimpRight, POWER)]))
            }
            11 => {
                info!("Testing actuator: drive right down");
                Some(HydraulicMotion::Change(vec![(Actuator::LimpRight, -POWER)]))
            }
            12 => {
                info!("Testing actuator: drive straight up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::LimpLeft, POWER),
                    (Actuator::LimpRight, POWER),
                ]))
            }
            13 => {
                info!("Testing actuator: drive straight down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::LimpLeft, -POWER),
                    (Actuator::LimpRight, -POWER),
                ]))
            }
            _ => {
                self.program = 0;
                None
            }
        }
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        false
    }
}
