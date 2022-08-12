use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

pub(super) struct TestProgram {
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
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Boom,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            1 => {
                info!("Testing actuator: boom down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Boom,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            2 => {
                info!("Testing actuator: arm up");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Arm,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            3 => {
                info!("Testing actuator: arm down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Arm,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            4 => {
                info!("Testing actuator: bucket up");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Bucket,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            5 => {
                info!("Testing actuator: bucket down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Bucket,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            6 => {
                info!("Testing actuator: slew up");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Slew,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            7 => {
                info!("Testing actuator: slew down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::Slew,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            8 => {
                info!("Testing actuator: drive left up");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::LimpLeft,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            9 => {
                info!("Testing actuator: drive left down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::LimpLeft,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            10 => {
                info!("Testing actuator: drive right up");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::LimpRight,
                    HydraulicMotion::POWER_MAX,
                )]))
            }
            11 => {
                info!("Testing actuator: drive right down");
                Some(HydraulicMotion::Change(vec![(
                    Actuator::LimpRight,
                    -HydraulicMotion::POWER_MAX,
                )]))
            }
            12 => {
                info!("Testing actuator: drive straight up");
                Some(HydraulicMotion::StraightDrive(HydraulicMotion::POWER_MAX))
            }
            13 => {
                info!("Testing actuator: drive straight down");
                Some(HydraulicMotion::StraightDrive(-HydraulicMotion::POWER_MAX))
            }
            14 => {
                info!("Testing all actuators up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_MAX),
                    (Actuator::Arm, HydraulicMotion::POWER_MAX),
                    (Actuator::Bucket, HydraulicMotion::POWER_MAX),
                    (Actuator::Slew, HydraulicMotion::POWER_MAX),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_MAX),
                    (Actuator::LimpRight, HydraulicMotion::POWER_MAX),
                ]))
            }
            15 => {
                info!("Testing all actuators down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, -HydraulicMotion::POWER_MAX),
                    (Actuator::Arm, -HydraulicMotion::POWER_MAX),
                    (Actuator::Bucket, -HydraulicMotion::POWER_MAX),
                    (Actuator::Slew, -HydraulicMotion::POWER_MAX),
                    (Actuator::LimpLeft, -HydraulicMotion::POWER_MAX),
                    (Actuator::LimpRight, -HydraulicMotion::POWER_MAX),
                ]))
            }
            _ => {
                self.program = 0;
                None
            }
        }
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        false
    }
}
