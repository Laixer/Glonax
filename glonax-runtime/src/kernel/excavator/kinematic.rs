use crate::core::{self, metric::MetricValue};

use crate::runtime::operand::*;
use crate::signal::Encoder;

use super::consts::*;
use super::HydraulicMotion;

pub(super) struct KinematicProgram {
    model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    objective: super::body::Objective,
}

impl KinematicProgram {
    pub fn new(
        model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
        params: Parameter,
    ) -> Self {
        if params.len() != 3 {
            panic!("Expected 3 parameter, got {}", params.len());
        }

        Self {
            model: model.clone(),
            objective: super::body::Objective::new(
                model,
                nalgebra::Point3::new(params[0], params[1], params[2]),
            ),
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

                        if let Ok(mut model) = self.model.try_write() {
                            model.update_boom_angle(angle_at_datum);
                            model.update_slew_angle(0.0);
                        };

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

                        if let Ok(mut model) = self.model.try_write() {
                            model.update_arm_angle(angle_at_datum);
                        };

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
                super::BODY_PART_BUCKET => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(BUCKET_ENCODER_RANGE, BUCKET_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        // TODO: Offset is negative.
                        // let angle_offset = core::deg_to_rad(36.8);

                        // TODO: REMOVE MOVE MOVE MOVE MOVE
                        // unsafe {
                        //     AGENT.update_boom_angle(core::deg_to_rad(60.0));
                        //     AGENT.update_arm_angle(core::deg_to_rad(-40.0));
                        //     AGENT.update_slew_angle(0.0);
                        // }
                        if let Ok(mut model) = self.model.try_write() {
                            model.update_boom_angle(core::deg_to_rad(60.0));
                            model.update_arm_angle(core::deg_to_rad(-40.0));
                            model.update_slew_angle(0.0);
                        };

                        debug!(
                            "Bucket Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                        );
                    }
                }
                super::BODY_PART_FRAME => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(SLEW_ENCODER_RANGE, SLEW_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_at_datum = angle;

                        if let Ok(mut model) = self.model.try_write() {
                            model.update_slew_angle(angle_at_datum);
                        };

                        debug!(
                            "Turn Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                        );
                    }
                }
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

        if let Ok(model) = self.model.try_read() {
            if let Some(effector_point) = model.effector_point() {
                debug!(
                    "Effector point: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                    effector_point.x, effector_point.y, effector_point.z,
                );

                let effector_point = model.effector_point_abs().unwrap();

                debug!(
                    "Effector point AGL: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                    effector_point.x, effector_point.y, effector_point.z,
                );

                if effector_point.y < 0.20 {
                    debug!("GROUND GROUND GROUND GROUND GROUND");
                }
            }
        }

        let rig_error = self.objective.erorr_diff();

        let mut motion_vector = vec![];

        if let Some(angle_boom_error) = rig_error.angle_boom() {
            let power_boom = (angle_boom_error * 15_000.0) as i16;
            let power_boom = if angle_boom_error.is_sign_positive() {
                (-power_boom).max(-20_000) - 12_000
            } else {
                (-power_boom).min(20_000) + 12_000
            };

            if angle_boom_error.abs() > 0.02 {
                motion_vector.push((super::Actuator::Boom, power_boom));
            }
        }

        if let Some(angle_arm_error) = rig_error.angle_arm() {
            let power_arm = (angle_arm_error * 15_000.0) as i16;
            let power_arm = if angle_arm_error.is_sign_positive() {
                power_arm.min(20_000) + 12_000
            } else {
                power_arm.max(-20_000) - 12_000
            };

            if angle_arm_error.abs() > 0.02 {
                motion_vector.push((super::Actuator::Arm, power_arm));
            }
        }

        if !motion_vector.is_empty() {
            Some(HydraulicMotion::Change(motion_vector))
        } else {
            None
        }
    }

    fn can_terminate(&self, _: &mut Context) -> bool {
        let rig_error = self.objective.erorr_diff();

        if let (Some(angle_boom_error), Some(angle_arm_error)) =
            (rig_error.angle_boom(), rig_error.angle_arm())
        {
            angle_arm_error.abs() < 0.02 && angle_boom_error.abs() < 0.02
        } else {
            false
        }
    }
}
