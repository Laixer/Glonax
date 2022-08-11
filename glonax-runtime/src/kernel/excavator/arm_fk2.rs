use crate::algorithm::fk::ForwardKinematics;
use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;

use super::HydraulicMotion;

pub struct ArmFk2Program {
    theta_boom: Option<f32>,
    theta_arm: Option<f32>,
    terminate: bool,
}

impl ArmFk2Program {
    pub fn new() -> Self {
        Self {
            theta_boom: None,
            theta_arm: None,
            terminate: false,
        }
    }
}

impl Program for ArmFk2Program {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Some(guard) = context.reader.try_lock() {
            // ARM
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Arm.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let encoder_range = 249.0..511.0;
                    let angle_range = 0.0..2.1;

                    let domain_value = {
                        let value = value.x as f32;
                        let domain_value = if value < encoder_range.start {
                            encoder_range.start
                        } else if value > encoder_range.end {
                            encoder_range.end
                        } else {
                            value
                        };

                        domain_value - encoder_range.start
                    };

                    let delta_radian = angle_range.end / (encoder_range.end - encoder_range.start);
                    let delta_percent = 100.0 / (encoder_range.end - encoder_range.start);
                    let angle = domain_value * delta_radian;
                    let percentage = domain_value * delta_percent;

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
            // BOOM
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Boom.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let encoder_range = 517.0..665.0;
                    let angle_range = 0.0..1.178;

                    let domain_value = {
                        let value = value.x as f32;
                        let domain_value = if value < encoder_range.start {
                            encoder_range.start
                        } else if value > encoder_range.end {
                            encoder_range.end
                        } else {
                            value
                        };

                        domain_value - encoder_range.start
                    };

                    let delta_radian = angle_range.end / (encoder_range.end - encoder_range.start);
                    let delta_percent = 100.0 / (encoder_range.end - encoder_range.start);
                    let angle = domain_value * delta_radian;
                    let percentage = domain_value * delta_percent;

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
            let boom_x = super::BOOM_LENGTH * theta_boom.cos();
            let boom_y = super::BOOM_LENGTH * theta_boom.sin();

            info!("Boom point: X {:>+5.2} Y {:>+5.2}", boom_x, boom_y);

            let arm_x = super::ARM_LENGTH * theta_arm.cos();
            let arm_y = super::ARM_LENGTH * theta_arm.sin();

            info!("Arm point: X {:>+5.2} Y {:>+5.2}", arm_x, arm_y);

            let fk_x = boom_x + arm_x;
            let fk_y = super::FRAME_HEIGHT + boom_y + arm_y;

            info!("Effector point: X {:>+5.2} Y {:>+5.2}", fk_x, fk_y);

            ////////////////////////////////////////////

            let fk = ForwardKinematics::new(super::BOOM_LENGTH, super::ARM_LENGTH);

            let mut effector_point = fk.solve((0.0, theta_boom, theta_arm));
            effector_point.y += super::FRAME_HEIGHT;

            debug!(
                "Effector point AGL: X {:>+5.2} Y {:>+5.2}",
                effector_point.x, effector_point.y,
            );
        }

        if let (Some(theta_boom), Some(theta_arm)) = (self.theta_boom, self.theta_arm) {
            // if let Some(theta_arm) = self.theta_arm {
            // if let Some(theta_boom) = self.theta_boom {
            const ARM_SET_ANGLE: f32 = 0.0;
            const ARM_SPEED_MAX: i16 = 32_000;
            const ARM_SPEED_MIN: i16 = 5_000;

            const BOOM_SET_ANGLE: f32 = 1.08;
            const BOOM_SPEED_MAX: i16 = 15_000;
            const BOOM_SPEED_MIN: i16 = 5_000;

            // const ACTUATOR: super::Actuator = super::Actuator::Arm;

            let angle_arm_error = ARM_SET_ANGLE - theta_arm;
            let angle_boom_error = BOOM_SET_ANGLE - theta_boom;

            let power_arm = (angle_arm_error * 10.0 * 1_000.0) as i16;
            let power_arm = if angle_arm_error.is_sign_positive() {
                power_arm.min(20_000) + 12_000
            } else {
                power_arm.max(-20_000) - 12_000
            };

            let power_boom = (angle_boom_error * 10.0 * 1_500.0) as i16;
            let power_boom = if angle_boom_error.is_sign_positive() {
                (-power_boom).max(-22_000) - 10_000
            } else {
                ((-power_boom).min(22_000) + 10_000) //.min(BOOM_SPEED_MAX)
            };

            let power_arm = if angle_arm_error.abs() < 0.03 {
                0
            } else {
                power_arm
            };
            let power_boom = if angle_boom_error.abs() < 0.03 {
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

            // if angle_arm_error.abs() < 0.03 && angle_boom_error.abs() < 0.03 {
            //     self.terminate = true;
            //     Some(HydraulicMotion::Stop(vec![
            //         super::Actuator::Arm,
            //         super::Actuator::Boom,
            //     ]))
            // } else {
            //     Some(HydraulicMotion::Change(vec![
            //         (super::Actuator::Arm, power_arm),
            //         (super::Actuator::Boom, power_boom),
            //     ]))
            // }
            None
        } else {
            None
        }
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        self.terminate
    }
}
