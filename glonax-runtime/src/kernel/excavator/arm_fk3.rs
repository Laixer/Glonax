use crate::{algorithm::fk::ForwardKinematics, core::metric::MetricValue, runtime::operand::*};

use super::HydraulicMotion;

pub struct ArmFk3Program {
    boom_angle: f32,
    arm_angle: f32,
}

impl ArmFk3Program {
    pub fn new() -> Self {
        Self {
            boom_angle: 0.0,
            arm_angle: 0.0,
        }
    }
}

impl Program for ArmFk3Program {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Some(guard) = context.reader.try_lock() {
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Arm.into()) {
                if let MetricValue::Acceleration(vec) = signal.value {
                    self.arm_angle = -vec.x.atan2(vec.y);
                }
            }
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Boom.into()) {
                if let MetricValue::Acceleration(vec) = signal.value {
                    self.boom_angle = vec.x.atan2(vec.y);
                }
            }
        }

        let fk_x = (super::BOOM_LENGTH * self.boom_angle.cos())
            + (super::ARM_LENGTH * self.arm_angle.cos());
        let fk_y = (super::BOOM_LENGTH * self.boom_angle.sin())
            + (super::ARM_LENGTH * -self.arm_angle.sin())
            + super::FRAME_HEIGHT;

        debug!("Effector point: X {:>+5.2} Y {:>+5.2}", fk_x, fk_y);

        None
        // Some(HydraulicMotion::Change(vec![(super::Actuator::Arm, 12_096)]))
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
