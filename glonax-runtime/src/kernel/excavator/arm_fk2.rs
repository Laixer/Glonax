use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;
use crate::signal::Encoder;

use super::consts::*;
use super::HydraulicMotion;

pub(super) struct KinematicProgram {
    target_boom: f32,
    target_arm: f32,
    normal: super::body::DynamicBody,
    terminate: bool,
}

impl KinematicProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 2 {
            panic!("Expected 2 parameter, got {}", params.len());
        }

        let target = nalgebra::Point3::new(params[0], params[1], 0.0);

        let rigid_body = super::body::RigidBody {
            length_boom: BOOM_LENGTH,
            length_arm: ARM_LENGTH,
        };

        let target = super::body::DynamicBody::from_effector_point(rigid_body, target);

        Self {
            target_boom: target.angle_boom.unwrap(),
            target_arm: target.angle_arm.unwrap(),
            normal: super::body::DynamicBody::with_rigid_body(rigid_body),
            terminate: false,
        }
    }

    fn decode_signal(&mut self, reader: &crate::signal::SignalReader) {
        if let Some(guard) = reader.try_lock() {
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Boom.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let encoder = Encoder::new(BOOM_ENCODER_RANGE, BOOM_ANGLE_RANGE);

                    let angle = encoder.scale(value.x as f32);
                    let percentage = encoder.scale_to(100.0, value.x as f32);

                    let angle_offset = core::deg_to_rad(5.3);
                    let angle_at_datum = angle - angle_offset;

                    self.normal.update_boom_angle(angle_at_datum);
                    self.normal.update_slew_angle(0.0);

                    debug!(
                        "Boom Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum {:>+5.2}rad {:>+5.2}°",
                        value.x,
                        angle,
                        core::rad_to_deg(angle),
                        percentage,
                        angle_at_datum,
                        core::rad_to_deg(angle_at_datum)
                    );
                }
            }
            if let Some(signal) = guard.most_recent_by_source(super::BodyPart::Arm.into()) {
                if let MetricValue::Angle(value) = signal.value {
                    let encoder = Encoder::new(ARM_ENCODER_RANGE, ARM_ANGLE_RANGE);

                    let angle = encoder.scale(value.x as f32);
                    let percentage = encoder.scale_to(100.0, value.x as f32);

                    let angle_offset = core::deg_to_rad(36.8);

                    if let Some(angle_boom) = self.normal.angle_boom {
                        let angle_at_datum = angle_boom - angle_offset - (2.1 - angle);

                        self.normal.update_arm_angle(angle_at_datum);

                        debug!(
                            "Arm Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum {:>+5.2}rad {:>+5.2}°",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                            angle_at_datum,
                            core::rad_to_deg(angle_at_datum)
                        );
                    }
                }
            }
        }
    }
}

impl Program for KinematicProgram {
    type MotionPlan = HydraulicMotion;

    fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        self.decode_signal(&context.reader);

        if let Some(effector_point) = self.normal.effector_point() {
            let boom_point = self.normal.boom_point().unwrap();
            debug!(
                "Boom point AGL: X {:>+5.2} Y {:>+5.2}",
                boom_point.x, boom_point.y,
            );

            debug!(
                "Effector point AGL: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                effector_point.x, effector_point.y, effector_point.z,
            );
        };

        if let (Some(angle_boom), Some(angle_arm)) = (self.normal.angle_boom, self.normal.angle_arm)
        {
            let angle_boom_error = self.target_boom - angle_boom;
            let angle_arm_error = self.target_arm - angle_arm;

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
                "Boom Angle:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad  Power: {:>+5.2}",
                angle_boom,
                core::rad_to_deg(angle_boom),
                angle_boom_error,
                power_boom
            );
            debug!(
                "Arm Angle:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad  Power: {:>+5.2}",
                angle_arm,
                core::rad_to_deg(angle_arm),
                angle_arm_error,
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
