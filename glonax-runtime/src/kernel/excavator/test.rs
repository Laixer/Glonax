use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

pub(super) struct TestProgram {
    program: u32,
}

impl TestProgram {
    pub fn new() -> Self {
        Self { program: 0 }
    }
}

impl Program for TestProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, _: &mut Context) -> Option<Self::MotionPlan> {
        match self.program {
            0 => {
                self.program += 1;

                debug!("Testing actuator: boom up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_MAX),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            1 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: boom down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_MIN),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            2 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: arm up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_MAX),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            3 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: arm down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_MIN),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            4 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: bucket up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_MAX),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            5 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: bucket down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_MIN),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            6 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: slew up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_MAX),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            7 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: slew down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_MIN),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            8 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive left up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_MAX),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            9 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive left down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_MIN),
                    (Actuator::LimpRight, HydraulicMotion::POWER_NEUTRAL),
                ]))
            }
            10 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive right up");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_MAX),
                ]))
            }
            11 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive right down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Arm, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Bucket, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::Slew, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_NEUTRAL),
                    (Actuator::LimpRight, HydraulicMotion::POWER_MIN),
                ]))
            }
            12 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive straight up");
                Some(HydraulicMotion::StraightDrive(HydraulicMotion::POWER_MAX))
            }
            13 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing actuator: drive straight down");
                Some(HydraulicMotion::StraightDrive(HydraulicMotion::POWER_MIN))
            }
            14 => {
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing all actuators up");
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
                self.program += 1;

                std::thread::sleep(std::time::Duration::from_secs(5));

                debug!("Testing all actuators down");
                Some(HydraulicMotion::Change(vec![
                    (Actuator::Boom, HydraulicMotion::POWER_MIN),
                    (Actuator::Arm, HydraulicMotion::POWER_MIN),
                    (Actuator::Bucket, HydraulicMotion::POWER_MIN),
                    (Actuator::Slew, HydraulicMotion::POWER_MIN),
                    (Actuator::LimpLeft, HydraulicMotion::POWER_MIN),
                    (Actuator::LimpRight, HydraulicMotion::POWER_MIN),
                ]))
            }
            _ => {
                self.program = 0;

                std::thread::sleep(std::time::Duration::from_secs(5));

                None
            }
        }
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        false
    }
}
