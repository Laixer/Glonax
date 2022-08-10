use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;

use super::HydraulicMotion;

pub struct ArmFk2Program {
    theta_boom: f32,
    theta_arm: f32,
}

impl ArmFk2Program {
    pub fn new() -> Self {
        Self {
            theta_boom: 0.0,
            theta_arm: 0.0,
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
                    let encoder_range = 101.0..372.0;
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
                    let angle_at_datum = self.theta_boom - angle_offset - (2.1 - angle);
                    self.theta_arm = angle_at_datum;

                    debug!(
                        "Arm Signal: {:?}/{:?}\tAngle rel.: {:.2}rad {:.2}° {:.1}%\tAngle datum {:.2}rad {:.2}°",
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
                    let encoder_range = 765.0..913.0;
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
                    self.theta_boom = angle_at_datum;

                    debug!(
                        "Boom Signal: {:?}/{:?}\tAngle rel.: {:.2}rad {:.2}° {:.1}%\tAngle datum {:.2}rad {:.2}°",
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

            let boom_x = super::BOOM_LENGTH * self.theta_boom.cos();
            let boom_y = super::BOOM_LENGTH * self.theta_boom.sin();

            info!("Boom point: X {:>+5.2} Y {:>+5.2}", boom_x, boom_y);

            let arm_x = super::ARM_LENGTH * self.theta_arm.cos();
            let arm_y = super::ARM_LENGTH * self.theta_arm.sin();

            info!("Arm point: X {:>+5.2} Y {:>+5.2}", arm_x, arm_y);

            let fk_x = boom_x + arm_x;
            let fk_y = super::FRAME_HEIGHT + boom_y + arm_y;

            info!("Effector point: X {:>+5.2} Y {:>+5.2}", fk_x, fk_y);
        }

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
