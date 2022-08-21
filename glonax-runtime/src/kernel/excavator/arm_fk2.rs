use crate::algorithm::fk::ForwardKinematics;
use crate::algorithm::ik::InverseKinematics;
use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;
use crate::signal::Scalar;

use super::HydraulicMotion;

pub(super) struct ArmFk2Program {
    target_boom: f32,
    target_arm: f32,
    theta_boom: Option<f32>,
    theta_arm: Option<f32>,
    terminate: bool,
}

impl ArmFk2Program {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 2 {
            panic!("Expected 2 parameter, got {}", params.len());
        }

        let ik = InverseKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

        // let target = nalgebra::Point3::new(params[0], params[1], 0.0);
        let target = nalgebra::Point3::new(5.21, 0.0, 0.0);

        if let Some((theta_0, theta_1, theta_2)) = ik.solve(target) {
            debug!(
                "Theta 0 {:>+5.2} Theta 1 {:>+5.2} Theta 2 {:>+5.2}",
                theta_0, theta_1, theta_2
            );

            Self {
                target_boom: theta_1,
                target_arm: theta_2,
                theta_boom: None,
                theta_arm: None,
                terminate: false,
            }
        } else {
            // TODO: exit with result
            warn!("Target out of range");

            Self {
                target_boom: 0.0,
                target_arm: 0.0,
                theta_boom: None,
                theta_arm: None,
                terminate: true,
            }
        }
    }
}

const ARM_ENCODER_RANGE: std::ops::Range<f32> = 249.0..511.0;
const ARM_ANGLE_RANGE: std::ops::Range<f32> = 0.0..2.1;

const BOOM_ENCODER_RANGE: std::ops::Range<f32> = 517.0..665.0;
const BOOM_ANGLE_RANGE: std::ops::Range<f32> = 0.0..1.178;

const ENACT: bool = true;

impl Program for ArmFk2Program {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Some(guard) = context.reader.try_lock() {
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Arm.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let w = Scalar::new(ARM_ENCODER_RANGE);

                    let domain_value = w.normalize(value.x as f32);

                    let angle = w.scale(ARM_ANGLE_RANGE.end, domain_value);
                    let percentage = w.scale(100.0, domain_value);

                    let angle_offset = core::deg_to_rad(36.8);
                    let angle_at_datum =
                        self.theta_boom.unwrap_or(0.0) - angle_offset - (2.1 - angle);
                    self.theta_arm = Some(angle_at_datum);

                    debug!(
                        "Arm Signal: {:?}/{:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum {:>+5.2}rad {:>+5.2}°",
                        value.x,
                        domain_value,
                        angle,
                        core::rad_to_deg(angle),
                        percentage,
                        angle_at_datum,
                        core::rad_to_deg(angle_at_datum)
                    );
                }
            }
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Boom.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let w = Scalar::new(BOOM_ENCODER_RANGE);

                    let domain_value = w.normalize(value.x as f32);

                    let angle = w.scale(BOOM_ANGLE_RANGE.end, domain_value);
                    let percentage = w.scale(100.0, domain_value);

                    let angle_offset = core::deg_to_rad(5.3);
                    let angle_at_datum = angle - angle_offset;
                    self.theta_boom = Some(angle_at_datum);

                    debug!(
                        "Boom Signal: {:?}/{:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum {:>+5.2}rad {:>+5.2}°",
                        value.x,
                        domain_value,
                        angle,
                        core::rad_to_deg(angle),
                        percentage,
                        angle_at_datum,
                        core::rad_to_deg(angle_at_datum)
                    );
                }
            }
        }

        if let (Some(theta_boom), Some(theta_arm)) = (self.theta_boom, self.theta_arm) {
            let fk = ForwardKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

            let mut effector_point = fk.solve((0.0, theta_boom, theta_arm));
            effector_point.y += super::FRAME_HEIGHT;

            debug!(
                "Effector point AGL: X {:>+5.2} Y {:>+5.2}",
                effector_point.x, effector_point.y,
            );
        }

        if let (Some(theta_boom), Some(theta_arm)) = (self.theta_boom, self.theta_arm) {
            let angle_arm_error = self.target_arm - theta_arm;
            let angle_boom_error = self.target_boom - theta_boom;

            let power_arm = (angle_arm_error * 10.0 * 1_500.0) as i16;
            let power_arm = if angle_arm_error.is_sign_positive() {
                power_arm.min(20_000) + 12_000
            } else {
                power_arm.max(-20_000) - 12_000
            };

            let power_boom = (angle_boom_error * 10.0 * 1_500.0) as i16;
            let power_boom = if angle_boom_error.is_sign_positive() {
                (-power_boom).max(-20_000) - 12_000
            } else {
                (-power_boom).min(20_000) + 12_000
            };

            let power_arm = if angle_arm_error.abs() < 0.02 {
                0
            } else {
                power_arm
            };
            let power_boom = if angle_boom_error.abs() < 0.02 {
                0
            } else {
                power_boom
            };

            debug!(
                "Arm Angle:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad  Power: {:>+5.2}",
                theta_arm,
                core::rad_to_deg(theta_arm),
                angle_arm_error,
                power_arm
            );
            debug!(
                "Boom Angle:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad  Power: {:>+5.2}",
                theta_boom,
                core::rad_to_deg(theta_boom),
                angle_boom_error,
                power_boom
            );

            if ENACT {
                if angle_arm_error.abs() < 0.02 && angle_boom_error.abs() < 0.02 {
                    self.terminate = true;
                    Some(HydraulicMotion::Stop(vec![
                        super::Actuator::Arm,
                        super::Actuator::Boom,
                    ]))
                } else {
                    Some(HydraulicMotion::Change(vec![
                        (super::Actuator::Arm, power_arm),
                        (super::Actuator::Boom, power_boom),
                    ]))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        self.terminate
    }
}
