use crate::runtime::program::*;

use super::HydraulicMotion;

pub(super) struct KinematicProgram {
    domain: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    objective: super::body::Objective,
}

impl KinematicProgram {
    pub fn new(
        model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
        params: &Vec<f32>,
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

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
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
            }
        }

        let rig_error = self.objective.erorr_diff();

        let mut motion_vector = vec![];

        if let Some(error) = rig_error.angle_boom() {
            let power = super::consts::MOTION_PROFILE_BOOM.proportional_power_inverse(error);
            motion_vector.push((super::Actuator::Boom, power));
        }

        if let Some(error) = rig_error.angle_arm() {
            let power = super::consts::MOTION_PROFILE_ARM.proportional_power(error);
            motion_vector.push((super::Actuator::Arm, power));
        }

        if let Some(error) = rig_error.angle_slew() {
            let power = super::consts::MOTION_PROFILE_SLEW.proportional_power(error);
            motion_vector.push((super::Actuator::Slew, power));
        }

        if !motion_vector.is_empty() {
            Some(HydraulicMotion::Change(motion_vector))
        } else {
            None
        }
    }

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, _: &mut Context) -> bool {
        let rig_error = self.objective.erorr_diff();

        if let (Some(angle_boom_error), Some(angle_arm_error), Some(angle_slew_error)) = (
            rig_error.angle_boom(),
            rig_error.angle_arm(),
            rig_error.angle_slew(),
        ) {
            angle_arm_error.abs() < 0.02
                && angle_boom_error.abs() < 0.02
                && angle_slew_error.abs() < 0.02
        } else {
            false
        }
    }

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction. This method is called after `can_terminate`
    /// returns true and before the program is terminated.
    fn term_action(&self, _context: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Stop(vec![
            super::Actuator::Boom,
            super::Actuator::Arm,
            super::Actuator::Slew,
        ]))
    }
}
