use crate::runtime::operand::*;

use super::HydraulicMotion;

pub(super) struct KinematicProgram {
    domain: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
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
            domain: model.clone(),
            objective: super::body::Objective::from_point(
                model,
                nalgebra::Point3::new(params[0], params[1], params[2]),
            ),
        }
    }
}

#[async_trait::async_trait]
impl Program for KinematicProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Ok(mut domain) = self.domain.try_write() {
            domain.signal_update(&mut context.reader).await;
        }

        if let Ok(domain) = self.domain.try_read() {
            if let Some(effector_point) = domain.effector_point_abs() {
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

        if let Some(error) = rig_error.angle_boom() {
            let boom_profile = super::body::MotionProfile {
                scale: 15_000.0,
                offset: 12_000,
                limit: 20_000,
                cutoff: 0.02,
            };

            if let Some(power_boom) = boom_profile.proportional_power_inverse(error) {
                motion_vector.push((super::Actuator::Boom, power_boom));
            }
        }

        if let Some(error) = rig_error.angle_arm() {
            let arm_profile = super::body::MotionProfile {
                scale: 15_000.0,
                offset: 12_000,
                limit: 20_000,
                cutoff: 0.02,
            };

            if let Some(power_arm) = arm_profile.proportional_power(error) {
                motion_vector.push((super::Actuator::Arm, power_arm));
            }
        }

        if let Some(error) = rig_error.angle_slew() {
            let arm_profile = super::body::MotionProfile {
                scale: 10_000.0,
                offset: 10_000,
                limit: 20_000,
                cutoff: 0.02,
            };

            if let Some(power_slew) = arm_profile.proportional_power(error) {
                motion_vector.push((super::Actuator::Slew, power_slew));
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
