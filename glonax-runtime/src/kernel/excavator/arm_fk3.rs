use crate::{
    algorithm::{fk::ForwardKinematics, ik::InverseKinematics},
    core::{metric::MetricValue, rad_to_deg},
    runtime::operand::*,
};

use super::HydraulicMotion;

const ARM_SET_ANGLE: f32 = -1.5708;
const ARM_SPEED_MAX: i16 = 12_000;
const ARM_SPEED_MIN: i16 = 5_000;

const BOOM_SET_ANGLE: f32 = 0.80;

pub(super) struct ArmFk3Program {
    boom_angle: Option<f32>,
    arm_angle: Option<f32>,
    done: bool,
}

impl ArmFk3Program {
    pub fn new() -> Self {
        Self {
            // boom_angle: Some(0.349066),
            boom_angle: None,
            // arm_angle: None,
            arm_angle: None,
            done: false,
        }
    }
}

impl Program for ArmFk3Program {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Some(guard) = context.reader.try_lock() {
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Boom.into()) {
                if let MetricValue::Acceleration(vec) = signal.value {
                    self.boom_angle = Some(vec.x.atan2(-vec.y));
                }
            }
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Arm.into()) {
                if let MetricValue::Acceleration(vec) = signal.value {
                    self.arm_angle = Some(vec.x.atan2(-vec.y));
                }
            }
        }

        if let (Some(boom_angle), Some(arm_angle)) = (self.boom_angle, self.arm_angle) {
            let fk = ForwardKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

            let mut effector_point = fk.solve((0.0, boom_angle, arm_angle));
            effector_point.y += super::FRAME_HEIGHT;

            debug!(
                "Effector point AGL: X {:>+5.2} Y {:>+5.2}",
                effector_point.x, effector_point.y,
            );

            const MOVE_DOWN: bool = false;
            const ACTUATOR: super::Actuator = super::Actuator::Boom;

            // let err = ARM_SET_ANGLE - arm_angle;
            let angle_error = BOOM_SET_ANGLE - boom_angle;

            let power = (angle_error * 10.0 * 1_000.0) as i16;
            let power = if MOVE_DOWN {
                (power - 5_000).max(-ARM_SPEED_MAX)
            } else {
                (power + 5_000).min(ARM_SPEED_MAX)
            };

            debug!(
                "Angle: {:>+5.2}  Err {:>+5.2}  Pwr: {:>+5.2}",
                rad_to_deg(boom_angle),
                angle_error,
                power
            );

            if angle_error.abs() < 0.03 {
                self.done = true;
                Some(HydraulicMotion::Stop(vec![ACTUATOR]))
            } else {
                Some(HydraulicMotion::Change(vec![(ACTUATOR, power)]))
            }
        } else {
            None
        }
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        // self.done
        false
    }
}
