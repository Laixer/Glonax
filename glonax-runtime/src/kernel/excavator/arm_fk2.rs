use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;

use super::HydraulicMotion;

pub struct ArmFk2Program {
    theta_boom: f32,
    theta_arm: f32,
}

// const BOOM_20: f32 = 0.3473205;
// const BOOM_20: f32 = 0.052838428;

impl ArmFk2Program {
    pub fn new() -> Self {
        Self {
            // theta_boom: 0.3473205, // 0 DEG
            // theta_arm: -1.5708,
            theta_boom: core::deg_to_rad(19.5),
            // theta_boom: 0.0,
            theta_arm: 0.0,
        }
    }
}

impl Program for ArmFk2Program {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Some(guard) = context.reader.try_lock() {
            // ARM
            if let Some(signal) = guard.most_recent_by_source(11) {
                if let MetricValue::Stroke(stroke) = signal.value {
                    let encoder_range = 355.0..3450.0;
                    let cylinder_range = 0.0..1665.0;
                    let angle_range = 0.0..2.1;

                    let encoder_value = {
                        let stroke = stroke.x as f32;
                        if stroke < encoder_range.start {
                            0.0
                        } else if stroke > encoder_range.end {
                            encoder_range.end - encoder_range.start
                        } else {
                            stroke - encoder_range.start
                        }
                    };

                    let delta = cylinder_range.end / (encoder_range.end - encoder_range.start);
                    let delta2 = angle_range.end / (encoder_range.end - encoder_range.start);
                    let delta3 = 100.0 / (encoder_range.end - encoder_range.start);
                    let dist = encoder_value * delta;
                    let dist2 = encoder_value * delta2;
                    let dist3 = encoder_value * delta3;

                    let angle_arm_offset = core::deg_to_rad(36.8);
                    let angle_arm_arch = self.theta_boom - angle_arm_offset - dist2;
                    // let angle_arm_arch = angle_arm_offset + dist2;
                    self.theta_arm = angle_arm_arch;

                    debug!(
                        "ARM Raw: {:?}; Stroke: {:.0} mm/{:.1} %; Rel. angle {:.2} deg; Arch angle {:.2} deg",
                        encoder_value,
                        dist,
                        dist3,
                        core::rad_to_deg(dist2),
                        core::rad_to_deg(angle_arm_arch)
                    );
                }
            }
            // BOOM
            if let Some(signal) = guard.most_recent_by_source(10) {
                if let MetricValue::Stroke(stroke) = signal.value {
                    let encoder_range = 345.0..2475.0;
                    // let cylinder_range = 0.0..1345.0;
                    let angle_range = 0.0..2.1;

                    // let delta = cylinder_range.end / (encoder_range.end - encoder_range.start);
                    let delta2 = angle_range.end / (encoder_range.end - encoder_range.start);
                    // let delta2 = 0.052838428;
                    // let dist = (stroke.x as f32 - encoder_range.start) * delta;
                    let _dist2 = (stroke.x as f32 - encoder_range.start) * delta2;

                    // self.theta_boom = dist2;

                    // debug!(
                    //     "BOOM Raw: {:?}; Stroke: {:.0} mm; Rel. angle {:.4} rad",
                    //     stroke.x, dist, dist2
                    // );
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
