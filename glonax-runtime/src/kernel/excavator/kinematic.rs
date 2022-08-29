use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;
use crate::signal::Encoder;

use super::consts::*;
use super::HydraulicMotion;

pub(super) struct KinematicProgram {
    normal: super::body::DynamicBody,
    target: super::body::DynamicBody,
    terminate: bool,
}

impl KinematicProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 2 {
            panic!("Expected 2 parameter, got {}", params.len());
        }

        let effector_point = nalgebra::Point3::new(params[0], params[1], 0.0);

        let rigid_body = super::body::RigidBody {
            length_boom: BOOM_LENGTH,
            length_arm: ARM_LENGTH,
        };

        Self {
            normal: super::body::DynamicBody::with_rigid_body(rigid_body),
            target: super::body::DynamicBody::from_effector_point(rigid_body, effector_point),
            terminate: false,
        }
    }

    async fn decode_signal(&mut self, reader: &mut crate::signal::SignalReader) {
        if let Ok((source, signal)) = reader.recv().await {
            match source {
                super::BODY_PART_BOOM => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(BOOM_ENCODER_RANGE, BOOM_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_offset = core::deg_to_rad(5.3);
                        let angle_at_datum = angle - angle_offset;

                        self.normal.update_boom_angle(angle_at_datum);
                        self.normal.update_slew_angle(0.0);

                        debug!(
                            "Boom Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum: {:>+5.2}rad {:>+5.2}°",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                            angle_at_datum,
                            core::rad_to_deg(angle_at_datum)
                        );
                    }
                }
                super::BODY_PART_ARM => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(ARM_ENCODER_RANGE, ARM_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_offset = core::deg_to_rad(36.8);
                        let angle_at_datum = -angle_offset - (2.1 - angle);

                        self.normal.update_arm_angle(angle_at_datum);

                        debug!(
                            "Arm Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum: {:>+5.2}rad {:>+5.2}°",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                            angle_at_datum,
                            core::rad_to_deg(angle_at_datum)
                        );
                    }
                }
                super::BODY_PART_BUCKET => todo!(),
                _ => {}
            }
        }
    }
}

#[async_trait::async_trait]
impl Program for KinematicProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        self.decode_signal(&mut context.reader).await;

        if let Some(effector_point) = self.normal.effector_point() {
            debug!(
                "Effector point AGL: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                effector_point.x, effector_point.y, effector_point.z,
            );
        };

        if let Some((angle_boom_error, angle_arm_error)) = self.target.erorr_diff(&self.normal) {
            let power_boom = (angle_boom_error * 10.0 * 1_500.0) as i16;
            let power_boom = if angle_boom_error.is_sign_positive() {
                (-power_boom).max(-20_000) - 12_000
            } else {
                (-power_boom).min(20_000) + 12_000
            };

            let power_arm = (angle_arm_error * 10.0 * 1_500.0) as i16;
            let power_arm = if angle_arm_error.is_sign_positive() {
                power_arm.min(20_000) + 12_000
            } else {
                power_arm.max(-20_000) - 12_000
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
                "Normal Boom:\t {:>+5.2}rad {:>+5.2}°  Target Boom:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°  Power: {:>+5.2}",
                self.normal.angle_boom.unwrap(),
                core::rad_to_deg(self.normal.angle_boom.unwrap()),
                self.target.angle_boom.unwrap(),
                core::rad_to_deg(self.target.angle_boom.unwrap()),
                angle_boom_error,
                core::rad_to_deg(angle_boom_error),
                power_boom
            );
            debug!(
                "Normal Arm:\t\t {:>+5.2}rad {:>+5.2}°  Target Arm:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°  Power: {:>+5.2}",
                self.normal.angle_arm.unwrap(),
                core::rad_to_deg(self.normal.angle_arm.unwrap()),
                self.target.angle_arm.unwrap(),
                core::rad_to_deg(self.target.angle_arm.unwrap()),
                angle_arm_error,
                core::rad_to_deg(angle_arm_error),
                power_arm
            );

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
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        self.terminate
    }
}
